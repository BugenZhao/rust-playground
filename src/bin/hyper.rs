use std::convert::Infallible;

use hyper::{
    http::HeaderValue,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body = Body::from("Hello World");
    let mut res = Response::new(body);

    res.headers_mut().insert("i64", 42.into());
    // res.extensions_mut().insert::<i64>(42_i64);
    // res.extensions_mut().insert::<i32>(42_i32);

    Ok(res)
}

async fn server() {
    // Construct our SocketAddr to listen on...
    let addr = "127.0.0.1:19999".parse().unwrap();

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn client() {
    let response = hyper::Client::new()
        .get("http://localhost:19999".parse().unwrap())
        .await
        .unwrap();

    let ext = response.headers().get("i64").unwrap();
    println!("i64: {:?}", ext);
}

#[tokio::main]
async fn main() {
    tokio::spawn(server());
    client().await;
}
