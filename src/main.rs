use std::env;
use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

#[macro_use]
extern crate log;

use hyper::Body;
use warp::http::Response;
use warp::{self, multipart::FormData, Filter};

use rand::{self, distributions::Distribution, Rng};

use async_std::fs::File;
use async_std::prelude::*;
use futures::StreamExt;

use bytes::buf::Buf;

fn init_logging() {
    use simplelog::*;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_level_padding(LevelPadding::Off)
                .build(),
            TerminalMode::Mixed,
        )
        .unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_level_padding(LevelPadding::Off)
                .build(),
            std::fs::File::create("super_simple_upload.log").unwrap(),
        ),
    ])
    .unwrap();
}

#[tokio::main]
async fn main() {
    init_logging();
    let status_handler = warp::get().map(|| "super-simple-upload running...");

    let upload_handler = warp::post()
        .and(warp::multipart::form().max_length(1024 * 1024 * 1024 /* 1GB max */))
        .and(warp::header("Authorization"))
        .and_then(handle_upload);

    let routes = status_handler.or(upload_handler);

    let port: u16 = env::var("PORT")
        .map(|s| s.parse().expect("PORT env var is not a valid port!"))
        .unwrap_or(8080);
    
    warp::serve(routes).run(([0, 0, 0, 0], port)).await
}

async fn handle_upload(
    mut form: FormData,
    key: String,
) -> Result<warp::reply::Response, warp::Rejection> {
    if !check_key(key) {
        return Ok(Response::builder().status(403).body(Body::empty()).unwrap());
    }

    let mut names: Vec<String> = Vec::new();

    while let Some(Ok(part)) = form.next().await {
        let extension = part.filename().and_then(|name| name.split(".").last());
        let orig_filename = part.filename().unwrap_or("<none>").to_string();
        let name = match extension {
            Some(extension) => format!("{}.{}", generate_random_string(5), extension),
            None => generate_random_string(6),
        };

        if let Err(_) = write_file(&name, part).await {
            warn!(
                "An error occurred while attempting to write {} (orig: {})",
                &name, &orig_filename
            );

            return Ok(Response::builder().status(500).body(Body::empty()).unwrap());
        }

        info!("Uploaded {} (orig: {})", &name, &orig_filename);
        names.push(name);
    }

    Ok(Response::builder()
        .body(Body::from(names.join("\n")))
        .unwrap())
}

async fn write_file(name: &String, part: warp::multipart::Part) -> Result<(), std::io::Error> {
    let target_path = PathBuf::from("./uploads").join(name);

    let mut f = File::create(target_path).await?;
    let mut chunks = part.stream();
    while let Some(Ok(buf)) = chunks.next().await {
        f.write_all(buf.bytes()).await?;
    }

    Ok(())
}

struct WordCharacters;
impl Distribution<char> for WordCharacters {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        const GEN_STR_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                abcdefghijklmnopqrstuvwxyz\
                0123456789\
                -_~";

        const RANGE: u32 = GEN_STR_CHARSET.len() as u32;
        loop {
            let var = rng.next_u32() >> (32 - 6);
            if var < RANGE {
                return GEN_STR_CHARSET[var as usize] as char;
            }
        }
    }
}

fn generate_random_string(n: usize) -> String {
    let mut rng = rand::thread_rng();

    std::iter::repeat(())
        .map(|()| rng.sample(WordCharacters))
        .take(n)
        .collect()
}

lazy_static! {
    static ref KEYS: Mutex<HashMap<String, String>> = Mutex::new(
        serde_json::from_str(
            &std::fs::read_to_string("keys.json").expect("Could not read keys.json")
        )
        .expect("Could not parse keys.json")
    );
}

fn check_key(key: String) -> bool {
    let keys_guard = KEYS.lock();

    if let Ok(keys) = keys_guard {
        match keys.get(&key) {
            Some(identifier) => {
                info!("{} is uploading some files:", identifier);
                true
            }
            None => false,
        }
    } else {
        false
    }
}
