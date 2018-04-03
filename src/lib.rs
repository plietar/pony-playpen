#[macro_use]
extern crate log;
extern crate libc;
extern crate lru_cache;
extern crate wait_timeout;

use lru_cache::LruCache;

use std::io::Write;
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use docker::Container;

mod docker;

pub struct Playpen {
    cache: Mutex<LruCache<CacheKey, (ExitStatus, Vec<u8>)>>,
}

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    cmd: String,
    args: Vec<String>,
    input: String,
}

impl Playpen {
    pub fn new() -> Playpen {
        Playpen {
            cache: Mutex::new(LruCache::new(256)),
        }
    }

    fn exec(&self,
            cmd: &str,
            args: Vec<String>,
            input: String) -> io::Result<(ExitStatus, Vec<u8>)> {

        // Build key to look up
        let key = CacheKey {
            cmd: cmd.to_string(),
            args: args,
            input: input,
        };
        let mut cache = self.cache.lock().unwrap();
        if let Some(prev) = cache.get_mut(&key) {
            return Ok(prev.clone())
        }
        drop(cache);

        let container = "ponylang-playpen";

        let container = try!(Container::new(cmd, &key.args, &[], container));

        let tuple = try!(container.run(key.input.as_bytes(), Duration::new(5, 0)));
        let (status, mut output, timeout) = tuple;
        if timeout {
            output.extend_from_slice(b"\ntimeout triggered!");
        }
        let mut cache = self.cache.lock().unwrap();
        if status.success() {
            cache.insert(key, (status.clone(), output.clone()));
        }
        Ok((status, output))
    }

    fn parse_output(raw: &[u8]) -> (String, String) {
        let mut split = raw.splitn(2, |b| *b == b'\xff');
        let compiler = String::from_utf8_lossy(split.next().unwrap_or(&[])).into_owned();
        let output = String::from_utf8_lossy(split.next().unwrap_or(&[])).into_owned();

        (compiler, output)
    }

    pub fn evaluate(&self, code: String) -> io::Result<(ExitStatus, String, String)> {
        let (status, raw_output) = self.exec("/usr/local/bin/evaluate.sh", vec![], code)?;
        let (compiler, output) = Self::parse_output(&raw_output);
        Ok((status, compiler, output))
    }

    pub fn compile(&self, code: String, emit: CompileOutput) -> io::Result<(ExitStatus, String, String)> {
        let args = emit.as_opts().iter().map(|x| String::from(*x)).collect();
        let (status, raw_output) = self.exec("/usr/local/bin/compile.sh", args, code)?;
        let (compiler, output) = Self::parse_output(&raw_output);
        Ok((status, compiler, output))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CompileOutput {
    Asm,
    Llvm,
}

impl CompileOutput {
    pub fn as_opts(&self) -> &'static [&'static str] {
        match *self {
            CompileOutput::Asm => &["--pass=asm"],
            CompileOutput::Llvm => &["--pass=ir"],
        }
    }
}

impl FromStr for CompileOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asm" => Ok(CompileOutput::Asm),
            "llvm-ir" => Ok(CompileOutput::Llvm),
            _ => Err(format!("unknown output format {}", s)),
        }
    }
}

/// Highlights compiled rustc output according to the given output format
pub fn highlight(output_format: CompileOutput, output: &str) -> String {
    let lexer = match output_format {
        CompileOutput::Asm => "gas",
        CompileOutput::Llvm => "llvm",
    };

    let mut child = Command::new("pygmentize")
                            .arg("-l")
                            .arg(lexer)
                            .arg("-f")
                            .arg("html")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn().unwrap();
    child.stdin.take().unwrap().write_all(output.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}
