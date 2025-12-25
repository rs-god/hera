use chrono::prelude::*;
use futures::TryStreamExt;
use sqlx::Row;
use xmysql::MysqlConf;

#[derive(Debug, sqlx::FromRow)]
struct Stu {
    id: i64,
    name: String,
    age: i32,
    id_card: String,
    last_update: NaiveDate, // 时间格式
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let dsn = "mysql://root:root123456@localhost/test";
    let mysql_conf = MysqlConf::new(dsn).with_max_connections(10);
    let pool = mysql_conf.init_pool().await?;
    let row: (i64,) = sqlx::query_as("select ?")
        .bind(120i64)
        .fetch_one(&pool)
        .await?;

    println!("res: {}", row.0);
    assert_eq!(row.0, 120);

    // 使用fetch，获取cursor游标，自己处理row
    let sql = "select * from student where id >= ?";
    let mut rows = sqlx::query(sql).bind(1).fetch(&pool);
    while let Some(row) = rows.try_next().await? {
        let stu = Stu {
            id: row.get("id"),
            name: row.get("name"),
            age: row.get("age"),
            id_card: row.get("id_card"),
            last_update: row.get("last_update"),
        };

        // println!("stu = {:?}", stu);
        println!(
            "id: {},name: {} age: {} id_card: {} last_update: {}",
            stu.id, stu.name, stu.age, stu.id_card, stu.last_update
        );
    }

    Ok(())
}
