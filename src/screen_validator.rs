use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::ExitCode,
};

use serde::de::IgnoredAny;
use serde_json;

use crate::{
    ProgressBar,
    error::{ParseError, Result},
};

pub struct ScreenValidator {
    total_round: usize,
    curser: usize,
    pub clear_buf: bool,
}

impl ScreenValidator {
    pub fn new() -> Self {
        Self {
            total_round: 0,
            curser: 0,
            clear_buf: false,
        }
    }

    pub fn capture_screen(&mut self, buf: &[u8]) -> Result<()> {
        self.clear_buf = false;
        if buf[self.curser] == b'=' {
            self.clear_buf = true;
            match self.extract_round_no(&buf[self.curser..]) {
                Err(e) => {
                    let massage = format!("ERROR: unable to find round number. {}", e);
                    self.report_error(buf, &massage, 0);
                }
                _ => {}
            };
        } else {
            self.curser = buf.len();
        }

        if !self.clear_buf {
            return Ok(());
        }

        match serde_json::from_slice::<IgnoredAny>(&buf[0..self.curser]) {
            Err(e) => {
                let massage = format!("Unable to parse screen {}", e);
                self.report_error(buf, &massage, e.line());
                return Err(ParseError::General.into());
            }
            _ => {}
        };

        self.curser = 0;
        self.total_round += 1;
        Ok(())
    }

    fn extract_round_no(&self, input: &[u8]) -> Result<u32> {
        let marker = b"Round";
        let pos = input
            .windows(marker.len())
            .position(|window| window == marker)
            .ok_or_else(|| io::Error::other("missing Round"))?;

        let tail = &input[pos + marker.len()..];

        if !tail.first().is_some_and(u8::is_ascii_whitespace) {
            return Err(io::Error::other("expected whitespace after Round").into());
        }

        let start = tail
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .ok_or_else(|| io::Error::other("missing number after Round"))?;

        let digits = &tail[start..];
        let len = digits.iter().take_while(|b| b.is_ascii_digit()).count();

        if len == 0 {
            return Err(io::Error::other("missing number after Round").into());
        }

        let number = std::str::from_utf8(&digits[..len])
            .map_err(|_| io::Error::other("invalid number"))?
            .parse::<u32>()
            .map_err(|_| io::Error::other("number out of range"))?;

        Ok(number)
    }

    pub fn report_error(&self, buf: &[u8], massage: &str, line: usize) {
        let stderr = std::io::stderr();
        let mut err = BufWriter::new(stderr.lock());

        err.write_all(b"\x1b[31mERROR:\x1b[0m ")
            .expect("ERROR: unable to write to stderr");
        let mut lines = 0usize;
        for (_, ch) in buf.iter().enumerate() {
            if *ch == b'\n' {
                lines += 1;
            }

            if lines + 1 == line {
                err.write_all(b"\x1b[31m")
                    .expect("ERROR: unable to write to stderr");
            }
            err.write(&[*ch]).expect("ERROR: unable to write to stderr");
            if lines + 1 == line {
                err.write_all(b"\x1b[0m")
                    .expect("ERROR: unable to write to stderr");
            }
        }
        err.write(b"\n").expect("ERROR: unable to write to stderr");
        write!(err, "\x1b[31mERROR:\x1b[0m {}\n", massage)
            .expect("ERROR: unable to write to stderr");

        err.flush().expect("ERROR: unable to flush stderr");
    }
}

pub fn validate_screens(file_path: &str, buf_size: usize) -> ExitCode {
    let file_size: usize = fs::metadata(file_path)
        .expect("ERROR: unable to read the the metadata of the file")
        .len() as usize;
    let f = File::open(file_path).expect("ERROR: unable to open the file.");
    let mut reader = BufReader::with_capacity(buf_size, f);

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let mut screen_validator = ScreenValidator::new();
    let mut progress = ProgressBar::new(file_size, 50);

    let mut buf = Vec::<u8>::with_capacity(1 << 10);
    let mut processed = 0usize;
    while let Ok(size) = reader.read_until(b'\n', &mut buf) {
        if size == 0 {
            break;
        }
        processed += size;

        match screen_validator.capture_screen(&buf) {
            Err(_) => {
                return ExitCode::FAILURE;
            }
            _ => {}
        }

        progress.process_bar(&mut out, processed, 2);

        if screen_validator.clear_buf {
            buf.clear(); //do not clear for every loop.
        }
    }
    write!(out, "Total Round {}", screen_validator.total_round).expect("ERROR: unable to write to stdout");
    out.flush().expect("ERROR: unable to flush");
    ExitCode::SUCCESS
}
