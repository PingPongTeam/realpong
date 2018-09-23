use rocket::config::{Config, Environment};
use rocket::response::NamedFile;
use rocket::State;
use std::path::{Path, PathBuf};
use std::thread;

struct FsServerConfig {
    http_root: String,
}

#[get("/")]
fn index(state: State<FsServerConfig>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&state.http_root).join("index.html")).ok()
}

#[get("/<file..>")]
fn files(file: PathBuf, state: State<FsServerConfig>) -> Option<NamedFile> {
    match file.extension() {
        Some(ext) if ext == "json" => return None,
        _ => {}
    }
    NamedFile::open(Path::new(&state.http_root).join(file)).ok()
}

pub fn start(http_root: String, http_port: u16) -> std::thread::JoinHandle<()> {
    let config = FsServerConfig {
        http_root: http_root,
    };

    let rocket_config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(http_port)
        .workers(12)
        .unwrap();

    println!(
        "Serving files from '{}' at port {}",
        &config.http_root, http_port
    );
    thread::spawn(move || {
        // For serving static files. Will run forever.
        rocket::custom(rocket_config, false)
            .mount("/", routes![index, files])
            .manage(config)
            .launch();
    })
}
