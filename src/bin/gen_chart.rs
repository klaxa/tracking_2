use num_traits::cast::FromPrimitive;
use rusqlite::Connection;
use chrono::prelude::*;
use chrono::naive::Days;
use chrono::{Datelike, Local, Duration};
use std::collections::HashMap;
use std::env;
use rand::{thread_rng, Rng};
use clap::{Parser, ArgAction};
use plotters::prelude::*;
use plotters::backend::BitMapBackend;
use lazy_static::lazy_static;

const BACKGROUND: RGBColor = RGBColor(128, 128, 128);
const TIME_MARGIN: i32 = 50;
const DATE_MARGIN: i32 = 30;
const TEXT_BLOCK_SIZE: i32 = 24;
const TEXT_MARGIN: i32 = 4;
const DAILY_TIME_MARGIN: i32 = 6 * TEXT_BLOCK_SIZE;
const DAY_MARGIN: i32 = 5;
const DAY_WIDTH: i32 = 140;
const BAR_MARGIN: i32 = 20;
const BAR_WIDTH: i32 = DAY_WIDTH - BAR_MARGIN * 2;
const LEGEND_MARGIN: i32 = 5;


lazy_static! {
    static ref COLORS: Vec<RGBColor> = vec![RGBColor(255, 0, 0), RGBColor(0, 255, 0), RGBColor(0, 0, 255), RGBColor(255, 255, 0), RGBColor(255, 0, 255), RGBColor(0, 255, 255), RGBColor(255, 255, 255), RGBColor(0, 0, 0), RGBColor(85, 85, 85), RGBColor(170, 170, 170), RGBColor(128, 255, 0), RGBColor(128, 0, 255), RGBColor(255, 128, 0)];
}


#[derive(Debug, Clone)]
struct Row {
    class: String,
    ts: i64,
}

#[derive(Debug)]
struct TaskClass {
    class: String,
    count: i64,
    color: RGBColor
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The database to connect to, defaults to 'tracking.db' can also be set with TRACKING_DB environment variable")]
    database: Option<String>,

    #[arg(short, long, help = "The start date in the format YYYY-MM-DD, defaults to today")]
    start: Option<String>,

    #[arg(short, long, help = "The end date in the format YYYY-MM-DD, defaults to today")]
    end: Option<String>,

    #[arg(short, long, help = "Only generate graph for the start date", action = ArgAction::SetTrue)]
    today: Option<bool>,

    #[arg(short, long, help = "Only generate graph for the week containing start date", action = ArgAction::SetTrue)]
    week: Option<bool>,

    #[arg(short, long, help = "Only generate graph for the month containing start date", action = ArgAction::SetTrue)]
    month: Option<bool>,

    #[arg(short, long, help = "Inlcude idle time in graph", action = ArgAction::SetTrue)]
    idle: Option<bool>,

    #[arg(long, help = "Height of the 24 hour portion of the graph, defaults to 500 px", default_value_t = 500)]
    height: i32
}


fn fmt(ts: Duration) -> String {
    format!("{}:{:0>2}:{:0>2} ", ts.num_hours(), ts.num_minutes() % 60, ts.num_seconds() % 60)
}

fn datestr_to_local(s: &str, end: bool) -> Result<DateTime<Local>, ()> {
    let parts = s.split('-');
    let parts: Vec<&str> = parts.collect();
    if parts.len() != 3 {
        return Err(());
    }

    let y: i32 = parts[0].parse().unwrap();
    let m: u32 = parts[1].parse().unwrap();
    let d: u32 = parts[2].parse().unwrap();
    if end {
        return Ok(Local.with_ymd_and_hms(y, m, d, 23, 59, 59).unwrap());
    }
    return Ok(Local.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap());
}


