use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::ExitCode,
};

use crate::{
    ProgressBar,
    error::{ParseError, Result},
};

pub struct Line<'a> {
    pub data: &'a [u8],
    pub curser: usize,
}

impl<'a> Line<'a> {
    pub fn new(line: &'a [u8]) -> Self {
        Self {
            data: line,
            curser: 0,
        }
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.curser < self.data.len() {
            let byte = self.data[self.curser];
            self.curser += 1;
            Some(byte)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<u8> {
        if self.curser < self.data.len() {
            let byte = self.data[self.curser];
            Some(byte)
        } else {
            None
        }
    }

    pub fn prev(&self) -> Option<u8> {
        if self.curser >= 1 {
            return Some(self.data[self.curser - 1]);
        }
        None
    }

    pub fn get_curser(&self) -> usize {
        self.curser
    }
}

enum Expected {
    Entry,
    EntryOrWin,
    CommaOrPound,
    None,
}

pub struct LogValidator {
    pub total_win: u64,
    pub total_round: u64,
}

impl LogValidator {
    pub fn new() -> Self {
        Self {
            total_win: 0,
            total_round: 0,
        }
    }

    pub fn validate_single_line(&mut self, line: &[u8]) -> Result<()> {
        let mut line = Line::new(line.trim_ascii());
        if line.peek() == None {
            return Ok(());
        }

        self.total_round += 1; // increment the round

        // first parse round;
        if self.extract_number(&mut line).is_none() {
            self.report_error(&line, "Expected Round Number.");
            return Err(ParseError::General.into());
        };

        self.consume_semicolon(&mut line)?;
        self.validate_round(&mut line)?;
        Ok(())
    }

    fn extract_number(&self, line: &mut Line) -> Option<usize> {
        let mut number: usize = 0;
        let mut found_digit = false;
        while let Some(ch) = line.peek() {
            if !ch.is_ascii_digit() {
                break;
            }

            found_digit = true;
            number = number
                .checked_mul(10)
                .and_then(|v| v.checked_add((ch - b'0') as usize))?;
            let _ = line.next();
        }

        if found_digit { Some(number) } else { None }
    }

    fn extract_code<'a>(&self, line: &mut Line<'a>) -> Option<&'a str> {
        let start = line.get_curser();
        let mut len = 0;
        let mut found_code = false;
        while let Some(ch) = line.peek() {
            if !ch.is_ascii_alphabetic() {
                break;
            }

            found_code = true;
            len += 1;
            let _ = line.next();
        }

        if !found_code {
            return None;
        }

        std::str::from_utf8(&line.data[start..(start + len)]).ok()
    }

    fn consume_semicolon(&self, line: &mut Line) -> Result<()> {
        self.consume_char(line, b';')
    }

    fn consume_colon(&self, line: &mut Line) -> Result<()> {
        self.consume_char(line, b':')
    }

    fn consume_char(&self, line: &mut Line, ch: u8) -> Result<()> {
        match line.next() {
            Some(found) if found == ch => Ok(()),
            Some(found) => {
                line.curser -= 1;
                self.report_error(
                    &line,
                    &format!(
                        "Expected \"{}\" but found \"{}\"",
                        ch as char, found as char
                    ),
                );
                return Err(ParseError::General.into());
            }
            None => {
                line.curser -= 1;
                self.report_error(&line, "Unexpected line end.");
                return Err(ParseError::General.into());
            }
        }
    }

