use redis::Commands;
use redis::RedisResult;
use xredis::RedisConf;

fn main() {
    let dsn = "redis://:@127.0.0.1:6379/0";
    let pool = RedisConf::builder().with_dsn(dsn).init_pool();
    let mut conn = pool.get().unwrap();

    // 设置单个pool timeout
    // let mut conn = pool.get_timeout(Duration::from_secs(2)).unwrap();
    let res: RedisResult<String> = conn.set("my_user", "daheige");
    if res.is_err() {
        println!("redis set error:{}", res.err().unwrap().to_string());
    } else {
        println!("set success");
    }
}

// async exec redis operation
#[tokio::test]
async fn redis_async_test() -> RedisResult<()> {
    // you must use this module for async exec.
    use redis::AsyncCommands;
    let dsn = "redis://:@127.0.0.1:6379/0";
    let client = RedisConf::builder().with_dsn(dsn).client()?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = conn.set("user1", "daheige").await?;

    // async exec set cmd
    let _: () = redis::cmd("set")
        .arg(&["user2", "foo"])
        .query_async(&mut conn)
        .await?;

    let result = redis::cmd("mget")
        .arg(&["user1", "user2"])
        .query_async(&mut conn)
        .await;
    println!("{:?}", result);
    assert_eq!(result, Ok(("daheige".to_string(), "foo".to_string())));
    Ok(())
}

#[test]
fn test_redis_cluster() {
    let nodes = vec![
        "redis://:@127.0.0.1:6380/0",
        "redis://:@127.0.0.1:6381/0",
        "redis://:@127.0.0.1:6382/0",
        "redis://:@127.0.0.1:6383/0",
        "redis://:@127.0.0.1:6384/0",
        "redis://:@127.0.0.1:6385/0",
    ];

    let pool = RedisConf::builder()
        .with_cluster_nodes(nodes)
        .init_cluster_pool();
    let mut conn = pool.get().unwrap();

    let res: RedisResult<String> = conn.set("my_user", "daheige");
    if res.is_err() {
        println!("redis set error:{}", res.err().unwrap().to_string());
    } else {
        println!("set success");
    }

    let res: RedisResult<String> = conn.get("my_user");
    println!("res: {:?}", res);
}
