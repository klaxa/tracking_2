use rusqlite::Connection;
use rusqlite::ErrorCode::*;
use std::collections::VecDeque;
use std::env;
use std::path::Path;
use std::time::Duration;
use std::process::Command;
use json;
use chrono::Local;
use lazy_static::lazy_static;
use clap::Parser;
use tokio::time;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    database: Option<String>,

    #[arg(short, long)]
    idlefile: Option<String>
}


#[derive(Debug)]
struct FocusEntry {
    class: String,
    title: String,
    ts: String,
}

lazy_static! {
    static ref EMPTY: json::JsonValue = json::object!{};
}

fn get_focused_window(obj: &json::JsonValue) -> &json::JsonValue {
    if obj["focused"] == true {
        return obj;
    }
    if !obj["nodes"].is_null() {
        for node in obj["nodes"].members() {
            let res = get_focused_window(node);
            if res["focused"] == true {
                return res;
            }
        }
    }

    return &EMPTY;
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut db = if let Ok(s) = env::var("TRACKING_DB") {
        s
    } else {
        "tracking.db".to_string()
    };

    let args = Args::parse();
    if args.database.is_some() {
        db = args.database.unwrap();
    }

    let mut idle_file = if let Ok(s) = env::var("TRACKING_IDLE_FILE") {
        s
    } else {
        "/tmp/tracking-idle".to_string()
    };

    if args.idlefile.is_some() {
        idle_file = args.idlefile.unwrap();
    }


    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "create table if not exists tracking (
            id integer primary key,
            class text not null,
            title text not null,
            idle integer not null,
            ts integer not null unique
    );",
    (),
    ).unwrap();

    let mut interval = time::interval(Duration::from_secs(10));
    let mut cache: VecDeque<[String;4]> = VecDeque::new();

    eprintln!("Started logging to {} at {}", db, Local::now());

    'main: loop {
        interval.tick().await;

        let output = Command::new("i3-msg").args(["-t", "get_tree"]).output().expect("Could not call i3-msg -t get_tree");
        let output = String::from_utf8(output.stdout).unwrap_or_default();
        let output = json::parse(&output).unwrap_or(json::JsonValue::Null);
        let focus = get_focused_window(&output);

        let now = Local::now().timestamp().to_string();
        let focus_entry = if focus["window_properties"].is_null() {
            FocusEntry {
                class: "idle".to_string(),
                title: "idle".to_string(),
                ts: now
            }
        } else {
            FocusEntry {
                class: focus["window_properties"]["class"].to_string(),
                title: focus["window_properties"]["title"].to_string(),
                ts: now
            }
        };

        let idle = if Path::new(&idle_file).exists() || focus_entry.class.eq("idle") || focus_entry.class.eq("feh") {
            "1".to_string()
        } else {
            "0".to_string()
        };

        let entry = [focus_entry.class.clone(), focus_entry.title.clone(), idle, focus_entry.ts.clone()];

        while cache.len() > 0 {
            eprintln!("Cache not empty, attempting to write to db");
            if let Some(centry) = cache.front() {
                eprintln!("Attempting to insert entry: {:?}", entry);
                if let Err(e) = conn.execute(
                        "INSERT INTO tracking (class, title, idle, ts) values (?1, ?2, ?3, ?4);",
                                    centry.clone(),
                    ) {
                    eprintln!("Error logging cached entry: {}", e);
                    if e.sqlite_error().is_some_and(|e| e.code.eq(&DatabaseBusy) || e.code.eq(&DatabaseLocked) || e.code.eq(&DiskFull)) {
                        eprintln!("Database busy, locked or disk full, caching in memory: {:?}", entry);
                        cache.push_back(entry);
                        continue 'main;
                    } else {
                        eprintln!("Unrecoverable error, dropping entry: {:?}", entry);
                        cache.pop_front();
                    }
                } else {
                    eprintln!("Successfully write cached entry to db: {:?}", entry);
                    cache.pop_front();
                }
            }
        }

        if let Err(e) = conn.execute(
                "INSERT INTO tracking (class, title, idle, ts) values (?1, ?2, ?3, ?4);",
                        entry.clone(),
            ) {
                eprintln!("Error logging entry: {}", e);
                eprintln!("Adding entry to cache: {:?}", entry);
                cache.push_back(entry);
            }
    }

}
