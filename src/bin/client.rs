//! 命令行客户端入口。

use rkv::config::ClientConfig;

fn main() {
    let cfg = match ClientConfig::from_args(std::env::args().skip(1)) {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    };
    println!("[rkv-client] 目标服务器: {}", cfg.addr);
    println!("[rkv-client] 交互功能将在阶段五接入");
}