    fn validate_round(&mut self, line: &mut Line) -> Result<()> {
        let mut total_round_win = 0;
        let mut last_win = 0;
        let mut expected = Expected::Entry;
        while let Some(ch) = line.peek() {
            match expected {
                Expected::Entry if !ch.is_ascii_alphabetic() => {
                    self.report_error(line, "Expected a Entry Code");
                    return Err(ParseError::General.into());
                }
                Expected::EntryOrWin if !ch.is_ascii_alphabetic() && ch != b'#' => {
                    self.report_error(line, "Expected a Entry Code or Win");
                    return Err(ParseError::General.into());
                }
                Expected::CommaOrPound if ch != b',' && ch != b'#' => {
                    self.report_error(line, "Expected \",\" or \"#\" here.");
                    return Err(ParseError::General.into());
                }
                _ => {}
            }

            //TODO: for now it is non to avoid any error.
            expected = Expected::None;

            match ch {
                ch if ch.is_ascii_alphabetic() => {
                    let Some(code) = self.extract_code(line) else {
                        self.report_error(line, "Expected code there.");
                        return Err(ParseError::General.into());
                    };

                    self.consume_colon(line)?;

                    let Some(number1) = self.extract_number(line) else {
                        self.report_error(line, "Expected a Number here");
                        return Err(ParseError::General.into());
                    };
                    self.consume_colon(line)?;

                    let Some(number2) = self.extract_number(line) else {
                        self.report_error(line, "Expected a Number here");
                        return Err(ParseError::General.into());
                    };

                    if number1 >= number2 {
                        self.report_error(
                            line,
                            &format!("Number in code \"{}\" is not correct.", code),
                        );
                        return Err(ParseError::General.into());
                    }

                    expected = Expected::CommaOrPound;
                }
                b',' => {
                    let _ = line.next();
                    expected = Expected::Entry;
                }
                b'#' => {
                    let Some(prev) = line.prev() else {
                        self.report_error(line, "Unexpected Begenning of line");
                        return Err(ParseError::General.into());
                    };
                    let _ = line.next();
                    let Some(win) = self.extract_number(line) else {
                        self.report_error(line, "Expected number here.");
                        return Err(ParseError::General.into());
                    };
                    total_round_win += win;

                    if prev == b';' {
                        last_win += win;
                    }
                    // TODO: determine what to expect. requirement is not clear for this.
                }
                b';' => {
                    let _ = self.consume_semicolon(line);
                    expected = Expected::EntryOrWin;
                }
                _ => {
                    self.report_error(line, &format!("Unexpected chracter \"{}\".", ch as char));
                    return Err(ParseError::General.into());
                }
            }
        }

        //TODO: add a flag wheather to check the wins or not.
        if total_round_win != 2 * last_win {
            self.report_error(
                line,
                &format!("Wins are not matching please check the win."),
            );
            return Err(ParseError::General.into());
        }

        self.total_win += last_win as u64;

        Ok(())
    }

    fn report_error(&self, line: &Line, massage: &str) {
        let stderr = std::io::stderr();
        let mut err = BufWriter::new(stderr.lock());

        err.write_all(b"\x1b[31mERROR:\x1b[0m ")
            .expect("ERROR: unable to write to stderr");
        for (i, ch) in line.data.iter().enumerate() {
            if i == line.curser {
                err.write_all(b"\x1b[31m")
                    .expect("ERROR: unable to write to stderr");
            }
            err.write(&[*ch]).expect("ERROR: unable to write to stderr");
            if i == line.curser {
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

pub fn validate_random_log(file_path: &str, buf_size: usize) -> ExitCode {
    let file_size: usize = fs::metadata(file_path)
        .expect("ERROR: unable to read the the metadata of the file")
        .len() as usize;
    let f = File::open(file_path).expect("ERROR: unable to open the file.");
    let mut reader = BufReader::with_capacity(buf_size, f);

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let mut processed: usize = 0;
    let mut progress = ProgressBar::new(file_size, 50);

    let mut log_validator = LogValidator::new();

    let mut buf = Vec::<u8>::with_capacity(1024);
    while let Ok(size) = reader.read_until(b'\n', &mut buf) {
        if size == 0 {
            break;
        }
        // out.write_all(b"\x1b[2KProcessing: ").expect("ERROR: unable to write to stdout.");
        // out.write_all(&buf).expect("ERROR: unable to write to stdout.");
        match log_validator.validate_single_line(&buf) {
            Err(_) => {
                return ExitCode::FAILURE;
            }
            _ => {}
        }

        processed += size;
        progress.process_bar(&mut out, processed, 2);

        buf.clear();
    }

    write!(out, "Total Round: {}\n", log_validator.total_round)
        .expect("ERROR: unable to write to stdout.");
    write!(out, "Total Win: {}\n", log_validator.total_win)
        .expect("ERROR: unable to write to stdout.");
    out.flush().expect("Unable to flush the stdout");
    ExitCode::SUCCESS
}
