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

pub async fn get_date(student_code: &str) -> Result<Vec<Data>, sqlx::Error> {
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
        serde_json::from_value::<HashMap<String, Data>>(data_map)
            .unwrap()
            .iter()
            .map(|(_k, v)| v.clone())
            .collect()
    })
}

pub async fn set_data(student_code: &str, data: &Vec<Data>) -> Result<(), sqlx::Error> {
    if data.is_empty() {
        Ok(())
    } else {
        let data = data
            .iter()
            .map(|d| {
                (
                    format!("{}-{}", d.time_begin, d.time_end),
                    to_value(d).unwrap(),
                )
            })
            .collect::<Map<String, Value>>();
        query!(
            r#"
            insert into student_schedule(student_code, schedule_data)
            values($1, $2)
            on conflict(student_code) do update
            set schedule_data = student_schedule.schedule_data || $2
            "#,
            student_code,
            Value::Object(data),
        )
        .execute(&*DB)
        .await?;
        Ok(())
    }
}
