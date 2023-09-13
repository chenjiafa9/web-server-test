use std::sync::{mpsc, Arc, Mutex};
use std::thread;

///ThreadPool 结构体
/// workers : 包含Worker结构体的Vec
/// sender : 通道发送端，泛型为枚举<控制是否终止或执行任务>
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}
///类型别名动态分发约束与FnOnce() + Send + 'static
type Job = Box<dyn FnOnce() + Send + 'static>;

///Worker结构体
/// id 线程标识
/// thread Option包裹类型为 线程创建后类型thread::JoinHandle<()>
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

///Message 枚举
/// NewJob 元祖结构体 Job 闭包
/// Treminate 代表无任务枚举值
enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    ///new函数
    ///size 线程数
    pub fn new(size: usize) -> ThreadPool {
        println!("开始初始化线程池.");
        //线程数大于0
        assert!(size > 0);
        //创建通道
        println!("创建通道.");
        let (sender, receiver) = mpsc::channel();
        //通过Arc+Mutex包装后的接收端可以分发给多个线程
        let receiver = Arc::new(Mutex::new(receiver));
        //创建一个空Vec -- 长度为size大小
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            //循环指定线程次数，并把Worker实例放入其中（通过new关联函数获取）{id，包装后的接收端--锁保证同时只会有一个线
            //程接收到内容， Arc 保证接收端在个线程中的共享}
            println!("创建工作线程{}",id);
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }
        //返回ThreadPool实例{包含Worker实例的Vec ，发送端}
        ThreadPool { workers, sender }
    }
    ///execute 方法
    /// 接收一个闭包 约束与FnOnce() + Send + 'static
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        //Box包装参数闭包
        let job = Box::new(f);
        //通过发送端发送该闭包（放在枚举中，NewJob代表执行线程）
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    ///手动为ThreadPool结构体实现drop函数
    fn drop(&mut self) {
        //第一次for循环向每个线程发送终止信号 {workers包含所有线程}
        for _ in &mut self.workers{
            println!("向所有线程发送终止命令.");
            self.sender.send(Message::Terminate).unwrap();
        }
        
        //第二次for循环为正在执行的线程任务调用join（join：等待线程任务结束后再关闭线程）
        for worker in &mut self.workers {
            println!("关闭线程： {}", worker.id);
            //thread通过Option包装Some有值代表线程有任务执行{take方法:返回Option的Some值，替换为None}
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    ///Worker结构体关联函数new
    /// id 线程标识
    /// receiver 接收端:<Arc<Mutex<JoinHandle<()>>>>
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        println!("工作线程{}已创建.",id);
        //loop循环  move把接收端移动到闭包
        let thread = thread::spawn(move || {loop {
            //接收端获取锁lock()，调用recv()阻塞当前线程，等待执行完毕
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                //模式匹配Message枚举值 NewJob 代表有闭包任务执行
                Message::NewJob(job) => {
                    println!("线程： {} 获得一个任务.", id);
                    //线程使用该闭包 
                    job()
                }
                //无任务， 直接返回
                Message::Terminate => {
                    println!("线程： {} 无任务.", id);
                    break
                }
            }
        }});
        //返回Worker实例
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
