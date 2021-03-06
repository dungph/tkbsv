mod get;
mod utils;

use async_std::task;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use get::get_html;
use scraper::{Html, Selector};
use std::error::Error;
use std::str::FromStr;
use tide::{http::Mime, Request, Response, StatusCode};
use utils::{get_period_time, parse_list_uint, parse_weekday, to_ics};
use std::collections::HashMap;

fn main() {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|_| async { 
            let mut res = Response::new(StatusCode::Accepted);
            res.set_content_type(tide::http::mime::HTML);
            res.set_body(include_str!("index.html"));
            Ok(res)
        });
        app.at("/ics/*").get(|req: Request<()>| async move {
            let info: String = req.url().path().replace("/ics/", "");
            if info.find('_').is_some() {
                let vec = info.split('_').collect::<Vec<&str>>();
                let content = process(&vec[0].to_uppercase(), &vec[1]).await;
                let content = to_ics(content.unwrap_or(Vec::new()));
                let mut res = Response::new(StatusCode::Accepted);
                res.set_content_type(Mime::from_str("text/calendar").unwrap());
                res.set_body(content);
                Ok(res)
            } else {
                Ok("Example CT010101_Passwd".into()) 
            }
        });
        app.at("/json/*").get(|req: Request<()>| async move {
            let path = req.url().path().replace("/json/", "");
            let (usr, pwd) = path.split_at(path.find('/').unwrap_or(0));
            let pwd: String = pwd[1..].to_string();
            let vec = process(&usr.to_uppercase(), &pwd).await.unwrap_or(Vec::new());
            
            let doc = vec.iter()
                .map(|dat| dat.to_map())
                .collect::<Vec<HashMap<&'static str, String>>>();

            let mut res = Response::new(StatusCode::Accepted);
            res.set_content_type(Mime::from_str("application/json")?);
            res.set_body(tide::Body::from_json(&doc)?);
            Ok(res)
        });
        let port = std::env::var("PORT").unwrap_or("8080".to_string());
        app.listen(format!("0.0.0.0:{}", port)).await.unwrap();
    });
}

async fn process(usr: &str, pwd: &str) 
    -> Result<Vec<Data>, Box<dyn Error + Send + Sync>> 
{
    let vec = parse_html(get_html(usr, pwd).await?)?;
    Ok(vec.iter()
        .map(|(cl, ts, ps)| Data::parse(cl, ts, ps))
        .flatten()
        .collect())
}

#[derive(Debug)]
pub struct Data {
    class: String,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
    place: String,
}

impl Data {
    pub fn to_map(&self) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();
        map.insert("title", format!("{}\n{}", self.class, self.place));
        map.insert("start", utils::to_utc(self.time_begin).to_rfc3339());
        map.insert("end", utils::to_utc(self.time_end).to_rfc3339());
        map
    }
    pub fn class(&self) -> String {
        self.class.to_string()
    }
    pub fn place(&self) -> String {
        self.place.to_string()
    }
    pub fn begin(&self) -> NaiveDateTime {
        self.time_begin
    }
    pub fn end(&self) -> NaiveDateTime {
        self.time_end
    }
    fn parse(class: &str, times: &str, places: &str) -> Vec<Data> {
        let mut default = String::new(); 
        let mut map = HashMap::new();
        if places.find("(").is_some() {
            places.split("(")
                .skip(1)
                .map(|s| s
                    .split(")")
                    .map(|s| s.trim())
                    .collect::<Vec<&str>>()
                ).map(|vec| (vec.get(0).unwrap_or(&"1").clone(),
                            vec.get(1).unwrap_or(&"N/A").clone())
                ).map(|(i, p)| (parse_list_uint(i), p))
                .map(|(vec, p)| vec
                    .iter()
                    .map(|i| map.insert(i.clone() as usize, p.to_string()))
                    .all(|_| true)
                ).all(|_| true);
        } else {
            default = places.to_string();
        }

        times
            .split("Từ ")
            .skip(1)
            .enumerate()
            .map(|(i, s)| s.replace(&format!("({})", i + 1), ""))
            .map(|s| {
                s.split(':')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>()
            })
            .map(|vec| (vec[0].clone(), vec[1].clone()))
            .map(|(r, o)| (Data::parse_range(&r), Data::parse_wd_period(&o)))
            .map(|(r, d)| Data::merge_date_time(r, d))
            .enumerate()
            .map(|(i, vec)| {
                vec.iter()
                    .map(|(b, e)| Data {
                        class: class.to_string(),
                        time_begin: *b,
                        time_end: *e,
                        place: map.get(&(i+1)).unwrap_or(&default).clone(),
                    })
                    .collect::<Vec<Data>>()
            })
            .flatten()
            .collect()
    }

    fn merge_date_time(
        range: (NaiveDate, NaiveDate),
        time: Vec<(Weekday, (NaiveTime, NaiveTime))>,
    ) -> Vec<(NaiveDateTime, NaiveDateTime)> {
        time.iter()
            .map(|(wd, (bt, et))| {
                let mut vec = Vec::new();
                let mut date = range.0 + Duration::days(wd.num_days_from_monday() as i64);
                while date < range.1 {
                    vec.push((date.and_time(*bt), date.and_time(*et)));
                    date += Duration::days(7);
                }
                vec
            })
            .flatten()
            .collect()
    }
    fn parse_wd_period(all: &str) -> Vec<(Weekday, (NaiveTime, NaiveTime))> {
        fn get_ps_str_time(ps: &str) -> (NaiveTime, NaiveTime) {
            let vec = parse_list_uint(ps);
            let b = vec[0];
            let e = vec.last().unwrap();
            (get_period_time(b).0, get_period_time(*e).1)
        }

        all.trim()
            .split("Thứ")
            .skip(1)
            .map(|s| s.split("tiết").map(|s| s.trim()).collect::<Vec<&str>>())
            .map(|vec| (vec[0], vec[1]))
            .map(|(wd, ps)| (parse_weekday(wd), get_ps_str_time(ps)))
            .collect()
    }

    fn parse_range(range: &str) -> (NaiveDate, NaiveDate) {
        let vec = range
            .split("đến")
            .map(|s| s.trim())
            .map(|s| Data::parse_date(s))
            .collect::<Vec<NaiveDate>>();
        (vec[0], vec[1])
    }

    fn parse_date(s: &str) -> NaiveDate {
        let dmy = parse_list_uint(s);
        NaiveDate::from_ymd(dmy[2] as i32, dmy[1], dmy[0])
    }
}

fn parse_html(doc: String) -> Result<Vec<(String, String, String)>, Box<dyn Error + Send + Sync>> {
    let all = Html::parse_document(&doc);
    let select = Selector::parse(r#"tr[class="cssListItem"]"#).unwrap();
    let select_alt = Selector::parse(r#"tr[class="cssListAlternativeItem"]"#).unwrap();
    let body = all.select(&select).chain(all.select(&select_alt));
    let each_col = Selector::parse("td").unwrap();

    fn to_string<'a, T: Iterator<Item = R> + 'a, R: AsRef<str>>(iter: T) -> String {
        let mut ret = String::new();
        let _ = iter
            .map(|s| ret.push_str(s.as_ref().trim()))
            .collect::<Vec<()>>();
        ret
    }
    Ok(body
        .map(|n| {
            let vec = n
                .select(&each_col)
                .map(|e| to_string(e.text()))
                .collect::<Vec<String>>();
            (vec[1].clone(), vec[3].clone(), vec[4].clone())
        })
        .collect())
}
