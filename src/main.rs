use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    // Graceful stop 을 테스트하기 위해 두 개의 요청만 받고 종료
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.")
}

// TCP 요청에 대해 비지니스 로직을 처리하는 메소드
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status_line, contents.len(), contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

// ThreadPool -> (Channel) -> Worker (waiting)
impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // Job 을 worker 에게 전송할 때 사용할 channel 을 생성한다.
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        // 유한 개의 worker 그룹을 만든다.
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender
        }
    }

    // f 라는 요청을 받아서 job 으로 만든다.
    fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

// Graceful termination
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Terminate message 전송
        println!("Sending terminate message to all workers");
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers");
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // Worker 는 쓰레드 생성 후 무한 루프 돌면서 receiver 를 기다린다.
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(thread)
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
