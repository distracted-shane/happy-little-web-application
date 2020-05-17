#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::{Read, BufReader};

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use async_std::task;
use rustls::{NoClientAuth, ServerConfig};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use serde::{Deserialize, Serialize};
use tera::Tera;

lazy_static! {
    // The CSS framework won't change often and it is so smol.
    // So we'll just lazy-load it statically. [Eventually we'll want to procompress this shiz]
    pub static ref CSS: String = {

        // Open file or err
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/css/picnic.min.css");
        let mut f = match File::open(path) {
            Ok(t) => t,
            Err(e) => {
                println!("CSS file open error(s): {}\n\tPath: {}", e, path);
                ::std::process::exit(1);
            }
        };

        // Read to string or err
        let mut data = String::new();
        if f.read_to_string(&mut data).is_err() {
                println!("CSS file read error(s).");
                ::std::process::exit(1);
        }
        data
    };

// The JS framework won't be changing often and it is smol, so smol.
// So we'll just lazy-load it statically. [Eventually we'll want to procompress this shiz]
    pub static ref JS: String = {

        // Open file or err
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/JS/umbrella.min.js");
        let mut f = match File::open(path) {
            Ok(t) => t,
            Err(e) => {
                println!("JS file open error(s): {}\n\tPath: {}", e, path);
                ::std::process::exit(1);
            }
        };

        // Read to string or err
        let mut data = String::new();
        if f.read_to_string(&mut data).is_err() {
                println!("JS file read error(s).");
                ::std::process::exit(1);
        }
    data
    };
}

// Enum of contexts or configs we can grab from JSON
#[derive(Serialize, Deserialize, Debug)]
pub enum Context {
    Content(Option<ContentConf>),
    App(Option<AppConf>),
    Server(Option<ServerConf>),
    Ssl(Option<SslConf>),
}

// Struct for content context. If you change this, remember:
//   - To update content.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentConf {
    name: String,
    url: String,
    author: String,
    description: String,
    charset: String,
    lang: String,
    css: String,
    js: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConf {
    templates: String,
    css: String,
    javascript: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConf {
    socket: String,
    hostname: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SslConf {
    certfile: String,
    keyfile: String,
    socket: String,
}

// Load a context from JSON
async fn load_json_ctx(json_ctx: Context, path: &str) -> Context {
    // Open file or err
    let full_path = env!("CARGO_MANIFEST_DIR").to_owned() + path;
    let mut f = match File::open(&full_path) {
        Ok(t) => t,
        Err(e) => {
            println!("JSON file open error(s): {}\n\tPath: {}", e, full_path);
            ::std::process::exit(1);
        }
    };

    // Read to string or err
    let mut data = String::new();
    if f.read_to_string(&mut data).is_err() {
        println!("JSON file read error(s).");
        ::std::process::exit(1);
    }

    // Load the context to the correct struct. Enums. Enums!!!
    match json_ctx {
        Context::Content(_) => {
            let content_ctx: ContentConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::Content(Some(content_ctx.clone()))); //Eventually remove or rework w/o clone; just for testing
            Context::Content(Some(content_ctx))
        },

        Context::App(_) => {
            let app_ctx: AppConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e); //Eventually remove or rework w/o clone; just for testing
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::App(Some(app_ctx.clone())));
            Context::App(Some(app_ctx))
        },

        Context::Server(_) => {
            let server_ctx: ServerConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e); //Eventually remove or rework w/o clone; just for testing
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::Server(Some(server_ctx.clone())));
            Context::Server(Some(server_ctx))
        },

        Context::Ssl(_) => {
            let ssl_ctx: SslConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e); //Eventually remove or rework w/o clone; just for testing
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::Ssl(Some(ssl_ctx.clone())));
            Context::Ssl(Some(ssl_ctx))
        }
    }
}

// General templating route.
async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // Load context for Tera. Make sure it worked-- that it isn't empty or some
    // other kind of context.
    let ctx = match load_json_ctx(Context::Content(None), "/json/content.json").await {
        Context::Content(Some(t)) => tera::Context::from_serialize(t).unwrap(),
        Context::Content(None) => {
            println!("Error: recieved blank context for content.");
            ::std::process::exit(1);
        }
        _ => {
            println!("Error: recieved incorrect context type.");
            ::std::process::exit(1);
        }
    };
    let s = tmpl.render("index.html.tera", &ctx).unwrap();
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(s))
}

// Send CSS. [Eventually we'll want to procompress this shiz]
async fn css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(&*CSS))
}

// Send JS. [Eventually we'll want to procompress this shiz]
async fn js() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        .body(&*JS))
}

// Load application-specific configurations
async fn app_conf(path: &str) -> AppConf {
    match load_json_ctx(Context::App(None), path).await {
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
    match load_json_ctx(Context::Server(None), path).await {
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
    match load_json_ctx(Context::Ssl(None), path).await {
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
            .service(web::resource("/").route(web::get().to(index)))
            .route(&app.css, web::get().to(css))
            .route(&app.javascript, web::get().to(js))
    })
    .bind(&svr.socket)?
    .bind_rustls(&ssl.socket, config)?
    .server_hostname(&svr.hostname)
    .run()
    .await
}
