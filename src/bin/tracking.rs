use rusqlite::Connection;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use std::process::Command;
use json;
use chrono::Local;
use lazy_static::lazy_static;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    database: Option<String>,
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

fn main() {
    let mut db = if let Ok(s) = env::var("TRACKING_DB") {
        s
    } else {
        "tracking.db".to_string()
    };

    let args = Args::parse();
    if args.database.is_some() {
        db = args.database.unwrap();
    }
    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "create table if not exists tracking (
            id integer primary key,
            class text not null,
            title text not null,
            ts integer not null unique
    );",
    (),
    ).unwrap();
    loop {
        
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
        conn.execute(
            "INSERT INTO tracking (class, title, ts) values (?1, ?2, ?3);",
                    &[&focus_entry.class, &focus_entry.title, &focus_entry.ts],
        ).unwrap();
        sleep(Duration::from_secs(10));
    }
}
