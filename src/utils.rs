use crate::Data;
use chrono::{offset::Utc, DateTime, FixedOffset, NaiveDateTime, NaiveTime, Weekday};
use icalendar::{Calendar, Component, Event};

pub fn parse_list_uint(s: &str) -> Vec<u32> {
    s.trim_matches(|c: char| !c.is_digit(10))
        .split(|c: char| !c.is_digit(10))
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

pub fn parse_weekday(s: &str) -> Weekday {
    let d = s
        .trim_matches(|c: char| !c.is_digit(10))
        .parse::<u8>()
        .unwrap_or(8);
    match d {
        2 => Weekday::Mon,
        3 => Weekday::Tue,
        4 => Weekday::Wed,
        5 => Weekday::Thu,
        6 => Weekday::Fri,
        7 => Weekday::Sat,
        _ => Weekday::Sun,
    }
}

pub fn get_period_time(n: u32) -> (NaiveTime, NaiveTime) {
    let (sh, sm, eh, em) = match n {
        1 => (7, 0, 7, 45),
        2 => (7, 50, 8, 35),
        3 => (8, 40, 9, 25),
        4 => (9, 35, 10, 20),
        5 => (10, 25, 11, 10),
        6 => (11, 15, 12, 0),
        7 => (12, 30, 13, 15),
        8 => (13, 20, 14, 5),
        9 => (14, 10, 14, 55),
        10 => (15, 5, 15, 50),
        11 => (15, 55, 16, 40),
        12 => (15, 45, 17, 30),
        13 => (18, 0, 18, 45),
        14 => (18, 50, 19, 35),
        15 => (19, 40, 20, 25),
        16 => (20, 30, 21, 15),
        _ => (21, 15, 21, 20),
    };
    (
        NaiveTime::from_hms(sh, sm, 0),
        NaiveTime::from_hms(eh, em, 0),
    )
}

pub fn to_utc(t: NaiveDateTime) -> DateTime<Utc> {
    let offset = FixedOffset::east(7 * 3600);
    DateTime::from_utc(t - offset, Utc)
}

pub fn to_ics(vec: Vec<Data>) -> Vec<u8> {
    let events = vec
        .iter()
        .map(|lesson| {
            Event::new()
                .summary(lesson.class().as_str())
                .description(format!("Địa điểm: {}", lesson.place()).as_str())
                .location(&lesson.place().trim())
                .starts(to_utc(lesson.begin()))
                .ends(to_utc(lesson.end()))
                .done()
        })
        .collect::<Vec<Event>>();

    let mut cal = Calendar::new();
    cal.name("TKBSV");
    cal.extend(events);

    format!("{}", cal).into_bytes()
}
