#![feature(plugin, use_extern_macros)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

extern crate pony_playpen;

use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::Json;
use serde_derive::Deserialize;
use serde_json::{json, json_internal};
use std::io;
use std::path::{PathBuf, Path};
use std::process::Command;

use pony_playpen::*;

const ENV: &'static str = "web";
fn base_env() -> Vec<(String, String)> {
    vec![(PLAYPEN_ENV_VAR_NAME.into(), ENV.into())]
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/web.html")
}

#[get("/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[derive(Deserialize)]
struct Evaluate {
    code: String,
}
#[post("/evaluate.json", data = "<request>")]
fn evaluate(request: Json<Evaluate>, cache: State<Cache>) -> Json {
    let request = request.0;

    let args = vec![];
    let (_status, output) = cache.exec(
        "/usr/local/bin/evaluate.sh", args, base_env(), request.code
    ).unwrap();

    let mut split = output.splitn(2, |b| *b == b'\xff');
    let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

    let mut response = json!({
        "rustc": rustc,
    });

    if let Some(program) = split.next() {
        let output = String::from_utf8_lossy(program).into_owned();
        response["program"] = json!(output);
    }

    Json(response)
}

#[derive(Deserialize)]
struct Compile {
    emit: Option<String>,
    code: String,
}

#[post("/compile.json", data = "<request>")]
fn compile(request: Json<Compile>, cache: State<Cache>) -> Json {
    let request = request.0;

    let emit = request.emit
        .map(|emit| emit.parse())
        .unwrap_or(Ok(CompileOutput::Asm))
        .unwrap();

    let mut args = vec![];
    for opt in emit.as_opts() {
        args.push(String::from(*opt));
    }

    let (_status, output) = cache.exec(
        "/usr/local/bin/compile.sh", args, base_env(), request.code
    ).unwrap();

    let mut split = output.splitn(2, |b| *b == b'\xff');
    let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

    match split.next() {
        Some(program_out) => {
            // Compilation succeeded
            let output = highlight(emit, &String::from_utf8_lossy(program_out).into_owned());
            Json(json!({
                "result": output,
            }))
        }
        None => {
            Json(json!({
                "error": rustc,
            }))
        }
    }
}

fn main() {
    // Make sure pygmentize is installed before starting the server
    Command::new("pygmentize").spawn().unwrap().kill().unwrap();

    rocket::ignite()
        .mount("/", routes![index, assets, evaluate, compile])
        .manage(Cache::new())
        .launch();
}
