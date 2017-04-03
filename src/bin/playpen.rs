#[macro_use] extern crate iron;
extern crate env_logger;
extern crate hyper;
extern crate router;
extern crate pony_playpen;
extern crate rustc_serialize;
extern crate staticfile;
extern crate unicase;

use std::env;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use hyper::header;
use iron::headers;
use iron::method::Method;
use iron::middleware::{BeforeMiddleware, AfterMiddleware};
use iron::modifiers::Header;
use iron::typemap;
use iron::prelude::*;
use iron::status;
use router::Router;
use pony_playpen::*;
use rustc_serialize::json;
use staticfile::Static;
use unicase::UniCase;

const ENV: &'static str = "web";

fn base_env() -> Vec<(String, String)> {
    vec![(PLAYPEN_ENV_VAR_NAME.into(), ENV.into())]
}

#[derive(Clone, Debug)]
struct XXssProtection(bool);

impl header::Header for XXssProtection {
    fn header_name() -> &'static str {
        "X-XSS-Protection"
    }

    fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<Self> {
        if raw.len() == 1 {
            let line = &raw[0];
            if line.len() == 1 {
                let byte = line[0];
                match byte {
                    b'1' => return Ok(XXssProtection(true)),
                    b'0' => return Ok(XXssProtection(false)),
                    _ => ()
                }
            }
        }
        Err(hyper::Error::Header)
    }
}

impl header::HeaderFormat for XXssProtection {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 {
            f.write_str("1")
        } else {
            f.write_str("0")
        }
    }
}

fn index(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok,
                       Path::new("static/web.html"),
                       Header(XXssProtection(false)))))
}

/// The JSON-encoded request sent to `evaluate.json`.
#[derive(RustcDecodable)]
struct EvaluateReq {
    separate_output: Option<bool>,
    code: String,
}

fn evaluate(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));

    let data: EvaluateReq = itry!(json::decode(&body));
    let separate_output = data.separate_output.unwrap_or(false);

    let cache = req.extensions.get::<AddCache>().unwrap();
    let (_status, output) = itry!(cache.exec("/usr/local/bin/evaluate.sh",
                                             vec![], base_env(), data.code));

    let mut obj = json::Object::new();
    if separate_output {
        // {"rustc": "...", "program": "..."}
        let mut split = output.splitn(2, |b| *b == b'\xff');
        let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

        obj.insert(String::from("rustc"), json::Json::String(rustc));

        if let Some(program) = split.next() {
            // Compilation succeeded
            let output = String::from_utf8_lossy(program).into_owned();
            obj.insert(String::from("program"), json::Json::String(output));
        }
    } else {
        // {"result": "...""}
        let result = output.splitn(2, |b| *b == b'\xff')
                           .map(|sub| String::from_utf8_lossy(sub).into_owned())
                           .collect::<String>();

        obj.insert(String::from("result"), json::Json::String(result));
    }

    Ok(Response::with((status::Ok, format!("{}", json::Json::Object(obj)))))
}

#[derive(RustcDecodable)]
struct CompileReq {
    emit: Option<String>,
    code: String,
}

fn compile(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    itry!(req.body.read_to_string(&mut body));

    let data: CompileReq = itry!(json::decode(&body));
    let emit = itry!(data.emit.map(|emit| emit.parse()).unwrap_or(Ok(CompileOutput::Asm)));

    let mut args = vec![];
    for opt in emit.as_opts() {
        args.push(String::from(*opt));
    }

    let cache = req.extensions.get::<AddCache>().unwrap();
    let (_status, output) = itry!(cache.exec("/usr/local/bin/compile.sh",
                                             args,
                                             base_env(),
                                             data.code));
    let mut split = output.splitn(2, |b| *b == b'\xff');
    let rustc = String::from_utf8(split.next().unwrap().into()).unwrap();

    let mut obj = json::Object::new();
    match split.next() {
        Some(program_out) => {
            // Compilation succeeded
            let output = highlight(emit,
                                   &String::from_utf8_lossy(program_out).into_owned());
            obj.insert(String::from("result"), json::Json::String(output));
        }
        None => {
            obj.insert(String::from("error"), json::Json::String(rustc));
        }
    }

    Ok(Response::with((status::Ok, format!("{}", json::Json::Object(obj)))))
}

// This is neat!
struct EnablePostCors;
impl AfterMiddleware for EnablePostCors {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        Ok(res.set(Header(headers::AccessControlAllowOrigin::Any))
              .set(Header(headers::AccessControlAllowMethods(
                  vec![Method::Post,
                       Method::Options])))
              .set(Header(headers::AccessControlAllowHeaders(
                  vec![UniCase(String::from("Origin")),
                       UniCase(String::from("Accept")),
                       UniCase(String::from("Content-Type"))]))))
    }
}

struct AddCache {
    cache: Arc<Cache>,
}

impl typemap::Key for AddCache { type Value = Arc<Cache>; }

impl BeforeMiddleware for AddCache {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<AddCache>(self.cache.clone());
        Ok(())
    }
}

fn main() {
    env_logger::init().unwrap();

    // Make sure pygmentize is installed before starting the server
    Command::new("pygmentize").spawn().unwrap().kill().unwrap();

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/:path", Static::new("static"), "static");
    router.post("/evaluate.json", evaluate, "evaluate");
    router.post("/compile.json", compile, "compile");

    // Use our router as the middleware, and pass the generated response through `EnablePostCors`
    let mut chain = Chain::new(router);
    chain.link_before(AddCache { cache: Arc::new(Cache::new()) });
    chain.link_after(EnablePostCors);

    let addr = env::args().skip(1).next().unwrap_or("127.0.0.1".to_string());
    let addr = (&addr[..], 8080);
    println!("listening on {:?}", addr);
    Iron::new(chain).http(addr).unwrap();
}
