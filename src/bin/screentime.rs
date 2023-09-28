use rusqlite::Connection;
use chrono::prelude::*;
use chrono::naive::Days;
use chrono::{Datelike, Local, Duration};
use std::env;

#[derive(Debug)]
struct Res {
    class: String,
    count: i64,
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        return a;
    }
    b
}

fn fmt(ts: Duration) -> String {
    format!("{}:{:0>2}:{:0>2} ", ts.num_hours(), ts.num_minutes() % 60, ts.num_seconds() % 60)
}

fn main() {
    let mut db = if let Ok(s) = env::var("TRACKING_DB") {
        s
    } else {
        "tracking.db".to_string()
    };
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        db = args[1].clone();
    }
    let conn = Connection::open(&db).unwrap();
    let now = Local::now();
    let zero_hour = Local.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0).unwrap();
    let twenty_fourth_hour = zero_hour.checked_add_days(Days::new(1)).unwrap();
    let query = &format!("select class, count (*) FROM tracking where ts > {} and ts < {} and class not like 'idle' and class not like 'feh' group by class order by count (*) desc;", zero_hour.timestamp(), twenty_fourth_hour.timestamp());
    let mut stmt = conn.prepare(query).unwrap();
    
    let res = stmt.query_map((), |row| {
        Ok(Res {
            class: row.get(0)?,
            count: row.get(1)?,
        })
    }).unwrap();
    //let count: i64 = conn.query_row(query, [], |row| row.get(0)).unwrap();
    
    
    let mut counts = vec![];
    let mut count = 0;
    for r in res {
        let c = r.unwrap();
        count += c.count;
        if c.class != "feh" {
            counts.push(c);
        }
    }
    
    let nb = min(counts.len(), 3);
    let mut output = fmt(Duration::seconds(count * 10));
    for i in 0..nb {
        output += counts[i].class.as_str();
        output += ": ";
        output += &fmt(Duration::seconds(counts[i].count * 10));
    }
    println!("{}", output);
}