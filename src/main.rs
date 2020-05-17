#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::Read;

use actix_web::{web, App, Error, HttpResponse, HttpServer};
use async_std::task;
use serde::{Deserialize, Serialize};
use tera::Tera;

// The CSS framework won't change often and it is so smol.
// So we'll just lazy-load it statically.
lazy_static! {
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
}

// The JS framework won't be changing often and it is smol, so smol.
// So we'll just lazy-load it statically.
lazy_static! {
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

// Enum of contexts we can grab from JSON
#[derive(Serialize, Deserialize, Debug)]
pub enum Context {
    WWW(Option<SiteGlobal>),
    Server(Option<ServerGlobal>),
}

// Struct for global site context. If you change this, remember:
//   - To update site.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteGlobal {
    name: String,
    url: String,
    author: String,
    description: String,
    charset: String,
    lang: String,
}

// Struct for global serve context. If you change this, remember:
//   - To update server.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerGlobal {
    socket: String,
    templates: String,
    css: String,
    javascript: String,
}

// Load a context
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
        Context::WWW(_) => {
            let ctx: SiteGlobal = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::WWW(Some(ctx.clone()))); //Eventually remove or rework w/o clone; just for testing
            Context::WWW(Some(ctx))
        }

        Context::Server(_) => {
            let ctx: ServerGlobal = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e); //Eventually remove or rework w/o clone; just for testing
                    ::std::process::exit(1);
                }
            };
            println!("{:#?}", Context::Server(Some(ctx.clone())));
            Context::Server(Some(ctx))
        }
    }
}

async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // Load context for Tera. Make sure it worked-- that it isn't empty or some
    // other kind of context.
    let ctx = match load_json_ctx(Context::WWW(None), "/json/site.json").await {
        Context::WWW(Some(t)) => tera::Context::from_serialize(t).unwrap(),
        Context::WWW(None) => {
            println!("Something's very wrong. Recieved blank context for site.");
            ::std::process::exit(1);
        }
        Context::Server(_) => {
            println!("Something's very wrong. Recieved server context for site.");
            ::std::process::exit(1);
        }
    };
    let s = tmpl.render("base.html.tera", &ctx).unwrap();
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(s))
}

async fn css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(&*CSS))
}

async fn js() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        .body(&*JS))
}

fn server_conf(path: &str) -> ServerGlobal {
    let result = async {
        match load_json_ctx(Context::Server(None), path).await {
            Context::Server(Some(t)) => t,
            Context::Server(None) => {
                println!("Something's very wrong. Recieved blank context for server.");
                ::std::process::exit(1);
            }
            Context::WWW(_) => {
                println!("Something's very wrong. Recieved site context for server.");
                ::std::process::exit(1);
            }
        }
    };
    task::block_on(result)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let svr_conf = server_conf("/json/server.json");
        let tera_templates = env!("CARGO_MANIFEST_DIR").to_owned() + &svr_conf.templates;
        let tera = match Tera::new(&tera_templates) {
            Ok(t) => t,
            Err(e) => {
                println!("Tera error(s): {}", e);
                ::std::process::exit(1);
            }
        };

        App::new()
            .data(tera)
            .service(web::resource("/").route(web::get().to(index)))
            .route(&svr_conf.css, web::get().to(css))
            .route(&svr_conf.javascript, web::get().to(js))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
