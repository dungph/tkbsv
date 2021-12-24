mod database;
mod get_data;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use get_data::process;
use icalendar::{Calendar, Component, Event};
use std::collections::HashMap;
use std::str::FromStr;
use tide::{http::Mime, Error, Request, Response, StatusCode};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Data {
    class: String,
    time_begin: DateTime<Utc>,
    time_end: DateTime<Utc>,
    place: String,
}

impl Data {
    pub fn to_json_map(&self) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();
        map.insert("title", format!("{}\n{}", self.class, self.place));
        map.insert("start", (self.time_begin).to_rfc3339());
        map.insert("end", (self.time_end).to_rfc3339());
        map
    }
    pub fn class(&self) -> String {
        self.class.to_string()
    }
    pub fn place(&self) -> String {
        self.place.to_string()
    }
    pub fn to_ics_event(&self) -> Event {
        Event::new()
            .summary(self.class().as_str())
            .description(format!("Địa điểm: {}", self.place()).as_str())
            .location(self.place().trim())
            .starts(self.time_begin)
            .ends(self.time_end)
            .done()
    }
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    tide::log::start();
    database::migrate().await?;
    let mut app = tide::new();
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Accepted);
        res.set_content_type(tide::http::mime::HTML);
        res.set_body(include_str!("index.html"));
        Ok(res)
    });
    app.at("/ics/*").get(|req: Request<()>| async move {
        let info: String = req.url().path().replace("/ics/", "");
        let (usr, pwd) = info
            .split_once('_')
            .ok_or_else(|| Error::new(400, anyhow!("Example CT010101_Passwd")))?;
        let usr = usr.to_uppercase();
        database::set_data(&usr, &process(&usr, pwd).await?).await?;

        let events = database::get_data(&usr)
            .await?
            .iter()
            .map(Data::to_ics_event)
            .collect::<Vec<Event>>();
        let mut cal = Calendar::new();
        cal.name("TKBSV");
        cal.extend(events);

        let mut res = Response::new(StatusCode::Accepted);
        res.set_content_type(Mime::from_str("text/calendar").unwrap());
        res.set_body(format!("{}", cal).into_bytes());
        Ok(res)
    });
    app.at("/json/*").get(|req: Request<()>| async move {
        let path = req.url().path().replace("/json/", "");
        let (usr, pwd) = path
            .split_once('/')
            .ok_or_else(|| Error::new(400, anyhow!("Example CT010101_Passwd")))?;

        let usr = usr.to_uppercase();
        database::set_data(&usr, &process(&usr, pwd).await?).await?;

        let doc = database::get_data(&usr)
            .await?
            .iter()
            .map(|dat| dat.to_json_map())
            .collect::<Vec<HashMap<&'static str, String>>>();

        let mut res = Response::new(StatusCode::Accepted);
        res.set_content_type(Mime::from_str("application/json")?);
        res.set_body(tide::Body::from_json(&doc)?);
        Ok(res)
    });

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    Ok(app.listen(format!("0.0.0.0:{}", port)).await?)
}
