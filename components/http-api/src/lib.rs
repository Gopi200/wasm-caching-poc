use wstd::{
    http::{Body, Request, Response, StatusCode},
    io::AsyncRead,
    net::TcpStream,
};

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    match req.uri().path_and_query().unwrap().as_str() {
        "/" => home(req).await,
        _ => not_found(req).await,
    }
}

async fn home(_req: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    // Return a simple response with a string body
    let mut buf = [0u8; 4];
    assert_eq!(
        TcpStream::connect("127.0.0.1:8080")
            .await?
            .read(&mut buf)
            .await?,
        4
    );
    let current_value = u32::from_be_bytes(buf);
    Ok(Response::new(
        format!("Hello from wasmCloud!\nCurrent value is {current_value}").into(),
    ))
}

async fn not_found(_req: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not found\n".into())
        .unwrap())
}
