#![feature(plugin, use_extern_macros)]
#![plugin(rocket_codegen)]

extern crate hubcaps;
extern crate pony_playpen;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core as tokio;

use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::Json;
use serde_derive::Deserialize;
use serde_json::{json, json_internal};
use std::collections::HashMap;
use std::io;
use std::path::{PathBuf, Path};
use std::process::Command;
use hubcaps::gists::Gist;

use pony_playpen::*;

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
    branch: Option<String>,
}
#[post("/evaluate.json", data = "<request>")]
fn evaluate(request: Json<Evaluate>, playpen: State<Playpen>) -> Json {
    let request = request.0;

    let branch = request.branch.map(|branch| branch.parse().unwrap()).unwrap_or(Branch::Release);

    let (_status, compiler, output) = playpen.evaluate(branch, request.code).unwrap();

    Json(json!({
        "rustc": compiler,
        "program": output,
    }))
}

#[derive(Deserialize)]
struct Compile {
    emit: String,
    code: String,
    branch: Option<String>,
}

#[post("/compile.json", data = "<request>")]
fn compile(request: Json<Compile>, playpen: State<Playpen>) -> Json {
    let request = request.0;

    let emit = request.emit.parse().unwrap();
    let branch = request.branch.map(|branch| branch.parse().unwrap()).unwrap_or(Branch::Release);

    let (status, compiler, output) = playpen.compile(branch, request.code, emit).unwrap();

    if status.success() {
        let output = highlight(emit, &output);
        Json(json!({
            "result": output,
        }))
    } else {
        Json(json!({
            "error": compiler,
        }))
    }
}

#[derive(Deserialize)]
struct CreateGist {
    code: String,
}

fn create_gist(token: String,
               description: String,
               filename: String,
               code: String) -> Gist {
    use tokio::reactor::Core;
    use hubcaps::{Credentials, Github};
    use hubcaps::gists::{Content, GistOptions};

    let creds = Credentials::Token(token);

    let mut core = Core::new().expect("Unable to create the reactor");
    let github = Github::new("Pony Playground", Some(creds), &core.handle());

    let file = Content {
        filename: None,
        content: code,
    };

    let mut files = HashMap::new();
    files.insert(filename, file);
    let options = GistOptions {
        description: Some(description),
        public: Some(true),
        files,
    };

    core.run(github.gists().create(&options)).unwrap()
}

const GIST_FILENAME : &str = "main.pony";
const GIST_DESCRIPTION : &str = "Shared via Pony Playground";

struct GithubToken(String);

#[post("/gist.json", data = "<request>")]
fn gist(request: Json<CreateGist>, token: State<GithubToken>) -> Json {
    let request = request.0;

    let gist = create_gist(
        token.0.clone(), GIST_DESCRIPTION.into(), GIST_FILENAME.into(), request.code);

    Json(json!({
        "gist_id": gist.id,
        "gist_url": gist.html_url,
    }))
}

fn main() {
    // Make sure pygmentize is installed before starting the server
    Command::new("pygmentize").spawn().unwrap().kill().unwrap();

    let github_token = std::env::var("GITHUB_TOKEN").unwrap();

    rocket::ignite()
        .mount("/", routes![index, assets, evaluate, compile, gist])
        .manage(Playpen::new())
        .manage(GithubToken(github_token))
        .launch();
}
