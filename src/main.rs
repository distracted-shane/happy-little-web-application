#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::BufReader;

use actix_web::{middleware, web, App, HttpServer};
use async_std::task;
use rustls::{NoClientAuth, ServerConfig};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use tera::Tera;

mod json;
use  json::{Context, AppConf, ServerConf, SslConf};

mod routes;

// Load application-specific configurations
async fn app_conf(path: &str) -> AppConf {
    match json::load(Context::App(None), path).await {
        Context::App(Some(t)) => t,
        Context::App(None) => {
            println!("Error: recieved blank context for application.");
            ::std::process::exit(1);
        }
        _ => {
            println!("Error: recieved incorrect context type.");
            ::std::process::exit(1);
        }
    }
}

// Load server configurations
async fn server_conf(path: &str) -> ServerConf {
    match json::load(Context::Server(None), path).await {
        Context::Server(Some(t)) => t,
        Context::Server(None) => {
            println!("Error: recieved blank context for server.");
            ::std::process::exit(1);
        }
        _ => {
            println!("Error: recieved incorrect context type.");
            ::std::process::exit(1);
        }
    }
}

// Load SSL configurations
async fn ssl_conf(path: &str) -> SslConf {
    match json::load(Context::Ssl(None), path).await {
        Context::Ssl(Some(t)) => t,
        Context::Ssl(None) => {
            println!("Error: recieved blank context for server.");
            ::std::process::exit(1);
        }
        _ => {
            println!("Error: recieved incorrect context type.");
            ::std::process::exit(1);
        }
    }
}
// Le main
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let svr = server_conf("/json/server.json").await;
    let ssl = ssl_conf("/json/ssl.json").await;
    let cert_path = env!("CARGO_MANIFEST_DIR").to_owned() + &ssl.certfile;
    let key_path = env!("CARGO_MANIFEST_DIR").to_owned() + &ssl.keyfile;

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open(cert_path).unwrap());
    let key_file = &mut BufReader::new(File::open(key_path).unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = rsa_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    HttpServer::new(|| {
        let app = task::block_on(app_conf("/json/app.json"));
        let tera_templates = env!("CARGO_MANIFEST_DIR").to_owned() + &app.templates;
        let tera = match Tera::new(&tera_templates) {
            Ok(t) => t,
            Err(e) => {
                println!("Tera error(s): {}", e);
                ::std::process::exit(1);
            }
        };

        App::new()
            .wrap(middleware::Compress::default())
            .data(tera)
            .service(web::resource("/").route(web::get().to(routes::index)))
            .route(&app.css, web::get().to(routes::css))
            .route(&app.javascript, web::get().to(routes::js))
    })
    .bind(&svr.socket)?
    .bind_rustls(&ssl.socket, config)?
    .server_hostname(&svr.hostname)
    .run()
    .await
}
