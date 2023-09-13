use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use web_server::ThreadPool;
fn main() {
    //监听127.0.0.1:8888地址端口
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
    println!("启动成功！正在监听：{:?}",listener.local_addr().unwrap());
    //listener.incoming()方法返回监听连接及相应内容的迭代器
    for stream in listener.incoming() {
        //取出Result<T>
        let stream = stream.unwrap();
        //调用ThreadPool 关联函数new 指定线程数4
        let pool = ThreadPool::new(4);
        //传入闭包
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}
///接收监听到的流数据内容
fn handle_connection(mut stream: TcpStream) {
    //创建空数组长度1024
    let mut buffer = [0; 1024];
    //读取stream流数据到buffer
    stream.read(&mut buffer).unwrap();
    //请求头标识变量
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    //let if 模式匹配请求头与标识变量是否匹配 ，不匹配时返回404页面内容
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    //读取html文件
    let contents = fs::read_to_string(filename).unwrap();
    //响应内容拼接
    let response = format!(
        "{}\r\nContent-Length:{}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    //写入响应体转成位字节
    stream.write(response.as_bytes()).unwrap();
    //响应请求
    stream.flush().unwrap();
}
