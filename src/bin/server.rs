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
    println!("[rkv-server] 配置: 监听 {} 数据文件 {:?}", cfg.addr, cfg.data_file);
    println!("[rkv-server] 网络服务将在阶段五接入");
}
