//! Utility responses for the axum server

use axum::http::{StatusCode, HeaderMap};
use axum::response::IntoResponse;

use axum::
{
    http::Request,
    middleware::Next,
    response::Response,
    body::Bytes,
};

use chrono::Local;

/// Response that returns the body of the request, a reflection
/// 
/// # Example
/// 
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
/// use std::sync::{Arc, Mutex};
///
/// use axum::
/// {
///     routing::post, 
///     Router, 
///     middleware
/// };
/// 
/// use pulse::web::response::util::reflect;
/// 
/// pub async fn server() {
/// let app = Router::new()
/// .route("/", post(|| async move {  }))
/// .layer(middleware::from_fn(reflect));
/// 
/// let ip = Ipv4Addr::new(127,0,0,1);
/// let addr = SocketAddr::new(IpAddr::V4(ip), 3030);
/// 
/// axum::Server::bind(&addr)
/// .serve(app.into_make_service_with_connect_info::<SocketAddr>())
/// .await
/// .unwrap();
/// }
/// ````

pub async fn reflect<B>
(
    headers: HeaderMap,
    request: Request<B>,
    _next: Next<B>
) -> Result<Response, StatusCode>
where B: axum::body::HttpBody<Data = Bytes>
{
    let (_parts, body) = request.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return Err(StatusCode::BAD_REQUEST)
        }
    };

    let body = std::str::from_utf8(&bytes).unwrap().to_string();

    println!("[{}] Got request:\n\nheader:\n\n{:?}\n\nbody:\n\n{}", Local::now(), headers, body);
    
    Ok(format!("You sent:\n{}",body).into_response())
    
}

/// Response that logs the header and body of the request to stdout
/// 
/// # Example
/// 
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
/// use std::sync::{Arc, Mutex};
///
/// use axum::
/// {
///     routing::post, 
///     Router, 
///     middleware
/// };
/// 
/// use pulse::web::response::util::stdout_log;
/// 
/// 
/// pub async fn server() {
/// let app = Router::new()
/// .route("/", post(|| async move {  }))
/// .layer(middleware::from_fn(stdout_log));
/// 
/// let ip = Ipv4Addr::new(127,0,0,1);
/// let addr = SocketAddr::new(IpAddr::V4(ip), 3000);
/// 
/// axum::Server::bind(&addr)
/// .serve(app.into_make_service_with_connect_info::<SocketAddr>())
/// .await
/// .unwrap();
/// }
/// ````
pub async fn stdout_log<B>
(
    headers: HeaderMap,
    request: Request<B>,
    _next: Next<B>
) -> Result<Response, StatusCode>
where B: axum::body::HttpBody<Data = Bytes>
{
    let (_parts, body) = request.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return Err(StatusCode::BAD_REQUEST)
        }
    };

    let body = std::str::from_utf8(&bytes).unwrap().to_string();

    println!("[{}] Got request:\n\nheader:\n\n{:?}\n\nbody:\n\n{}", Local::now(), headers, body);
    
    Ok((StatusCode::OK).into_response())
    
}