#![feature(proc_macro_hygiene, decl_macro)]

extern crate hubcaps;
extern crate pony_playground;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_derive;
extern crate tokio;
extern crate url;

use rocket::State;
use rocket::response::NamedFile;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use serde_derive::Deserialize;
use std::io;
use std::path::{PathBuf, Path};
use std::process::Command;
use hubcaps::gists::Gist;
use url::Url;

use pony_playground::*;

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
fn evaluate(request: Json<Evaluate>, playpen: State<Playpen>) -> JsonValue {
    let request = request.0;

    let branch = request.branch.map(|branch| branch.parse().unwrap()).unwrap_or(Branch::Release);

    let (_status, compiler, output) = playpen.evaluate(branch, request.code).unwrap();

    json!({
        "rustc": compiler,
        "program": output,
    })
}

#[derive(Deserialize)]
struct Compile {
    emit: String,
    code: String,
    branch: Option<String>,
}

#[post("/compile.json", data = "<request>")]
fn compile(request: Json<Compile>, playpen: State<Playpen>) -> JsonValue {
    let request = request.0;

    let emit = request.emit.parse().unwrap();
    let branch = request.branch.map(|branch| branch.parse().unwrap()).unwrap_or(Branch::Release);

    let (status, compiler, output) = playpen.compile(branch, request.code, emit).unwrap();

    if status.success() {
        let output = highlight(emit, &output);
        json!({
            "result": output,
        })
    } else {
        json!({
            "error": compiler,
        })
    }
}

#[derive(Deserialize)]
struct CreateGist {
    code: String,
    base_url: String,
    branch: String,
}

fn create_gist(token: String,
               description: String,
               filename: String,
               code: String) -> Gist {
    use tokio::runtime::Runtime;
    use hubcaps::{Credentials, Github};
    use hubcaps::gists::GistOptions;

    let creds = Credentials::Token(token);

    let mut rt = Runtime::new().expect("Unable to create the reactor");
    let github = Github::new("Pony Playground", Some(creds));

    let files = [(filename, code)].iter().cloned().collect();
    let options = GistOptions::builder(files)
        .description(description)
        .public(true)
        .build();

    rt.block_on(github.gists().create(&options)).unwrap()
}

fn update_gist(token: String, id: &str, description: String) -> Gist {
    use tokio::runtime::Runtime;
    use hubcaps::{Credentials, Github};
    use hubcaps::gists::GistOptions;
    use std::collections::HashMap;

    let creds = Credentials::Token(token);

    let mut rt = Runtime::new().expect("Unable to create the reactor");
    let github = Github::new("Pony Playground", Some(creds));

    let options = GistOptions::builder(HashMap::<String, String>::new())
        .description(description)
        .build();

    rt.block_on(github.gists().edit(id, &options)).unwrap()
}

const GIST_FILENAME : &str = "main.pony";
const GIST_DESCRIPTION : &str = "Shared via Pony Playground";

struct GithubToken(String);

#[post("/gist.json", data = "<request>")]
fn gist(request: Json<CreateGist>, token: State<GithubToken>) -> JsonValue {
    let request = request.0;

    let gist = create_gist(
        token.0.clone(), GIST_DESCRIPTION.into(), GIST_FILENAME.into(), request.code);

    let mut url = Url::parse(&request.base_url).unwrap();
    url.query_pairs_mut().append_pair("gist", &gist.id);
    if request.branch != "release" {
        url.query_pairs_mut().append_pair("branch", &request.branch);
    }
    let url = url.into_string();

    update_gist(token.0.clone(), &gist.id, format!("{} ({})", GIST_DESCRIPTION, url));

    json!({
        "gist_id": gist.id,
        "gist_url": gist.html_url,
        "play_url": url,
    })
}

fn main() {
    // Make sure pygmentize is installed before starting the server
    match Command::new("pygmentize").arg("-V").status() {
        Ok(status) if status.success() => (),
        _ => panic!("Cannot find pygmentize."),
    };

    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(_) => panic!("Missing GITHUB_TOKEN environment variable."),
    };

    rocket::ignite()
        .mount("/", routes![index, assets, evaluate, compile, gist])
        .manage(Playpen::new())
        .manage(GithubToken(token))
        .launch();
}
