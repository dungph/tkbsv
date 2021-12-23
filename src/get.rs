use md5::compute;
use std::collections::HashMap;
use std::error::Error;
use surf::Body;

const URL: &str = "http://qldt.actvn.edu.vn/CMCSoft.IU.Web.Info/Reports/Form/StudentTimeTable.aspx";

pub async fn get_html(
    username: &str,
    passwd: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut login_page =
        surf::get("http://qldt.actvn.edu.vn/CMCSoft.IU.Web.info/Login.aspx").await?;
    let cookie = login_page.header("set-cookie").unwrap().to_string();
    let cookie = cookie.split(';').next().unwrap().to_string();
    let cookie = cookie.replace("[\"", "");

    fn get_state(body: &str) -> (String, String) {
        let view = body
            .lines()
            .filter(|s| s.find("VIEWSTATE").is_some())
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

    let login = surf::post("http://qldt.actvn.edu.vn/CMCSoft.IU.Web.info/Login.aspx")
        .header("Cookie", &*cookie)
        .body(Body::from_form(&form)?)
        .await?;

    let cookie1 = login.header("set-cookie").ok_or("Error")?.to_string();
    let cookie1 = cookie1.split(';').next().ok_or("Error")?.to_string();
    let cookie1 = cookie1.replace("[\"", "");

    let doc = surf::get(URL)
        .header("Cookie", format!("{}; {}", cookie, cookie1))
        .await?
        .body_string()
        .await?;

    Ok(doc)
}
