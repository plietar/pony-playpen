#[macro_use]
extern crate log;
extern crate libc;
extern crate wait_timeout;

use std::io::Write;
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::str::FromStr;
use std::time::Duration;

use docker::Container;
pub use branches::Branch;

mod docker;
mod branches;

pub struct Playpen;
impl Playpen {
    pub fn new() -> Playpen {
        Playpen
    }

    fn exec(&self,
            branch: Branch,
            cmd: &str,
            args: Vec<String>,
            input: String) -> io::Result<(ExitStatus, Vec<u8>)> {
        let container = try!(Container::new(cmd, &args, &[], branch.image()));

        let tuple = try!(container.run(input.as_bytes(), Duration::new(10, 0)));
        let (status, mut output, timeout) = tuple;
        if timeout {
            output.extend_from_slice(b"\ntimeout triggered!");
        }
        Ok((status, output))
    }

    fn parse_output(raw: &[u8]) -> (String, String) {
        let mut split = raw.splitn(2, |b| *b == b'\xff');
        let compiler = String::from_utf8_lossy(split.next().unwrap_or(&[])).into_owned();
        let output = String::from_utf8_lossy(split.next().unwrap_or(&[])).into_owned();

        (compiler, output)
    }

    pub fn evaluate(&self, branch: Branch, code: String) -> io::Result<(ExitStatus, String, String)> {
        let (status, raw_output) = self.exec(branch, "/usr/local/bin/evaluate.sh", vec![], code)?;
        let (compiler, output) = Self::parse_output(&raw_output);
        Ok((status, compiler, output))
    }

    pub fn compile(&self, branch: Branch, code: String, emit: CompileOutput) -> io::Result<(ExitStatus, String, String)> {
        let args = emit.as_opts().iter().map(|x| String::from(*x)).collect();
        let (status, raw_output) = self.exec(branch, "/usr/local/bin/compile.sh", args, code)?;
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
