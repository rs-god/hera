// 引入了chrono::Utc模块
use chrono::Utc;
use env_logger::Target;
use std::io::Write;

// 直接使用 crates.io 的 log crate 宏，业务项目可 `use log::info;` 或 `use logger::info;`
pub use log::{debug, error, info, warn};

pub struct Logger {
    caller_line: bool,   // 是否输出行号
    enable_json: bool,   // 是否以 JSON 格式输出
    time_format: String, // 时间格式化格式
}

impl Logger {
    pub fn new() -> Self {
        Self {
            caller_line: false,
            enable_json: false,
            time_format: "%Y-%m-%dT%H:%M:%SZ".to_string(),
        }
    }

    /// 开启 caller_line 模式，输出中携带调用行号。
    ///
    /// 普通格式示例：
    /// ```text
    /// [2025-11-09T01:19:41Z INFO logger::tests:77] info message
    /// ```
    pub fn with_caller_line(mut self) -> Self {
        self.caller_line = true;
        self
    }

    /// 开启 JSON 格式输出。
    ///
    /// 输出示例：
    /// ```json
    /// {"ts":"2026-07-15T14:25:16Z","level":"INFO","module":"logger::tests","caller":131,"msg":"hello,world","a":1,"b":"xxx"}
    /// ```
    pub fn with_json(mut self) -> Self {
        self.enable_json = true;
        self
    }

    /// 自定义时间格式，默认 `%Y-%m-%dT%H:%M:%SZ`。
    ///
    /// 例如 `%Y-%m-%d %H:%M:%S` 会输出 `2026-07-15 14:25:16`。
    pub fn with_time_format(mut self, format: &str) -> Self {
        self.time_format = format.to_string();
        self
    }

    /// 日志初始化。
    ///
    /// 日志 level 优先级从高到低：error > warn > info > debug > trace。
    /// 程序启动时可以通过 `RUST_LOG=info` 设置日志级别。
    ///
    /// 输出格式由 `with_caller_line()` 和 `with_json()` 共同决定：
    /// - 默认：env_logger 默认格式；
    /// - `with_caller_line()`：带行号的普通格式；
    /// - `with_json()`：JSON 格式（可再叠加 `with_caller_line()` 携带 caller 字段）。
    pub fn init(&self) {
        self.try_init().expect("Logger::init should not be called after logger initialized");
    }

    /// 尝试初始化日志，返回错误而不是 panic。
    pub fn try_init(&self) -> Result<(), log::SetLoggerError> {
        let mut builder = env_logger::Builder::from_default_env();
        builder.target(Target::Stdout);

        let time_format = self.time_format.clone();

        if self.enable_json {
            builder.format(move |buf, record| {
                let json = format_json(record, &time_format);
                writeln!(buf, "{}", json)
            });
        } else if self.caller_line {
            builder.format(move |buf, record| {
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    Utc::now().format(&time_format), // 时间格式
                    record.level(),                  // 日志级别
                    record.module_path().unwrap_or("unnamed"), // 模块名
                    record.line().unwrap_or(0),      // 行号
                    record.args()                    // 日志 message body
                )
            });
        }

