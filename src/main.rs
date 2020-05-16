#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::Read;

use actix_web::{web, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use tera::Tera;

// The CSS framework won't change often and its smol.
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

// The JS framework won't be changing often and its smol.
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
// Struct for global site context. If you change this, remember:
//   - To update site.json
#[derive(Serialize, Deserialize)]
pub struct SiteGlobal {
    name: String,
    url: String,
    author: String,
    description: String,
    charset: String,
    lang: String,
}

// Load the global context.
// Eventually, it will be abstracted to take an enum 'Context' and a file name
async fn load_global_ctx() -> SiteGlobal {
    // Open file or err
    let mut f = match File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/json/site.json")) {
        Ok(t) => t,
        Err(e) => {
            println!("JSON file open error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    // Read to string or err
    let mut data = String::new();
    if f.read_to_string(&mut data).is_err() {
            println!("JSON file read error(s).");
            ::std::process::exit(1);
        }

    // Deserialize to struct or err
    let ctx: SiteGlobal = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(e) => {
            println!("Serde deserialization error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    ctx
} 

async fn index(
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
        let ctx = tera::Context::from_serialize(load_global_ctx().await).unwrap();
        let s = tmpl.render("base.html.tera", &ctx).unwrap();
        Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(s))
}

async fn css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/css; charset=utf-8").body(&*CSS))
}

async fn js() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/javascript; charset=utf-8").body(&*JS))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    HttpServer::new(|| {
        let tera = match Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")) {
                         Ok(t) => t,
                         Err(e) => {
                             println!("Tera error(s): {}", e);
                             ::std::process::exit(1);
                         }
                     };

        App::new()
            .data(tera)
            .service(web::resource("/")
                .route(web::get().to(index)))
                .route("/css/picnic.min.css", web::get().to(css))
                .route("/js/umbrella.min.js", web::get().to(js))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}