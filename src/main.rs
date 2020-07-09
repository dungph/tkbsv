mod parse;
mod export;

use {
    chrono::{offset::Utc, DateTime, NaiveDate, FixedOffset, Datelike},
    parse::parse,
    export::to_ics,
    std::{env, fs, io, time},
};

#[derive(Debug, Clone)]
pub struct Lesson {
    date: NaiveDate, 
    period: u8,
    place: String,
    class: String,
}

impl Lesson {
    pub fn new() -> Self {
        Lesson {
            date: NaiveDate::from_ymd(2020, 02, 20),
            period: 1,
            place: "Place".to_string(),
            class: "Class".to_string(),
        }
    }

    fn get_time(&self) -> (u32, u32, u32, u32) {
        match self.period{
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
            12 => (16, 45, 17, 30),
            13 => (18, 0, 18, 45),
            14 => (18, 50, 19, 35),
            15 => (19, 40, 20, 25),
            16 => (20, 30, 21, 00),
            _ => (0, 0, 0, 0),
        }
    }

    fn offset() -> FixedOffset {
        FixedOffset::east(7*3600)
    }

    pub fn mod_period(&mut self, p: u8) {
        self.period = p;
    }

    pub fn mod_date(&mut self, t: NaiveDate) {
        self.date = t;
    }
    pub fn mod_place(&mut self, p: &str) {
        self.place = p.to_string();
    }
    pub fn mod_class(&mut self, c: &str) {
        self.class = c.to_string();
    }
    pub fn begin(&self) -> DateTime<Utc> {
        let (h, m, _, _) = self.get_time();
        let dt = self.date.and_hms(h, m, 0);
        DateTime::from_utc(dt - Lesson::offset(), Utc)
        
    }
    pub fn end(&self) -> DateTime<Utc> {
        let (_, _, h, m) = self.get_time(); 
        let dt = self.date.and_hms(h, m, 0);
        DateTime::from_utc(dt - Lesson::offset(), Utc)
    }
    pub fn place(&self) -> String {
        self.place.clone()
    }
    pub fn class(&self) -> String {
        self.class.clone()
    }
}

fn main() {
}

