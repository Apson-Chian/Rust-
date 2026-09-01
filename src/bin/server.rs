//! 服务器程序入口。

use rkv::config::ServerConfig;

fn main() {
    let cfg = match ServerConfig::from_args(std::env::args().skip(1)) {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    };
    // 数据文件损坏等启动期错误直接退出，避免带着不完整数据对外服务
    if let Err(e) = rkv::server::run(&cfg) {
        eprintln!("[rkv-server] 启动失败: {e}");
        std::process::exit(1);
    }
}