fn hour_lines(backend: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>, p_per_h: f32) {
    for h in 0..25 {
        let y = (h as f32 * p_per_h + DATE_MARGIN as f32) as i32;
        backend.draw(&PathElement::new(vec![(0, y), (DAY_WIDTH, y)], &BLACK)).unwrap();
        if h == 12 {
            backend.draw(&PathElement::new(vec![(0, y - 1), (DAY_WIDTH, y - 1)], &BLACK)).unwrap();
            backend.draw(&PathElement::new(vec![(0, y + 1), (DAY_WIDTH, y + 1)], &BLACK)).unwrap();
        }
        backend.present().unwrap();
    }
}


fn calculate_y(ts: i64, height: i32) -> i32 {

    let dt = DateTime::<Utc>::from_naive_utc_and_offset(NaiveDateTime::from_timestamp_opt(ts, 0).unwrap(), Utc).with_timezone(&Local);
    let hour_height = height as f32 / 24.0;
    let y = hour_height * dt.hour() as f32 + hour_height * (dt.minute() as f32 / 60.0);
    return y as i32;
}

fn main() {
    let mut db = if let Ok(s) = env::var("TRACKING_DB") {
        s
    } else {
        "tracking.db".to_string()
    };

    let now = Local::now();
    let args = Args::parse();
    if args.database.is_some() {
        db = args.database.unwrap();
    }

    let mut end = if args.end.is_some() {
        args.end.unwrap()
    } else {
        format!("{}-{}-{}", now.year(), now.month(), now.day())
    };

    let mut start = if args.start.is_some() {
        args.start.unwrap()
    } else {
        format!("{}-{}-{}", now.year(), now.month(), now.day())
    };

    let now = datestr_to_local(&start, false).unwrap();

    if args.week.is_some() && args.week.unwrap() {
        let mut check_day = now.clone();
        while !check_day.weekday().eq(&chrono::Weekday::Mon) {
            check_day = check_day.checked_sub_days(Days::new(1)).unwrap();
        }
        start = format!("{}-{}-{}", check_day.year(), check_day.month(), check_day.day());
        check_day = check_day.checked_add_days(Days::new(6)).unwrap();
        end = format!("{}-{}-{}", check_day.year(), check_day.month(), check_day.day());
    }

    if args.month.is_some() && args.month.unwrap() {
        start = format!("{}-{}-1", now.year(), now.month());
        let mut check_day = now.clone();
        let month = now.month();
        while check_day.month() == month {
            check_day = check_day.checked_add_days(Days::new(1)).unwrap();
        }
        check_day = check_day.checked_sub_days(Days::new(1)).unwrap();
        end = format!("{}-{}-{}", check_day.year(), check_day.month(), check_day.day());
    }

    if args.today.is_some() && args.today.unwrap() {
        end = start.clone();
    }

    let start = datestr_to_local(&start, false).unwrap();
    let end = datestr_to_local(&end, true).unwrap();
    println!("start: {}\nend:   {}", start, end);

    let conn = Connection::open(&db).unwrap();
    let query = if args.idle.is_some() && args.idle.unwrap() {
        format!("select class, ts FROM tracking where ts > {} and ts < {} order by ts asc;", start.timestamp(), end.timestamp())
    } else {
        format!("select class, ts FROM tracking where ts > {} and ts < {} and class not like 'idle' and class not like 'feh' order by ts asc;", start.timestamp(), end.timestamp())
    };
    let mut stmt = conn.prepare(&query).unwrap();

    let res = stmt.query_map((), |row| {
        Ok(Row {
            class: row.get(0)?,
            ts:    row.get(1)?,
        })
    }).unwrap();

    let query = if args.idle.is_some() && args.idle.unwrap() {
        format!("select class, count (*) FROM tracking where ts > {} and ts < {} group by class order by count (*) desc;", start.timestamp(), end.timestamp())
    } else {
        format!("select class, count (*) FROM tracking where ts > {} and ts < {} and class not like 'idle' and class not like 'feh' group by class order by count (*) desc;", start.timestamp(), end.timestamp())
    };
    let mut stmt = conn.prepare(&query).unwrap();

    let counts = stmt.query_map((), |row| {
        Ok(TaskClass {
            class: row.get(0)?,
            count: row.get(1)?,
            color: BLACK
        })
    }).unwrap();

    let mut count_data = vec![];
    let mut ci = 0;
    let mut color_map = HashMap::new();
    let mut total_count = 0;

    counts.for_each(|c| {
        let c = c.unwrap();
        let color = if ci >= COLORS.len() {
            let mut rng = thread_rng();
            let gray = rng.gen_range(10..245);
            RGBColor(gray, gray, gray)
        } else {
            COLORS[ci]
        };
        ci += 1;
        count_data.push(TaskClass{class: c.class.clone(), count: c.count, color});
        total_count += c.count;
        color_map.insert(c.class, color);
    });


    // we made it this far, we can draw stuff now



    let day_height = args.height + DATE_MARGIN;
    let day_graph_height = day_height + DAY_MARGIN + DAILY_TIME_MARGIN;
    let mut cur = start.clone();
    let mut day_graphs = vec![];
    let legend_height = TEXT_BLOCK_SIZE * count_data.len() as i32 + LEGEND_MARGIN;
    let height = day_graph_height + legend_height;
    let p_per_h = args.height as f32 / 24.0;
    let style = ("hack", (TEXT_BLOCK_SIZE - 2 * TEXT_MARGIN) as f32).into_font();
    let mut day_data = vec![];
    let mut cur_day_data = vec![];
    let mut day_end = cur.with_hour(23).unwrap().with_minute(59).unwrap().with_second(59).unwrap().timestamp();

    res.for_each(|r| {
        let row = r.unwrap();
        while row.ts > day_end {
           cur = cur.checked_add_days(Days::new(1)).unwrap();
           day_end = cur.with_hour(23).unwrap().with_minute(59).unwrap().with_second(59).unwrap().timestamp();
           day_data.push(cur_day_data.clone());
           cur_day_data.clear();
        }
        cur_day_data.push(row);
    });
    day_data.push(cur_day_data);

    cur = start.clone();

    let mut week_started_hours = Duration::seconds(0);
    let mut week_actual_hours = Duration::seconds(0);
    let mut month_started_hours = Duration::seconds(0);
    let mut month_actual_hours = Duration::seconds(0);


    for cur_day_data in day_data {

        let mut img = vec![0u8; (day_graph_height * DAY_WIDTH * 3) as usize];
        {
            let backend = BitMapBackend::with_buffer(&mut img, (DAY_WIDTH as u32, day_graph_height as u32)).into_drawing_area();
            backend.fill(&BACKGROUND).unwrap();
            let month = Month::from_u32(cur.month()).unwrap().name();
            let line = format!("{:.3}, {:2}. {:.3} {}", cur.weekday(), cur.day(), month, cur.year());
            backend.draw(&Text::new(line, (0, 5), style.clone())).unwrap();
            hour_lines(&backend, p_per_h);
            let mut secs = 0;
            for task in cur_day_data {
                let y = calculate_y(task.ts, args.height) + DATE_MARGIN;
                let color = color_map.get(&task.class).unwrap();
                backend.draw(&PathElement::new(vec![(BAR_MARGIN, y), (BAR_MARGIN + BAR_WIDTH, y)], color)).unwrap();
                secs += 10;
            }
            let mut y = day_height;
            let duration = Duration::seconds(secs);
            backend.draw(&Text::new(fmt(duration), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
            week_actual_hours = week_actual_hours.checked_add(&duration).unwrap();
            month_actual_hours = month_actual_hours.checked_add(&duration).unwrap();
            y += TEXT_BLOCK_SIZE;
            let duration = if duration.num_minutes() > 15 { Duration::hours(duration.num_hours() + 1) } else { Duration::hours(duration.num_hours()) };
            backend.draw(&Text::new(fmt(duration), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
            week_started_hours = week_started_hours.checked_add(&duration).unwrap();
            month_started_hours = month_started_hours.checked_add(&duration).unwrap();
            y += TEXT_BLOCK_SIZE;

            if cur.weekday().eq(&Weekday::Sun) {
                backend.draw(&Text::new(fmt(week_actual_hours), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
                y += TEXT_BLOCK_SIZE;
                backend.draw(&Text::new(fmt(week_started_hours), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
                y += TEXT_BLOCK_SIZE;
                week_actual_hours = Duration::seconds(0);
                week_started_hours = Duration::seconds(0);
            }

            let tomorrow = cur.checked_add_days(Days::new(1)).unwrap();
            if tomorrow.month() != cur.month() {
                backend.draw(&Text::new(fmt(month_actual_hours), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
                y += TEXT_BLOCK_SIZE;
                backend.draw(&Text::new(fmt(month_started_hours), (BAR_MARGIN * 2 + TEXT_MARGIN, y + TEXT_MARGIN), style.clone())).unwrap();
                month_actual_hours = Duration::seconds(0);
                month_started_hours = Duration::seconds(0);
            }

            backend.present().unwrap();
        }
        day_graphs.push(img);
        cur = cur.checked_add_days(Days::new(1)).unwrap();
    }

    let width = day_graphs.len() as i32 * DAY_WIDTH + TIME_MARGIN;

    let mut legend = vec![0u8; (width * legend_height * 3) as usize];

    {
        let backend = BitMapBackend::with_buffer(&mut legend, (width as u32, legend_height as u32)).into_drawing_area();
        backend.fill(&BACKGROUND).unwrap();
        let mut y = 0;
        for c in count_data {
            let s = ShapeStyle{color: c.color.to_rgba(), filled: true, stroke_width: 1};
            backend.draw(&Rectangle::new([(TEXT_MARGIN, y + TEXT_MARGIN), (TEXT_BLOCK_SIZE - TEXT_MARGIN, y + TEXT_BLOCK_SIZE - TEXT_MARGIN)], s)).unwrap();
            let mut line = format!(": {} {} ({:.2}%)", c.class, fmt(Duration::seconds(c.count * 10)), 100.0 * c.count as f32 / total_count as f32);
            if y == 0 {
                line += " total: ";
                line += &fmt(Duration::seconds(total_count * 10));
            }
            backend.draw(&Text::new(line, (TEXT_BLOCK_SIZE, y + TEXT_MARGIN), style.clone())).unwrap();
            y += TEXT_BLOCK_SIZE;
        }
        backend.present().unwrap();
    }

    let mut times = vec![0u8; (TIME_MARGIN * day_graph_height * 3) as usize];

    {
        let backend = BitMapBackend::with_buffer(&mut times, (TIME_MARGIN as u32, day_graph_height as u32)).into_drawing_area();
        backend.fill(&BACKGROUND).unwrap();
        let mut i = DATE_MARGIN as f32;
        let mut h = 0;
        while i < day_height as f32 {
            let line = format!("{:>2}:00", h);
            backend.draw(&Text::new(line, (TEXT_MARGIN, i as i32 - TEXT_MARGIN), style.clone())).unwrap();
            h += 1;
            i += p_per_h;
        }
        backend.draw(&Text::new("24:00", (TEXT_MARGIN, i as i32 - TEXT_MARGIN), style.clone())).unwrap();
        backend.present().unwrap();
    }

    {
        let mut backend = BitMapBackend::new("chart.png", (width as u32, height as u32));
        for i in 0..day_graphs.len() {
            backend.blit_bitmap((i as i32 * DAY_WIDTH + TIME_MARGIN, 0), (DAY_WIDTH as u32, day_graph_height as u32), &day_graphs[i]).unwrap();
        }
        backend.blit_bitmap((0, day_graph_height), (width as u32, legend_height as u32), &legend).unwrap();
        backend.blit_bitmap((0, 0), (TIME_MARGIN as u32, day_graph_height as u32), &times).unwrap();
        backend.present().unwrap();
    }
}
