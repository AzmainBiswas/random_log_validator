use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::{ExitCode};

use crate::error::Result;
use crate::log_parser::LogValidator;

mod log_parser;
mod error;

fn main() -> ExitCode {
    let file_path = "./tests/random_logs.txt";
    return validate_random_log(file_path);
}

fn validate_random_log(file_path: &str) -> ExitCode {
  let file_size: usize = fs::metadata(file_path).expect("ERROR: unable to read the the metadata of the file").len() as usize;
  let f = File::open(file_path).expect("ERROR: unable to open the file.");

  let mut reader = BufReader::new(f);

  let stdout = io::stdout();
  let mut out = BufWriter::new(stdout.lock());

  let mut buf = Vec::<u8>::with_capacity(1024);
  let mut processed: usize = 0;
  while let Ok(size) = reader.read_until(b'\n', &mut buf) {
      processed += size;
      if size == 0 {
          break;
      }
      write!(out, "\r\x1b[2KProcessing ").expect("Unable to write to file");
      out.write_all(&buf).expect("Unable to write to file.");

      let pct = (processed * 100) / file_size;
      let filled = (processed * 30) / file_size;
      let bar = "=".repeat(filled) + &" ".repeat(30 - filled);
      write!(out, "\r[{}] {}%", bar, pct).expect("Unable to write to file.");

      buf.clear();
      out.flush().expect("Unable to flush the stdout");
  }
  ExitCode::SUCCESS
}

struct ProgressBar {
  total: usize,
  current: usize,
}

impl ProgressBar {
  fn new(total: usize) -> Self {
    Self {
      total: total,
      current: 0,
    }
  }
}
