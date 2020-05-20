use std::fs::File;
use std::io::Read;
use std::path::Path;

use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse};

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

    // The custom CSS won't change often and it is smol.
    // So we'll just lazy-load it statically. [Eventually we'll want to procompress this shiz]
    pub static ref CUSTOM_CSS: String = {

        // Open file or err
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/css/custom.css");
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

// General index route.
pub async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // Load context for Tera. Make sure it worked-- that it isn't empty or some
    // other kind of context.
    let ctx = match json::load(Context::Content(None), "/json/content.json").await {
        Context::Content(Some(t)) => tera::Context::from_serialize(t).unwrap(),
        Context::Content(None) => {
            println!("Error: recieved blank context for content.");
            return Err(ErrorInternalServerError("ERR-IND-BC"));
        }
        _ => {
            println!("Error: recieved incorrect context type.");
            return Err(ErrorInternalServerError("ERR-IND-IC"));
        }
    };
    match tmpl.render("index.html.tera", &ctx) {
        Ok(t) => Ok(HttpResponse::Ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(t)),
        Err(e) => {
            println!("Error rendering template: {}", e);
            Err(ErrorInternalServerError("ERR-IND-RT"))
        }
    }
}

// Cisco route
pub async fn cisco(
    tmpl: web::Data<tera::Tera>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let base = env!("CARGO_MANIFEST_DIR").to_owned();
    let relative = format!("cisco/{}.html.tera", &path.0);
    let full = base + "/templates/" + &relative;

    if !Path::new(&full).exists() {
        println!("\tError: could not find template: {} ", full);
        Ok(e404().await)
    } else {
        let ctx = match json::load(Context::Content(None), "/json/content.json").await {
            Context::Content(Some(t)) => tera::Context::from_serialize(t).unwrap(),
            Context::Content(None) => {
                println!("Error: recieved blank context for content.");
                return Err(ErrorInternalServerError("ERR-CIS-BC"));
            }
            _ => {
                println!("Error: recieved incorrect context type.");
                return Err(ErrorInternalServerError("ERR-CIS-IC"));
            }
        };

        let t = tmpl.render(&relative, &ctx).unwrap();
        Ok(HttpResponse::Ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(t))
    }
}

// Linux route
pub async fn linux(
    tmpl: web::Data<tera::Tera>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let base = env!("CARGO_MANIFEST_DIR").to_owned();
    let relative = format!("linux/{}.html.tera", &path.0);
    let full = base + "/templates/" + &relative;

    // Drop out early (before templating) if file doesn't exist.
    if !Path::new(&full).exists() {
        println!("\tError: could not find template: {} ", full);
        Ok(e404().await)
    } else {
        let ctx = match json::load(Context::Content(None), "/json/content.json").await {
            Context::Content(Some(t)) => tera::Context::from_serialize(t).unwrap(),
            Context::Content(None) => {
                println!("Error: recieved blank context for content.");
                return Err(ErrorInternalServerError("ERR-LIN-BC"));
            }
            _ => {
                println!("Error: recieved incorrect context type.");
                return Err(ErrorInternalServerError("ERR-LIN-IC"));
            }
        };

        let t = tmpl.render(&relative, &ctx).unwrap();
        Ok(HttpResponse::Ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(t))
    }
}

// Send CSS. [Eventually we'll want to procompress this shiz]
pub async fn css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .header("Content-Type", "text/css; charset=utf-8")
        .body(&*CSS))
}

// Send CSS. [Eventually we'll want to procompress this shiz]
pub async fn custom_css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .header("Content-Type", "text/css; charset=utf-8")
        .body(&*CUSTOM_CSS))
}

// Send JS. [Eventually we'll want to procompress this shiz]
pub async fn js() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .header("Content-Type", "text/javascript; charset=utf-8")
        .body(&*JS))
}

// 404 Error
pub async fn e404() -> HttpResponse {
    HttpResponse::NotFound()
        .header("Content-Type", "text/plain; charset=ut8-8")
        .body("Error - file note found")
}
