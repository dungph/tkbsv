use chrono::{DateTime, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use md5::compute;
use scraper::{Html, Selector};
use std::collections::HashMap;
use surf::Body;
use tide::Error;

use crate::Data;

pub async fn process(usr: &str, pwd: &str) -> Result<Vec<Data>, Error> {
    let vec = parse_html(get_html(usr, pwd).await?)?;
    Ok(vec
        .iter()
        .flat_map(|(cl, ts, ps)| parse_table_row(cl, ts, ps))
        .collect())
}

fn parse_table_row(class: &str, times: &str, places: &str) -> Vec<Data> {
    let mut default = String::new();
    let mut map = HashMap::new();
    //  parse this to map
    //  (1,2,3)
    //  201-TA3-CNTT TA3
    //  (4,5)
    //  201-TA4-CNTT TA4
    //
    //  or this to default
    //  201-TA4-CNTT TA4
    if places.contains('(') {
        places
            .split('(')
            .skip(1)
            .map(|s| s.split_once(')').unwrap())
            .map(|(i, p)| (i.trim(), p.trim()))
            .map(|(i, p)| (parse_list_uint(i), p))
            .for_each(|(vec, p)| {
                vec.into_iter().for_each(|i| {
                    map.insert(i as usize, p.to_string());
                })
            });
    } else {
        default = places.to_string();
    }

    // parse this
    // Từ 30/05/2022 đến 05/06/2022: (1)
    //    Thứ 4 tiết 9,10,11,12 (TH)
    // Từ 06/06/2022 đến 12/06/2022: (2)
    //    Thứ 4 tiết 9,10,11,12 (TH)
    // Từ 13/06/2022 đến 19/06/2022: (3)
    //    Thứ 4 tiết 9,10,11,12 (TH)
    //    Thứ 6 tiết 9,10,11,12 (TH)
    // Từ 20/06/2022 đến 26/06/2022: (4)
    //    Thứ 4 tiết 9,10,11,12 (TH)
    times
        .split("Từ ")
        .skip(1)
        .map(|s| s.split_once(':').unwrap())
        .map(|(r, o)| (r, dbg!(o.split_once("Thứ").unwrap().1)))
        .map(|(r, o)| (r.trim(), o.trim()))
        .map(|(r, o)| (parse_date_range(r), parse_weekday_and_period(o)))
        .map(|(range, times)| {
            times
                .iter()
                .flat_map(|(wd, (bt, et))| {
                    let mut vec = Vec::new();
                    let mut date = range.0 + Duration::days(wd.num_days_from_monday() as i64);
                    while date < range.1 {
                        vec.push((date.and_time(*bt), date.and_time(*et)));
                        date += Duration::days(7);
                    }
                    vec
                })
                .collect::<Vec<(NaiveDateTime, NaiveDateTime)>>()
        })
        .enumerate()
        .flat_map(|(i, vec)| {
            vec.iter()
                .map(|(b, e)| Data {
                    class: class.to_string(),
                    time_begin: to_utc(*b),
                    time_end: to_utc(*e),
                    place: map.get(&(i + 1)).unwrap_or(&default).clone(),
                })
                .collect::<Vec<Data>>()
        })
        .collect()
}
fn parse_weekday_and_period(all: &str) -> Vec<(Weekday, (NaiveTime, NaiveTime))> {
    fn get_ps_str_time(ps: &str) -> (NaiveTime, NaiveTime) {
        let vec = parse_list_uint(ps);
        let b = vec[0];
        let e = vec.last().unwrap();
        (get_period_time(b).0, get_period_time(*e).1)
    }

    all.trim()
        .split("Thứ")
        .map(|s| s.split_once("tiết").unwrap())
        .map(|(wd, ps)| (wd.trim(), ps.trim()))
        .map(|(wd, ps)| (parse_weekday(wd), get_ps_str_time(ps)))
        .collect()
}

fn parse_date_range(range: &str) -> (NaiveDate, NaiveDate) {
    range
        .split_once("đến")
        .map(|(f, t)| (f.trim(), t.trim()))
        .map(|(f, t)| {
            (
                NaiveDate::parse_from_str(f, "%d/%m/%Y").unwrap(),
                NaiveDate::parse_from_str(t, "%d/%m/%Y").unwrap(),
            )
        })
        .unwrap()
}

pub fn parse_list_uint(s: &str) -> Vec<u32> {
    s.trim_matches(|c: char| !c.is_ascii_digit())
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

pub fn parse_weekday(s: &str) -> Weekday {
    let d = s
        .trim_matches(|c: char| !c.is_ascii_digit())
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

//fn parse_date(s: &str) -> NaiveDate {
//    NaiveDate::parse_from_str(s, "%d/%m/%Y").unwrap()
//}

/// return list of subject (each row in the table)
fn parse_html(doc: String) -> Result<Vec<(String, String, String)>, Error> {
    fn fmt_error<T>(_: T) -> Error {
        Error::new(502, anyhow::anyhow!("resources page has change format"))
    }
    let all = Html::parse_document(&doc);
    let select = Selector::parse(r#"tr[class="cssListItem"]"#).map_err(fmt_error)?;
    let select_alt = Selector::parse(r#"tr[class="cssListAlternativeItem"]"#).map_err(fmt_error)?;
    let body = all.select(&select).chain(all.select(&select_alt));
    let each_col = Selector::parse("td").map_err(fmt_error)?;

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
async fn get_html(username: &str, passwd: &str) -> Result<String, Error> {
    const DATA_URL: &str =
        "http://qldt.actvn.edu.vn/CMCSoft.IU.Web.Info/Reports/Form/StudentTimeTable.aspx";
    const LOGIN_URL: &str = "http://qldt.actvn.edu.vn/CMCSoft.IU.Web.info/Login.aspx";

    let mut login_page = surf::get(LOGIN_URL).await?;
    let cookie = login_page.header("set-cookie").unwrap().to_string();
    let cookie = cookie.split(';').next().unwrap().to_string();
    let cookie = cookie.replace("[\"", "");

    fn get_state(body: &str) -> (String, String) {
        let view = body
            .lines()
            .filter(|s| s.contains("VIEWSTATE"))
            .map(|s| s.split("value=\"").last().unwrap())
            .map(|s| s.split("\" />").next().unwrap())
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        (view[0].clone(), view[1].clone())
    }
    let view = get_state(&login_page.body_string().await.unwrap());

    let passwd = format!("{:x}", compute(passwd));
    let mut form = HashMap::new();
    form.insert("txtUserName", username.to_string());
    form.insert("txtPassword", passwd);
    form.insert("btnSubmit", "Đăng nhập".to_string());
    form.insert("__EVENTTARGET", String::new());
    form.insert("__EVENTARGUMENT", String::new());
    form.insert("__LASTFOCUS", String::new());
    form.insert("__VIEWSTATE", view.0);
    form.insert("__VIEWSTATEGENERATOR", view.1);
    form.insert(
        "PageHeader1$drpNgonNgu",
        "E43296C6F24C4410A894F46D57D2D3AB".to_string(),
    );
    form.insert("PageHeader1$hidisNotify", "0".to_string());
    form.insert("PageHeader1$hidValueNotify", ".".to_string());
    form.insert("hidUserId", String::new());
    form.insert("hidUserFullName", String::new());
    form.insert("hidTrainingSystemId", String::new());

    let login = surf::post(LOGIN_URL)
        .header("Cookie", &*cookie)
        .body(Body::from_form(&form)?)
        .await?;

    let cookie1 = login
        .header("set-cookie")
        .ok_or_else(|| Error::new(502, anyhow::anyhow!("expected cookie")))?
        .to_string();
    let cookie1 = cookie1
        .split(';')
        .next()
        .ok_or_else(|| Error::new(502, anyhow::anyhow!("mismatch cookie format")))?
        .to_string();
    let cookie1 = cookie1.replace("[\"", "");

    let doc = surf::get(DATA_URL)
        .header("Cookie", format!("{}; {}", cookie, cookie1))
        .await?
        .body_string()
        .await?;

    Ok(doc)
}
