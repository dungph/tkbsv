use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::{to_value, Map, Value};
use sqlx::query;

use crate::Data;

static DB: Lazy<sqlx::PgPool> = Lazy::new(|| {
    let url = std::env::var("DATABASE_URL").expect("set DATABASE_URL to your postgres uri");
    sqlx::PgPool::connect_lazy(&url).unwrap()
});

pub async fn migrate() -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(&*DB).await?;
    Ok(())
}

pub async fn get_data(student_code: &str) -> Result<Vec<Data>, sqlx::Error> {
    query!(
        r#"
        -- GET ENTITY'S FIELDS
        select schedule_data from student_schedule
        where student_code = $1
        "#,
        student_code
    )
    .fetch_one(&*DB)
    .await
    .map(|record| record.schedule_data)
    .map(|data_map| {
        serde_json::from_value::<HashMap<String, Vec<Data>>>(data_map)
            .unwrap()
            .iter()
            .flat_map(|(_k, v)| v.clone())
            .collect()
    })
}

pub async fn set_data(student_code: &str, data: &Vec<Data>) -> Result<(), sqlx::Error> {
    if data.is_empty() {
        Ok(())
    } else {
        let key = data
            .iter()
            .map(|dat| &dat.class)
            .filter_map(|class| class.rsplit_once('('))
            .filter_map(|(n, _)| n.trim().rsplit_once('-'))
            .find_map(|(s, year)| s.trim_end().rsplit_once('-').map(|r| (r.1, year)))
            .unwrap();
        let key = format!("{}-{}", key.0, key.1);

        let mut map = Map::new();
        map.insert(key, to_value(data).unwrap());

        query!(
            r#"
            insert into student_schedule(student_code, schedule_data)
            values($1, $2)
            on conflict(student_code) do update
            set schedule_data = student_schedule.schedule_data || $2
            "#,
            student_code,
            Value::Object(map),
        )
        .execute(&*DB)
        .await?;
        Ok(())
    }
}
