// 引入了chrono::Utc模块
use chrono::Utc;
use env_logger::Target;
use std::io::Write;

pub struct Logger {
    caller_line: bool, // 日志初始化是否自定义
}

impl Logger {
    pub fn new() -> Self {
        Self { caller_line: false }
    }

    // 如果开启了 caller_line 模式，输出结果如下
    // [2025-11-09T01:19:41Z INFO logger::tests:77] info message
    // [2025-11-09T01:19:41Z ERROR logger::tests:78] error message:invalid key
    pub fn with_caller_line(mut self) -> Self {
        self.caller_line = true;
        self
    }

    // 日志初始化
    // 其中日志level优先级从高到低：error > warn > info > debug > trace
    // 程序启动时可以通过 RUST_LOG=info 设置日志级别
    pub fn init(&self) {
        if !self.caller_line {
            // 如果你不关注日志时区的话，可以直接使用eng_logger默认方式初始化
            env_logger::builder().target(Target::Stdout).init();
            return;
        }

        // env_logger env settings
        env_logger::builder()
            .target(Target::Stdout)
            .format(|buf, record| {
                let level = record.level();
                writeln!(
                    buf,
                    "[{} {} {}:{}] {}",
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ"), // 时间格式
                    level,                                   // 日志级别
                    record.module_path().unwrap_or("unnamed"), // 模块名
                    record.line().unwrap_or(0),              // 行号
                    &record.args()                           // 日志message body内容
                )
            })
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{error, info};
    use std::env;
    #[test]
    fn test_logger() {
        // 通过环境变量 RUST_LOG 设置日志级别
        // 一般来说，程序启动时，可以先设置 RUST_LOG=info 而不需要通过unsafe方式来设置
        // 如果开启了caller_line模式，输出结果如下
        // [2025-11-09T01:19:41Z INFO logger::tests:77] info message
        // [2025-11-09T01:19:41Z ERROR logger::tests:78] error message:invalid key
        //
        // 否则就不包含函数行号等信息
        // [2025-11-09T01:19:41Z INFO  logger::tests] info message
        // [2025-11-09T01:19:41Z ERROR logger::tests] error message:invalid key
        unsafe {
            env::set_var("RUST_LOG", "info");
        }

        let logger = Logger::new().with_caller_line();
        // let logger = Logger::new();
        logger.init();

        info!("info message");
        error!("error message:{}", "invalid key");
    }
}
