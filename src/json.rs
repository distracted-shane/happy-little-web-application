use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

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
    custom_css: String,
    js: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConf {
    pub templates: String,
    pub css: String,
    pub custom_css: String,
    pub javascript: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConf {
    pub socket: String,
    pub hostname: String,
}

// Struct for app configs. If you change this, remember:
//   - To update app.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SslConf {
    pub certfile: String,
    pub keyfile: String,
    pub socket: String,
}

// Load a context from JSON
pub async fn load(json_ctx: Context, path: &str) -> Context {
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
            println!("Loaded content data from JSON");
            Context::Content(Some(content_ctx))
        }

        Context::App(_) => {
            let app_ctx: AppConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("Loaded app data from JSON");
            Context::App(Some(app_ctx))
        }

        Context::Server(_) => {
            let server_ctx: ServerConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("Loaded server data from JSON");
            Context::Server(Some(server_ctx))
        }

        Context::Ssl(_) => {
            let ssl_ctx: SslConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    ::std::process::exit(1);
                }
            };
            println!("Loaded SSL data from JSON");
            Context::Ssl(Some(ssl_ctx))
        }
    }
}
