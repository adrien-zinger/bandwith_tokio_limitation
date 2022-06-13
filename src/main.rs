use anyhow::Result;
use async_speed_limit::Limiter;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

//const READ_LIMIT: f64 = 1024.0;
const READ_LIMIT: f64 = 8192.0;
//const WRITE_LIMIT: f64 = 1024.0;
const WRITE_LIMIT: f64 = 8192.0;

struct Reader;
struct Writer;

impl Reader {
    /// Spawn a reader server on his own thread with his own tokio runtime
    ///
    /// Use tokio runtime to reproduce the tools in massa binders.
    pub fn spawn() -> Result<Receiver<Result<()>>> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            sender.send(rt.block_on(Self::run())).unwrap();
        });
        Ok(receiver)
    }

    async fn run() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        let read_half: OwnedReadHalf = listener.accept().await?.0.into_split().0;
        let limiter = <Limiter>::new(READ_LIMIT);
        let mut read_half = limiter.limit(read_half);
        let mut buf = [1; 1024];
        println!("reader started");
        while buf[0] != 0 {
            read_half.read_exact(&mut buf).await?;
            println!("read a buff of {}", buf[0]);
        }
        Ok(())
    }
}

impl Writer {
    /// Spawn a reader server on his own thread with his own tokio runtime
    ///
    /// Use tokio runtime to reproduce the tools in massa binders.
    pub fn spawn() -> Result<Receiver<Result<()>>> {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            sender.send(rt.block_on(Self::run())).unwrap();
        });
        Ok(receiver)
    }

    async fn run() -> Result<()> {
        let limiter = <Limiter>::new(WRITE_LIMIT);
        let stream = TcpStream::connect("127.0.0.1:8080").await?;
        let read_half: OwnedWriteHalf = stream.into_split().1;
        let mut write_half = limiter.limit(read_half);
        let buf = [1; 1024];
        for _ in 0..1000 {
            write_half.write_all(&buf).await?;
        }
        write_half.write_all(&[0; 1024]).await?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let on_reader_stop = Reader::spawn()?;
    let on_writer_stop = Writer::spawn()?;
    on_reader_stop.recv()??;
    on_writer_stop.recv()??;
    println!("ok!");
    Ok(())
}
