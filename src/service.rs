use std::net::{IpAddr,SocketAddr};
use std::collections::HashMap;

use futures::{Future,future};
use futures;
use futures::*;
use hyper::{self, Method, StatusCode, Body, Request, Response};
use hyper::service::{Service, NewService};
use hyper::server::conn::Http;
use serde_json;
use tokio;
use tokio::net::TcpListener;
use url::Url;

use metacritic::MetacriticLookupHandler;

pub struct LookupService {
    handler: MetacriticLookupHandler
}

impl NewService for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Service = LookupService;
    type Future = Box<Future<Item=Self::Service, Error=Self::InitError> + Send>;
    type InitError = hyper::Error;

    fn new_service(&self) -> Self::Future {
        Box::new(futures::future::ok(Self { handler: self.handler.clone() }))
    }
}

impl Service for LookupService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/lookup") => {
                let url = Url::parse(&format!("http://blahh.com{}", req.uri())).unwrap();
                let query_map: HashMap<String,String> = url.query_pairs().into_owned().collect();
                let game = query_map.get("game");
                if game.is_some() {
                    println!("Lookup request for game: {}", game.unwrap());
                    let found_game = self.handler.lookup_game(game.unwrap());
                    println!("Result: {:?}", found_game);
                    if found_game.is_some() {
                        Self::create_response(StatusCode::OK, Body::from(serde_json::to_string(&found_game.unwrap()).unwrap()))
                    } else {
                        Self::create_response(StatusCode::NOT_FOUND, Body::from(""))
                    }
                } else {
                    Self::create_response(StatusCode::NOT_FOUND, Body::from(""))
                }
            },
            _ => {
                Self::create_response(StatusCode::NOT_FOUND, Body::from(""))
            },
        }
    }
}

impl LookupService {
    pub fn start(host: IpAddr, port: u16, handler: MetacriticLookupHandler) {
        let addr = SocketAddr::new(host, port);
        let listener = TcpListener::bind(&addr).unwrap();
        let server = listener.incoming()
                             .map_err(|e| println!("error = {:?}", e))
                             .for_each(move |stream| {
                                 let future = Http::new()
                                     .serve_connection(stream, LookupService { handler: handler.clone() })
                                     .map_err(|e| eprintln!("server error: {}", e));
                                 tokio::spawn(future);
                                 Ok(())
                             });
        println!("Running Metacritic Score Lookup Service at {}", addr);
        tokio::run(server);
    }

    fn create_response(code: StatusCode, body: Body) -> <LookupService as Service>::Future {
        Box::new(
            future::ok(
                Response::builder()
                    .status(code)
                    .body(body)
                    .unwrap()))
    }
}
