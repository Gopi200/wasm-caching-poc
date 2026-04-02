use shared::{Action, Direction};
use wash_service_helpers::send_message;
use wstd::http::{Body, Request, Response, StatusCode};

type HttpResult = Result<Response<Body>, wstd::http::Error>;

#[wstd::http_server]
async fn main(req: Request<Body>) -> HttpResult {
    match req.uri().path_and_query().unwrap().as_str() {
        "/" => home(req).await,
        "/increment" => increment(req).await,
        "/decrement" => decrement(req).await,
        _ => not_found(req).await,
    }
}

async fn home(_req: Request<Body>) -> HttpResult {
    let value: i32 = send_message(8080, Action::Get).await.unwrap();
    Ok(Response::new(
        format!("Current cache value is {value}").into(),
    ))
}

async fn increment(_req: Request<Body>) -> HttpResult {
    let value: i32 = send_message(8080, Action::Update(Direction::Increment))
        .await
        .unwrap();
    Ok(Response::new(
        format!("Incremented cache value to {value}").into(),
    ))
}

async fn decrement(_req: Request<Body>) -> HttpResult {
    let value: i32 = send_message(8080, Action::Update(Direction::Decrement))
        .await
        .unwrap();
    Ok(Response::new(
        format!("Decremented cache value to {value}").into(),
    ))
}

async fn not_found(_req: Request<Body>) -> HttpResult {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not found\n".into())
        .unwrap())
}
