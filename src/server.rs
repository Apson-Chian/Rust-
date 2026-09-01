//! TCP 服务器：监听连接、读取请求、调用存储引擎并返回响应。
//!
//! 消息以 `\n` 分隔，服务器保证「一问一答」：读取到完整一行后才处理并回复。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::config::ServerConfig;
use crate::engine::Engine;
use crate::error::Result;
use crate::protocol::{Command, Response, MAX_LINE};

/// 启动服务器：恢复数据 → 监听端口 → 逐个处理连接
pub fn run(cfg: &ServerConfig) -> Result<()> {
    let mut engine = Engine::open(&cfg.data_file)?;
    let listener = TcpListener::bind(&cfg.addr)?;
    println!(
        "[rkv-server] 启动成功，监听 {}，数据文件 {}，已恢复 {} 个键",
        listener.local_addr()?,
        cfg.data_file.display(),
        engine.key_count()
    );

    for stream in listener.incoming() {
        match stream {
            // 单个连接出错只影响该连接，服务器继续运行
            Ok(s) => {
                if let Err(e) = handle_conn(s, &mut engine) {
                    eprintln!("[rkv-server] 连接异常结束: {e}");
                }
            }
            Err(e) => eprintln!("[rkv-server] 接受连接失败: {e}"),
        }
    }
    Ok(())
}

/// 处理单个连接上的所有请求，直到客户端 QUIT 或断开
fn handle_conn(stream: TcpStream, engine: &mut Engine) -> Result<()> {
    let peer = stream.peer_addr()?;
    println!("[rkv-server] 客户端接入: {peer}");
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    loop {
        let response = match read_request(&mut reader)? {
            Request::Eof => break,
            Request::TooLong => Response::Err(format!("请求超长，上限 {MAX_LINE} 字节")),
            Request::NotUtf8 => Response::Err("请求不是合法的 UTF-8 文本".into()),
            Request::Line(line) if line.trim().is_empty() => continue,
            // 解析失败只回错误，连接保持可用，后续合法命令照常处理
            Request::Line(line) => match Command::parse(&line) {
                Ok(cmd) => {
                    let resp = engine.execute(&cmd, 1);
                    if cmd == Command::Quit {
                        write_response(&mut writer, &resp)?;
                        break;
                    }
                    resp
                }
                Err(e) => Response::Err(e.to_string()),
            },
        };
        write_response(&mut writer, &response)?;
    }

    println!("[rkv-server] 客户端断开: {peer}");
    Ok(())
}

fn write_response(w: &mut impl Write, resp: &Response) -> Result<()> {
    w.write_all(resp.encode().as_bytes())?;
    w.write_all(b"\n")?;
    w.flush()?;
    Ok(())
}

/// 一次读取的结果
enum Request {
    /// 读到完整一行请求
    Line(String),
    /// 对端关闭
    Eof,
    /// 单行超过上限，已丢弃该行剩余内容
    TooLong,
    /// 内容不是合法 UTF-8
    NotUtf8,
}

/// 带长度上限地读取一行请求，防止超长请求耗尽内存。
/// 超长时丢弃本行剩余字节，使连接可以继续处理下一条命令。
fn read_request<R: BufRead>(reader: &mut R) -> Result<Request> {
    let mut buf = Vec::new();
    let n = reader
        .by_ref()
        .take(MAX_LINE as u64)
        .read_until(b'\n', &mut buf)?;
    if n == 0 {
        return Ok(Request::Eof);
    }
    if buf.last() != Some(&b'\n') {
        // 达到上限时这一行仍未结束，跳过其剩余部分
        skip_rest_of_line(reader)?;
        return Ok(Request::TooLong);
    }
    match String::from_utf8(buf) {
        Ok(line) => Ok(Request::Line(line)),
        Err(_) => Ok(Request::NotUtf8),
    }
}

fn skip_rest_of_line<R: BufRead>(reader: &mut R) -> Result<()> {
    let mut junk = Vec::new();
    loop {
        junk.clear();
        let n = reader
            .by_ref()
            .take(MAX_LINE as u64)
            .read_until(b'\n', &mut junk)?;
        if n == 0 || junk.last() == Some(&b'\n') {
            return Ok(());
        }
    }
}
