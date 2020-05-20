#[macro_use]
extern crate lazy_static;

use std::sync::mpsc;
use std::thread;

use actix_rt::System;
use actix_web::{middleware, web, App, HttpServer};
use async_std::task;
use dialoguer::{theme::ColorfulTheme, Select};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tera::Tera;
mod json;
use json::{AppConf, Context, ServerConf, SslConf};

mod routes;

// Load app configurations
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

// Le mains
#[actix_rt::main]
async fn main() {
    // We'll just loop forever. Until the user says otherwise. Allows pausing/resuming/reloading of server.
    loop {
        let svr = server_conf("/json/server.json").await;
        let ssl = ssl_conf("/json/ssl.json").await;
        let (tx, rx) = mpsc::channel();

        // SSL builder
        let builder = {
            let cert_path = env!("CARGO_MANIFEST_DIR").to_owned() + &ssl.certfile;
            let key_path = env!("CARGO_MANIFEST_DIR").to_owned() + &ssl.keyfile;
            let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file(key_path, SslFiletype::PEM)
                .unwrap();
            builder.set_certificate_chain_file(cert_path).unwrap();
            builder
        };

        // Server gets a thread
        thread::spawn(move || {
            let sys = System::new("http-server");
            let srv = HttpServer::new(|| {
                let app = task::block_on(app_conf("/json/app.json"));

                let tera = {
                    let tera_templates = env!("CARGO_MANIFEST_DIR").to_owned() + &app.templates;
                    match Tera::new(&tera_templates) {
                        Ok(t) => t,
                        Err(_) => {
                            //println!("Tera error(s): {}", e);
                            ::std::process::exit(1);
                        }
                    }
                };

                App::new()
                    .wrap(middleware::Compress::default())
                    .wrap(
                        middleware::DefaultHeaders::new()
                            .header("Referrer-Policy", "same-origin")
                            .header("X-Content-Type-Options", "nosniff")
                            .header("X-Frame-Options", "SAMEORIGIN")
                            .header("X-XSS-Protection", "1; mode=block"),
                    )
                    .data(tera)
                    .service(web::resource("/").route(web::get().to(routes::index)))
                    .service(web::resource("/linux/{article}").route(web::get().to(routes::linux)))
                    .service(web::resource("/cisco/{article}").route(web::get().to(routes::cisco)))
                    .service(web::resource(&app.css).route(web::get().to(routes::css)))
                    .service(
                        web::resource(&app.custom_css).route(web::get().to(routes::custom_css)),
                    )
                    .service(web::resource(&app.javascript).route(web::get().to(routes::js)))
            })
            .bind(&svr.socket)
            .unwrap()
            .bind_openssl(&ssl.socket, builder)
            .unwrap()
            .run();
            let _ = tx.send(srv);
            sys.run()
        });

        let srv = rx.recv().unwrap();

        let selections = &["Pause", "Resume", "Reload", "Quit"];

        // Loop over the menu
        loop {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Server action")
                .default(0)
                .items(&selections[..])
                .interact_opt()
                .unwrap();

            if let Some(selection) = selection {
                match selection {
                    0 => {
                        println!("Paused.");
                        srv.pause().await;
                    }
                    1 => {
                        println!("Resumed.");
                        srv.resume().await;
                    }
                    2 => {
                        println!("Reloaded.");
                        srv.stop(true).await;
                        break;
                    }
                    3 => {
                        println!("Quitting...");
                        srv.stop(true).await;
                        ::std::process::exit(0);
                    }
                    _ => println!("Woops!"),
                }
            }
        }
    }
}