        builder.try_init()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// 将一条 `log::Record` 格式化为 JSON 字符串。
fn format_json(record: &log::Record, time_format: &str) -> String {
    let mut visitor = JsonVisitor::new();
    // 忽略 key_values 访问错误，保证日志始终能输出
    let _ = record.key_values().visit(&mut visitor);

    let ts = Utc::now().format(time_format).to_string();
    let level = record.level();
    let module = record.module_path().unwrap_or("unnamed");
    let target = record.target();
    let caller = record.line().unwrap_or(0);
    let msg = record.args().to_string();

    // 为了保持字段顺序与示例一致，手动拼接 JSON。
    // 基础字段固定顺序：ts, level, module, [target], caller, msg
    let mut parts = Vec::with_capacity(6 + visitor.map.len());
    parts.push(format!("\"ts\":{}", json_str(&ts)));
    parts.push(format!("\"level\":{}", json_str(&level.to_string())));
    parts.push(format!("\"module\":{}", json_str(module)));

    // 如果用户通过 target: "xxx" 自定义了 target，且与 module 不同，则输出 target 字段
    if target != module {
        parts.push(format!("\"target\":{}", json_str(target)));
    }

    parts.push(format!("\"caller\":{}", caller));
    parts.push(format!("\"msg\":{}", json_str(&msg)));

    // 追加用户传入的 key/value 字段
    for (k, v) in visitor.map {
        parts.push(format!("{}:{}", json_str(&k), v));
    }

    format!("{{{}}}", parts.join(","))
}

/// 将字符串序列化为 JSON 字符串（带引号并转义）。
fn json_str(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// 访问 `log::kv::Source`，收集所有 key/value 到 serde_json::Map。
struct JsonVisitor {
    map: serde_json::Map<String, serde_json::Value>,
}

impl JsonVisitor {
    fn new() -> Self {
        Self {
            map: serde_json::Map::new(),
        }
    }
}

impl<'kvs> log::kv::VisitSource<'kvs> for JsonVisitor {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        let mut value_visitor = JsonValueVisitor::new();
        value.visit(&mut value_visitor)?;
        if let Some(v) = value_visitor.value {
            self.map.insert(key.to_string(), v);
        }
        Ok(())
    }
}

/// 访问单个 `log::kv::Value`，将其转换为 `serde_json::Value`。
struct JsonValueVisitor {
    value: Option<serde_json::Value>,
}

impl JsonValueVisitor {
    fn new() -> Self {
        Self { value: None }
    }
}

impl<'v> log::kv::VisitValue<'v> for JsonValueVisitor {
    fn visit_any(&mut self, value: log::kv::Value) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::String(value.to_string()));
        Ok(())
    }

    fn visit_i64(&mut self, value: i64) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::Number(value.into()));
        Ok(())
    }

    fn visit_u64(&mut self, value: u64) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::Number(value.into()));
        Ok(())
    }

    fn visit_i128(&mut self, value: i128) -> Result<(), log::kv::Error> {
        self.value = Some(json_number_from_i128(value));
        Ok(())
    }

    fn visit_u128(&mut self, value: u128) -> Result<(), log::kv::Error> {
        self.value = Some(json_number_from_u128(value));
        Ok(())
    }

    fn visit_f64(&mut self, value: f64) -> Result<(), log::kv::Error> {
        self.value = Some(json_number_from_f64(value));
        Ok(())
    }

    fn visit_bool(&mut self, value: bool) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::Bool(value));
        Ok(())
    }

    fn visit_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::String(value.to_string()));
        Ok(())
    }

    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::String(value.to_string()));
        Ok(())
    }

    fn visit_char(&mut self, value: char) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::String(value.to_string()));
        Ok(())
    }

    fn visit_null(&mut self) -> Result<(), log::kv::Error> {
        self.value = Some(serde_json::Value::Null);
        Ok(())
    }
}

fn json_number_from_i128(value: i128) -> serde_json::Value {
    if let Ok(n) = i64::try_from(value) {
        serde_json::Value::Number(n.into())
    } else {
        serde_json::Number::from_f64(value as f64)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
    }
}

fn json_number_from_u128(value: u128) -> serde_json::Value {
    if let Ok(n) = u64::try_from(value) {
        serde_json::Value::Number(n.into())
    } else {
        serde_json::Number::from_f64(value as f64)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
    }
}

fn json_number_from_f64(value: f64) -> serde_json::Value {
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_json() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }

        let logger = Logger::new().with_caller_line().with_json();
        let _ = logger.try_init();

        // 普通消息
        info!("hello,world");

        // 标准 log key/value 语法（key/value 在前，分号分隔）
        info!(a = 1, b = "xxx"; "hello,world");

        // 带格式化参数的消息
        info!("hello,{}", "world");

        // 格式化参数 + key/value
        info!(a = 1, b = "xxx"; "hello,{}", "world");

        // 更多类型
        info!(user_id = 42i64, score = 98.5, is_vip = true; "user login");

        // 带 target + key/value
        info!(target: "my_target", app = "demo"; "target message");

        // 带 target 普通 format
        info!(target: "my_target", "plain target message");
    }

    #[test]
    fn test_custom_time_format() {
        let record = log::Record::builder()
            .level(log::Level::Info)
            .module_path(Some("logger::tests"))
            .target("logger::tests")
            .file(Some("crates/logger/src/lib.rs"))
            .line(Some(100))
            .args(format_args!("hello,world"))
            .build();

        // 默认 ISO 8601 格式
        let json = format_json(&record, "%Y-%m-%dT%H:%M:%SZ");
        assert!(json.contains("\"ts\":\"20") && json.contains("T") && json.contains("Z\""));

        // 自定义格式：%Y-%m-%d %H:%M:%S（带空格，无 T 无 Z）
        let json = format_json(&record, "%Y-%m-%d %H:%M:%S");
        assert!(json.contains("\"ts\":\"20") && json.contains(" ") && !json.contains("T"));
    }
}
