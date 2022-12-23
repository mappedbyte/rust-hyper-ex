use std::fmt::format;
use anyhow::*;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Body, Client, Request, Server};
use hyper::service::{make_service_fn, service_fn};


fn proxy_create(req: &mut Request<Body>) -> Result<()> {
    for key in &["content-length", "accept_encoding", "content-encoding", "transfer-encoding"] {
        req.headers_mut().remove(*key);
    };
    let uri = req.uri();
    let uri_string = match uri.query() {
        Some(query_item) => format!("https://crates.io{}?{}", uri.path(), query_item),
        None => format!("https://crates.io{}", uri.path())
    };
    *req.uri_mut() = uri_string.parse().context("parsing URI Error")?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client: Client<_, hyper::Body> = Client::builder().build(https);
    let client: Arc<Client<_, hyper::Body>> = Arc::new(client);
    let addr = SocketAddr::from(([0, 0, 0, 0], 7000));
    let make_svc = make_service_fn(move |_| {
        let client = Arc::clone(&client);
        async move {
            Ok::<_>(service_fn(move |mut req| {
                let client = Arc::clone(&client);
                async move {
                    println!("proxy:{}", req.uri().path());
                    proxy_create(&mut req)?;
                    client.request(req).await.context("proxy request")
                }
            }))
        }
    });
    let _server = Server::bind(&addr).serve(make_svc).await.context("run server");

    Ok::<()>(())
}