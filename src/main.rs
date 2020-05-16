// #[macro_use]                         [Not currently used]
extern crate tera;
#[macro_use]
extern crate lazy_static;

// use std::error::Error;               [Not currently used]
use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};
use tera::Tera;


// Tera only needs to be defined once. So: 
//   - We are using lazy_static
lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Tera error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

// Global site variables shouldn't change much. So:
//   - We are using lazy_static
lazy_static! {
    pub static ref SITE_GLOBAL_CTX: SiteGlobal = {

        // Open file or err
        let mut f = match File::open("json/site.json") {
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
    };
}

#[derive(Serialize, Deserialize)]
pub struct SiteGlobal {
    name: String,
    url: String,
    author: String,
    charset: String,
    lang: String,
}

fn main() {}
