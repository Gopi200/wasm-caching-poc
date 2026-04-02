use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use wstd::{
    io::{AsyncRead, AsyncWrite},
    iter::AsyncIterator,
    net::TcpStream,
};

const READ_BUFFER_SIZE: usize = 4096;

#[derive(Serialize, Deserialize, Debug)]
/// The error type for errors originating from either side of a TCP connection.
pub enum TCP {
    /// The error that occurs when a message or a response could not be read by the recipient.
    Read,
    /// The error that occurs when a message or a response could not be written to the TCP stream
    Write,
    /// The error that occurs when a message or a response could not be deserialized into the expected data type.
    Deserialization,
    /// The error that occurs when a message or a response could not be serialized. Check [serde_json::to_vec] for more info.
    Serialization,
}

#[derive(Serialize, Deserialize, Debug)]
/// The error type for errors in communication between a TCP server and client.
pub enum CommunicationError {
    Server(TCP),
    Client(TCP),
    Connection,
}

/// Sends a message to the TCP server running on the given port in this workload.
///
/// Note that the serialized message needs to be deserializable into the type expected by the server, and vice versa for the response.
/// The recommended way to do this is to always send the same object the server expects, and always expect the same object the server sends.
/// It is however also possible to use a custom serialize/deserialize implementation for this.
pub async fn send_message<T: for<'de> serde::Deserialize<'de>, U: serde::Serialize>(
    port: u16,
    message: U,
) -> Result<T, CommunicationError> {
    let mut buf = Vec::new();
    let mut conn = TcpStream::connect(("127.0.0.1", port))
        .await
        .map_err(|_| CommunicationError::Connection)?;

    let message_bytes =
        serde_json::to_vec(&message).map_err(|_| CommunicationError::Client(TCP::Serialization))?;
    conn.write_all(&message_bytes)
        .await
        .map_err(|_| CommunicationError::Client(TCP::Write))?;
    conn.read_to_end(&mut buf)
        .await
        .map_err(|_| CommunicationError::Client(TCP::Read))?;
    serde_json::from_slice::<Result<T, TCP>>(&buf)
        .map_err(|_| CommunicationError::Client(TCP::Serialization))?
        .map_err(CommunicationError::Server)
}

/// Runs a TCP server on 0.0.0.0 with the given port.
/// If properly used on a wasm service this should mean every other component in the same workload can connect to it to send messages.
/// It is safe to run multiple services with the same port because they all run in separate workloads.
///
/// [serde_json] is used for the serialization and deserialization, so it is possible to use the [serde_json::Value] if you need to receive generic types.
pub async fn run_tcp_server<T: for<'de> serde::Deserialize<'de>, U: serde::Serialize>(
    port: u16,
    closure: impl AsyncFnMut(T) -> U + 'static + Clone,
) -> wstd::io::Result<()> {
    let listener =
        wstd::net::TcpListener::bind(&SocketAddr::from(([0, 0, 0, 0], port)).to_string()).await?;

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let mut stream = stream?;

        let mut closure = closure.clone();
        wstd::runtime::spawn(async move {
            read_message(&mut stream, async |stream, message| {
                send_response(stream, &Ok(closure(message).await)).await;
            })
            .await;
        })
        .detach();
    }
    Ok(())
}

async fn read_message<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
    mut closure: impl AsyncFnMut(&mut TcpStream, T),
) {
    match read_bytes(stream).await {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(val) => closure(stream, val).await,
            Err(_) => send_response(stream, &Err::<(), TCP>(TCP::Deserialization)).await,
        },
        Err(_) => send_response(stream, &Err::<(), TCP>(TCP::Read)).await,
    }
}

async fn read_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, TCP> {
    let mut vec = Vec::new();
    let mut buf = [0u8; READ_BUFFER_SIZE];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(READ_BUFFER_SIZE) => vec.extend_from_slice(&buf),
            Ok(size) => {
                vec.extend(buf.into_iter().take(size));
                break;
            }
            Err(_) => return Err(TCP::Read),
        }
    }
    Ok(vec)
}

async fn send_response<T: Serialize>(stream: &mut TcpStream, message: &Result<T, TCP>) {
    match serde_json::to_vec(&message) {
        Ok(bytes) => {
            let _ = stream.write_all(&bytes).await;
        }
        Err(_) => {
            let _ = stream
                .write_all(
                    &serde_json::to_vec(&Err::<T, TCP>(TCP::Serialization))
                        .expect("Serialization of serialization error failed"),
                )
                .await;
        }
    }
}
