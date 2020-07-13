use crate::Lesson;
use {
    calamine::{Reader, Xls},
    chrono::{Duration, NaiveDate},
    std::io::Cursor,
};

pub fn parse(data: Vec<u8>) -> Result<Vec<Lesson>, String> {
    let data = Cursor::new(data);
    let mut reader: Xls<Cursor<Vec<u8>>> =
        Reader::new(data).map_err(|_| "Cannot open this file")?;
    let sheet_name = reader.sheet_names()[0].clone();
    let range = reader
        .worksheet_range(sheet_name.as_str())
        .ok_or("Error")?
        .map_err(|_| "Error")?;

    let rows = range.rows();

    let focused = rows.filter(|row| row[0].is_int());

    let lessons: Vec<Lesson> = focused
        .map(|row| {
            (
                row[5].get_string().unwrap_or("N/A"),
                row[7].get_string().unwrap_or("N/A"),
            )
        })
        .map(|(class, others)| parse_all(class, others))
        .flatten()
        .collect();

    Ok(lessons)
}

fn parse_all(class: &str, others: &str) -> Vec<Lesson> {
    others
        .split("Từ")
        .map(|s| s.trim())
        .skip(1)
        .map(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            (parts[0], parts[1])
        })
        .map(|(f, s)| (f.trim(), s.trim()))
        .map(|(f, s)| (split_range(f), get_wd_period_place(s)))
        .map(|(r, vec)| map_rt_lesson(class, r, vec))
        .flatten()
        .collect()
}

fn map_rt_lesson(
    class: &str,
    range: (NaiveDate, NaiveDate),
    wd_period_places: Vec<(u32, (u8, u8), String)>,
) -> Vec<Lesson> {
    wd_period_places
        .iter()
        .map(|(wd, p, pl)| (map_wd_to_day(range, *wd), p, pl))
        .map(|(vec_d, p, pl)| {
            vec_d
                .iter()
                .map(|d| (d.clone(), p.clone(), pl.clone()))
                .collect::<Vec<(NaiveDate, (u8, u8), String)>>()
        })
        .flatten()
        .map(|(d, p, pl)| {
            let mut l = Lesson::new();
            l.mod_date(d);
            l.mod_period(p);
            l.mod_place(&pl);
            l.mod_class(class);
            l
        })
        .map(|v| v.clone())
        .collect()
}

fn map_wd_to_day(range: (NaiveDate, NaiveDate), wd: u32) -> Vec<NaiveDate> {
    let mut vec = Vec::new();
    let mut date = range.0 + Duration::days(wd as i64 - 2);
    while date < range.1 {
        vec.push(date);
        date += Duration::days(7);
    }
    vec
}

fn get_wd_period_place(info: &str) -> Vec<(u32, (u8, u8), String)> {
    info.lines()
        .map(|line| line.trim_matches(&['T', 'h', 'ứ', ' ', '\n', '\t'] as &[_]))
        .map(|line| line.trim())
        .map(|line| line.split("tiết").collect::<Vec<&str>>())
        .map(|vec| (vec[0], vec[1]))
        .map(|(f, s)| (f.trim(), s.trim()))
        .map(|(f, s)| (f.parse::<u32>().unwrap_or(2), s))
        .map(|(w, s)| (w, s.split("tại").collect::<Vec<&str>>()))
        .map(|(w, vec)| {
            (
                w,
                vec.get(0).unwrap().clone(),
                vec.get(1).unwrap_or(&"N/A").clone(),
            )
        })
        .map(|(w, ps, pl)| {
            let periods = ps.trim()
                .split(',')
                .map(|p| p.parse::<u8>().unwrap_or(1))
                .collect::<Vec<u8>>();
            (w, (*periods.first().unwrap_or(&1), *periods.last().unwrap_or(&16)), pl.to_string())

        })
        .collect()
}

fn split_range(range: &str) -> (NaiveDate, NaiveDate) {
    let v: Vec<NaiveDate> = range
        .split("đến")
        .map(|d| d.trim())
        .map(|d| {
            d.split('/')
                .map(|s| s.parse().unwrap_or(0))
                .collect::<Vec<i32>>()
        })
        .map(|dmy| NaiveDate::from_ymd(dmy[2], dmy[1] as u32, dmy[0] as u32))
        .collect();
    (v[0], v[1])
}
