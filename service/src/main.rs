use std::sync::{LazyLock, Mutex};

use wstd::{io::AsyncWrite, iter::AsyncIterator, net::TcpListener};

#[wstd::main]
async fn main() -> wstd::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Listening on {}", listener.local_addr()?);
    println!("type `nc localhost 8080` to create a TCP client");

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let mut stream = stream?;
        println!("Accepted from: {}", stream.peer_addr()?);
        wstd::runtime::spawn(async move {
            // If echo copy fails, we can ignore it.
            let _ = stream.write(&increment()).await;
        })
        .detach();
    }
    Ok(())
}

static CACHE: LazyLock<Mutex<u32>> = LazyLock::new(Mutex::default);

fn increment() -> [u8; 4] {
    // Return a simple response with a string body
    let mut val = CACHE.lock().unwrap();
    *val += 1;
    println!("{val}");
    val.to_be_bytes()
}
