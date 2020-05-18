use std::fs::File;
use std::io::Read;

use actix_web::{web, Error, HttpResponse};

use super::json::{self, Context};

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

// General templating route.
pub async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // Load context for Tera. Make sure it worked-- that it isn't empty or some
    // other kind of context.
    let ctx = match json::load(Context::Content(None), "/json/content.json").await {
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
pub async fn css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(&*CSS))
}

// Send JS. [Eventually we'll want to procompress this shiz]
pub async fn js() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        .body(&*JS))
}
