# hera

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tag](https://img.shields.io/badge/tag-v1.2.4-blue.svg)](https://github.com/rs-god/hera/releases/tag/v1.2.4)

Rust 基础设施组件库，采用 Workspace 多 Crate 架构，提供配置管理、加解密、日志、监控、平滑退出以及 MySQL/Redis/Pulsar 等中间件封装。

## 版本信息

| 项目 | 值 |
|---|---|
| 版本 | v1.2.4 |
| 仓库 | <https://github.com/rs-god/hera> |
| 协议 | MIT |
| 作者 | daheige |

## 特性概览

| 组件 | 定位 | 核心能力 |
|---|---|---|
| `config` | 配置读取 | YAML 配置文件加载与反序列化 |
| `crypto` | 加解密 | AES-128/192/256 CBC 模式加密，Base64 输出 |
| `logger` | 日志 | 普通文本 / JSON 输出，支持 `log` key/value，可自定义时间格式 |
| `monitor` | 监控 | 基于 `autometrics` 自动采集函数级 Prometheus 指标 |
| `shutdown` | 平滑退出 | 监听系统信号，支持异步优雅关闭 |
| `xmysql` | MySQL | 基于 `sqlx` 的异步连接池 |
| `xredis` | Redis | 单节点 / 集群连接池，支持同步与异步 |
| `xpulsar` | Pulsar | 异步 Producer/Consumer 封装，支持 Token 认证 |

---

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
crypto = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
logger = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
monitor = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
shutdown = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
config = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
xmysql = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
xredis = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
xpulsar = { git = "https://github.com/rs-god/hera.git", tag = "v1.2.4" }
```

> 如需使用 `logger` 的 key/value 语法，请同时依赖 `log = { version = "0.4", features = ["kv"] }`。

---

## 环境要求

- **Rust**: 1.85 或更高版本
- **Edition**: 2024
- 部分组件需要额外运行环境：
  - `xmysql`：MySQL 服务
  - `xredis`：Redis 单节点或集群
  - `xpulsar`：Pulsar 服务
  - `monitor`：可选 Prometheus 拉取端点

---

## 快速开始

下面以 `logger` 为例，演示如何在业务项目中使用 hera 组件：

```rust
use log::info;
use logger::Logger;

fn main() {
    Logger::new()
        .with_caller_line()
        .with_json()
        .init();

    info!(a = 1, b = "xxx"; "hello,world");
}
```

运行：

```bash
RUST_LOG=info cargo run
```

输出：

```json
{"ts":"2026-07-18T14:25:16Z","level":"INFO","module":"my_app::main","caller":7,"msg":"hello,world","a":1,"b":"xxx"}
```

---

## config — YAML 配置读取

**功能点**

- 读取 YAML 配置文件内容
- 支持反序列化为 `serde_yaml::Value` 或自定义结构体
- 基于 `ConfigTrait` trait 抽象

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

- 基于 `env_logger` 与 crates.io `log` crate 封装
- 支持标准输出（Stdout）
- 普通文本格式与 JSON 结构化格式可选
- 完整支持 `log` 原生 key/value 语法：`info!(key = value; "msg")`
- 可选 `caller_line` 模式：日志中携带模块路径与代码行号
- 可选 `with_json()` 模式：输出 JSON 格式日志
- 可选 `with_time_format()` 自定义时间格式，默认 `%Y-%m-%dT%H:%M:%SZ`
- 自定义 `target:` 标签可自动输出到 JSON
- 日志级别通过环境变量 `RUST_LOG` 控制，优先级：`error > warn > info > debug > trace`

**使用示例**

```rust
use log::info;
use logger::Logger;

// JSON 格式，携带 caller 行号
Logger::new()
    .with_caller_line()
    .with_json()
    .init();

info!(a = 1, b = "xxx"; "hello,world");
```

输出：

```json
{"ts":"2026-07-18T14:25:16Z","level":"INFO","module":"my_app::main","caller":7,"msg":"hello,world","a":1,"b":"xxx"}
```

自定义时间格式：

```rust
Logger::new()
    .with_json()
    .with_time_format("%Y-%m-%d %H:%M:%S")
    .init();
```

普通文本格式（带行号）：

```rust
Logger::new().with_caller_line().init();
info!("hello,world");
// [2026-07-18T14:25:16Z INFO my_app::main:7] hello,world
```

配合环境变量使用：

```bash
RUST_LOG=info cargo run
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

## 各 Crate 详细文档

每个 crate 的源码目录下都有独立的 `readme.md`，包含更详细的 API 说明与示例：

- [config](crates/config/readme.md) — YAML 配置读取
- [crypto](crates/crypto/readme.md) — AES 加解密
- [logger](crates/logger/readme.md) — 日志初始化（JSON / 普通文本）
- [monitor](crates/monitor/readme.md) — Prometheus 监控指标
- [shutdown](crates/shutdown/readme.md) — 平滑退出
- [xmysql](crates/xmysql/readme.md) — MySQL 连接池
- [xpulsar](crates/xpulsar/readme.md) — Pulsar 消息队列
- [xredis](crates/xredis/readme.md) — Redis 客户端/集群

---

## 许可证

本项目采用 [MIT](LICENSE) 协议开源。
