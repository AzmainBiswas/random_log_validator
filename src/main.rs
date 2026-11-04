use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use crate::log_parser::LogParser;

mod log_parser;

pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let mut log_parser = LogParser::new();
    let line = b"15;R:21:26,B:62:170,B:72:170,B:93:182,B:47:164,B:118:172,RS:30:31#0;B:45:101,B:30:110,B:34:107,B:85:118,B:3:100,W:2:3#4;#4\n";
    log_parser.parse_single_line(line)?;

    Ok(())
}

fn main2() -> Result<()> {
    let file_path = "./tests/random_logs-10k.txt";
    let file_size: usize = fs::metadata(file_path)?.len() as usize;
    let f = File::open(file_path)?;

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
        write!(out, "\r\x1b[2KProcessing ")?;
        out.write_all(&buf)?;

        let pct = (processed * 100) / file_size;
        let filled = (processed * 30) / file_size;
        let bar = "=".repeat(filled) + &" ".repeat(30 - filled);
        write!(out, "\r[{}] {}%", bar, pct)?;

        buf.clear();
        out.flush()?;
    }


    Ok(())
}
