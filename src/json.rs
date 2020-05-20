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
    let mut data = String::new();

    // Open and read file or err
    let full_path = env!("CARGO_MANIFEST_DIR").to_owned() + path;
    if let Ok(mut file) = File::open(&full_path) {
        if file.read_to_string(&mut data).is_err() {
            println!("JSON file read error(s).");
        }
    }

    // Load the context to the correct struct. Enums. Enums!!!
    match json_ctx {
        Context::Content(_) => {
            let content_ctx: ContentConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    return Context::Content(None);
                }
            };
            Context::Content(Some(content_ctx))
        }

        Context::App(_) => {
            let app_ctx: AppConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    return Context::App(None);
                }
            };
            Context::App(Some(app_ctx))
        }

        Context::Server(_) => {
            let server_ctx: ServerConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    return Context::Server(None);
                }
            };
            Context::Server(Some(server_ctx))
        }

        Context::Ssl(_) => {
            let ssl_ctx: SslConf = match serde_json::from_str(&data) {
                Ok(c) => c,
                Err(e) => {
                    println!("Serde deserialization error(s): {}", e);
                    println!("{}", &data);
                    return Context::Ssl(None);
                }
            };
            Context::Ssl(Some(ssl_ctx))
        }
    }
}
