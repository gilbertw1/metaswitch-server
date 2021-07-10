use std::net::{IpAddr,SocketAddr};
use std::collections::HashMap;
use std::convert::Infallible;

use hyper::{self, Method, StatusCode, Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use serde_json;
use url::Url;

use crate::metacritic::MetacriticLookupHandler;

pub async fn start(host: IpAddr, port: u16, handler: MetacriticLookupHandler) {
    let addr = SocketAddr::new(host, port);
    let make_service = make_service_fn(|_conn| {
        let handl = handler.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(handl.clone(), req)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_service);

    println!("Running Metacritic Score Lookup Service at {}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle(handler: MetacriticLookupHandler, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/lookup") => {
            let url = Url::parse(&format!("http://blahh.com{}", req.uri())).unwrap();
            let query_map: HashMap<String,String> = url.query_pairs().into_owned().collect();
            let game = query_map.get("game");
            if game.is_some() {
                println!("Lookup request for game: {}", game.unwrap());
                let found_game = handler.lookup_game(game.unwrap());
                println!("Result: {:?}", found_game);
                if found_game.is_some() {
                    Ok(create_response(StatusCode::OK, Body::from(serde_json::to_string(&found_game.unwrap()).unwrap())))
                } else {
                    Ok(create_response(StatusCode::NOT_FOUND, Body::from("")))
                }
            } else {
                Ok(create_response(StatusCode::NOT_FOUND, Body::from("")))
            }
        },
        _ => {
            Ok(create_response(StatusCode::NOT_FOUND, Body::from("")))
        },
    }
}

fn create_response(code: StatusCode, body: Body) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(body)
        .unwrap()
}
