#[macro_use]
extern crate log;
extern crate libc;
extern crate lru_cache;
extern crate wait_timeout;

use lru_cache::LruCache;

use std::error::Error;
use std::fmt;
use std::io::Write;
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use docker::Container;

mod docker;

pub const PLAYPEN_ENV_VAR_NAME: &'static str = "RUST_PLAYPEN_ENV";

/// Error type holding a description
pub struct StringError(pub String);

impl Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Cache {
    cache: Mutex<LruCache<CacheKey, (ExitStatus, Vec<u8>)>>,
}

#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    cmd: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    input: String,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: Mutex::new(LruCache::new(256)),
        }
    }

    /// Helper method for safely invoking a command inside a playpen
    pub fn exec(&self,
                cmd: &str,
                args: Vec<String>,
                env: Vec<(String, String)>,
                input: String)
                -> io::Result<(ExitStatus, Vec<u8>)> {
        // Build key to look up
        let key = CacheKey {
            cmd: cmd.to_string(),
            args: args,
            env: env,
            input: input,
        };
        let mut cache = self.cache.lock().unwrap();
        if let Some(prev) = cache.get_mut(&key) {
            return Ok(prev.clone())
        }
        drop(cache);

        let container = "ponylang-playpen";

        let container = try!(Container::new(cmd, &key.args, &key.env, container));

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
}

pub enum CompileOutput {
    Asm,
    Llvm,
}

impl CompileOutput {
    pub fn as_opts(&self) -> &'static [&'static str] {
        // We use statics here since the borrow checker complains if we put these directly in the
        // match. Pretty ugly, but rvalue promotion might fix this.
        static ASM: &'static [&'static str] = &["--pass=asm"];
        static LLVM: &'static [&'static str] = &["--pass=ir"];
        match *self {
            CompileOutput::Asm => ASM,
            CompileOutput::Llvm => LLVM,
        }
    }
}

impl FromStr for CompileOutput {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asm" => Ok(CompileOutput::Asm),
            "llvm-ir" => Ok(CompileOutput::Llvm),
            _ => Err(StringError(format!("unknown output format {}", s))),
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
