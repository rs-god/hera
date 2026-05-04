# hera 组件库使用指南
Rust infrastructure component library

`hera` 是一个 Rust 基础设施组件库，采用 Workspace 多 Crate 架构，提供配置管理、加解密、日志、监控、平滑退出以及 MySQL/Redis/Pulsar 等中间件封装。

---

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.2.0 |
| 仓库 | <https://github.com/rs-god/hera> |
| 协议 | MIT |
| 作者 | daheige |

## 目录

1. [引入方式](#引入方式)
2. [config — YAML 配置读取](#config--yaml-配置读取)
3. [crypto — AES 加解密](#crypto--aes-加解密)
4. [logger — 日志初始化](#logger--日志初始化)
5. [monitor — Prometheus 监控指标](#monitor--prometheus-监控指标)
6. [shutdown — 平滑退出](#shutdown--平滑退出)
7. [xmysql — MySQL 连接池](#xmysql--mysql-连接池)
8. [xpulsar — Pulsar 消息队列](#xpulsar--pulsar-消息队列)
9. [xredis — Redis 客户端/集群](#xredis--redis-客户端集群)

---

## 引入方式

在 `Cargo.toml` 中通过 git + tag 引入指定 crate：

```toml
[dependencies]
crypto = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
logger = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
monitor = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
shutdown = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
config = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
xmysql = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
xredis = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
xpulsar = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.0" }
```

---

## config — YAML 配置读取

**功能点**
- 读取 YAML 配置文件内容
- 支持反序列化为 `serde_yaml::Value` 或自定义结构体
- 基于 `ConfigTrait`  trait 抽象

**使用示例**

```rust
use config::{Config, ConfigTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppConfig {
    app_debug: bool,
    app_name: String,
    app_port: i32,
}

let mut c = Config::new("config/app.yaml");
c.load().expect("read file failed");

// 方式1：读取为自定义结构体
let cfg: AppConfig = serde_yaml::from_str(c.content()).unwrap();

// 方式2：读取为 serde_yaml::Value
let val = c.sections();
let cfg: AppConfig = serde_yaml::from_value(val).unwrap();
```

---

## crypto — AES 加解密

**功能点**
- 支持 AES-128、AES-192、AES-256 三种密钥长度
- CBC 模式 + PKCS7 填充
- 加密结果采用 Base64 编码
- 内置随机 key/iv 生成方法（16 进制字符串）

**核心类型**
- `Aes128Crypto` — 16 字节密钥
- `Aes192Crypto` — 24 字节密钥
- `Aes256Crypto` — 32 字节密钥
- `AesCrypto<T>` — 泛型底层结构

**使用示例**

```rust
use crypto::Aes256Crypto;

let key = Aes256Crypto::generate_key(); // 32 位 16 进制字符串
let iv = Aes256Crypto::generate_iv();   // 16 位 16 进制字符串
let c = Aes256Crypto::new(&key, &iv);

let s = "hello world";
let encrypted = c.encrypt(s).unwrap();   // Base64 密文
let decrypted = c.decrypt(&encrypted).unwrap();
assert_eq!(s, decrypted);
```

---

## logger — 日志初始化

**功能点**
- 基于 `env_logger` 封装
- 支持标准输出（Stdout）
- 可选 `caller_line` 模式：日志中携带模块路径与代码行号
- 日志级别通过环境变量 `RUST_LOG` 控制，优先级：`error > warn > info > debug > trace`

**使用示例**

```rust
use logger::Logger;

// 标准模式
Logger::new().init();

// 带行号模式，输出示例：
// [2025-11-09T01:19:41Z INFO logger::tests:77] info message
Logger::new().with_caller_line().init();

// 配合环境变量使用
// RUST_LOG=info cargo run
```

---

## monitor — Prometheus 监控指标

**功能点**
- 基于 `autometrics` 自动采集函数级指标（调用次数、延迟、成功率）
- 内置 SLO（Service Level Objective）定义：成功率 P99.9、延迟 P99 < 1000ms
- 提供 `/metrics` Prometheus 拉取端点
- 提供 `/check` 健康检查端点
- 集成 `axum` HTTP 服务与平滑退出

**核心 API**
- `prometheus_init(port)` — 启动独立 metrics HTTP 服务
- `API_SLO` — 预定义 SLO 常量，配合 `#[autometrics(objective = API_SLO)]` 使用

**使用示例**

```rust
use monitor::metrics::{prometheus_init, API_SLO};
use autometrics::autometrics;

#[tokio::main]
async fn main() {
    // 启动 metrics 服务，监听 8090 端口
    prometheus_init(8090).await;
}

#[autometrics(objective = API_SLO)]
pub async fn home() -> &'static str {
    "Hello, home!"
}
```

完整用法参考 `crates/monitor/examples/metrics_basic.rs`。

---

## shutdown — 平滑退出

**功能点**
- 监听系统退出信号（Ctrl+C / SIGTERM）
- 信号触发后等待指定时长再退出，便于执行清理逻辑
- 跨平台兼容（Unix/Windows）

**使用示例**

```rust
use shutdown::graceful_shutdown;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // 启动你的服务...

    // 等待退出信号，收到后等待 5 秒再退出
    graceful_shutdown(Duration::from_secs(5)).await;
}
```

与 `axum` 集成：

```rust
axum::serve(listener, router)
    .with_graceful_shutdown(graceful_shutdown(Duration::from_secs(5)))
    .await
    .unwrap();
```

---

## xmysql — MySQL 连接池

**功能点**
- 基于 `sqlx` 的异步 MySQL 连接池
- 可配置最大/最小连接数、最大生命周期、空闲超时、连接超时
- 返回 `sqlx::MySqlPool`，可直接用于 `sqlx` 查询 API

**配置项（默认值）**
- `max_connections`: 100
- `min_connections`: 10
- `max_lifetime`: 1800s
- `idle_timeout`: 600s
- `connect_timeout`: 10s

**使用示例**

```rust
use xmysql::MysqlConf;

let dsn = "mysql://root:root123456@localhost/test";
let pool = MysqlConf::new(dsn)
    .with_max_connections(10)
    .init_pool()
    .await
    .unwrap();

// 使用 sqlx API 查询
let row: (i64,) = sqlx::query_as("select ?")
    .bind(120i64)
    .fetch_one(&pool)
    .await?;
```

---

## xpulsar — Pulsar 消息队列

**功能点**
- 基于 `pulsar` crate 的异步客户端封装
- 支持 Token 认证
- 提供 Producer / Consumer Builder 快捷创建
- 内置 `Message` 结构体，基于 `serde_json` 序列化/反序列化

**使用示例**

```rust
use xpulsar::{PulsarConf, Message};
use pulsar::{producer, proto};

let conf = PulsarConf::new("pulsar://127.0.0.1:6650")
    .with_token("your-token"); // 可选

let builder = conf.pulsar_builder();
let pulsar_obj = conf.pulsar_obj(builder).await.unwrap();

// 创建生产者
let mut producer = pulsar_obj
    .producer()
    .with_topic("my-topic")
    .with_name("my_producer")
    .build()
    .await?;

producer.send_non_blocking(Message { data: "hello".into() }).await?;

// 创建消费者
let mut consumer = pulsar_obj
    .consumer()
    .with_topic("my-topic")
    .with_consumer_name("group-1")
    .with_subscription_type(SubType::Exclusive)
    .with_subscription("my_sub")
    .build()
    .await?;
```

---

## xredis — Redis 客户端/集群

**功能点**
- 支持单节点 Redis（`redis::Client`）和 Redis Cluster（`ClusterClient`）
- 基于 `r2d2` 的连接池管理
- 支持同步与异步操作
- Builder 风格配置

**配置项（默认值）**
- `max_size`: 20
- `min_idle`: 3
- `max_lifetime`: 1800s
- `idle_timeout`: 300s
- `connection_timeout`: 10s

**使用示例**

```rust
use xredis::RedisConf;
use redis::Commands;

// 单节点 + 连接池
let dsn = "redis://:@127.0.0.1:6379/0";
let pool = RedisConf::builder()
    .with_dsn(dsn)
    .init_pool();

let mut conn = pool.get().unwrap();
let _: () = conn.set("my_user", "daheige").unwrap();

// 集群 + 连接池
let nodes = vec![
    "redis://:@127.0.0.1:6380/0",
    "redis://:@127.0.0.1:6381/0",
    // ...
];
let pool = RedisConf::builder()
    .with_cluster_nodes(nodes)
    .init_cluster_pool();

// 异步操作
use redis::AsyncCommands;
let client = RedisConf::builder().with_dsn(dsn).client()?;
let mut con = client.get_multiplexed_async_connection().await?;
let _: () = con.set("name", "hello").await?;
let name: String = con.get("name").await?;
```

---
