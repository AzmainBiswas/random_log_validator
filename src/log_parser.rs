use std::io::{BufWriter, Write};

use crate::error::{ParseError, Result};

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

pub struct LogValidator {
    total_win: u64,
    total_round: u64,

    line_win: u64,
}

impl LogValidator {
    pub fn new() -> Self {
        Self {
            total_win: 0,
            total_round: 0,
            line_win: 0,
        }
    }

    pub fn validate_single_line(&mut self, line: &[u8]) -> Result<()> {
        self.total_round += 1; // increment the round
        self.line_win = 0;

        let mut line = Line::new(line.trim_ascii());

        if line.peek() == None {
            // TODO: check what to do with the empty line. ignoring for now
            return Ok(());
        }

        // first parse round;
        if self.extract_number(&mut line).is_none() {
            self.report_error(&line, "Expected Round Number.");
            return Err(ParseError::General.into());
        };

        let _ = self.consume_semicolon(&mut line)?;
        self.validate_round(&mut line)?;

        self.total_win += self.line_win;
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
            //TODO: Handle Error properly
            number = number.checked_mul(10)?.checked_add((ch - b'0') as usize)?;
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

    // fn consume_comma(&self, line: &mut Line) {}

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

    //b"15;R:21:26,B:62:170,B:72:170,B:93:182,B:47:164,B:118:172,RS:30:31#0;B:45:101,B:30:110,B:34:107,B:85:118,B:3:100,W:2:3#4;#4\n";
    fn validate_round(&self, line: &mut Line) -> Result<()> {
        let mut total_round_win = 0;
        let mut last_win = 0;
        let mut expected_entry = true;
        while let Some(ch) = line.peek() {
            if expected_entry {
                if !ch.is_ascii_alphabetic() {
                }

            }
            match ch {
                ch if ch.is_ascii_alphabetic() => {
                    //TODO: match code and compare the numbers.
                    let _code = self.extract_code(line).unwrap();
                    let _ = self.consume_colon(line);

                    //TODO: handle error
                    let number1 = self.extract_number(line).unwrap();
                    let _ = self.consume_colon(line);

                    //TODO: handle error
                    let number2 = self.extract_number(line).unwrap();

                    if number1 >= number2 {
                        todo!("Handle ERROR");
                    }
                    println!(">> {}:{}:{}", _code, number1, number2);
                }
                b',' => {
                    let _ = line.next();
                    //TODO: handle error
                    let next = line.peek().unwrap();
                    if !(next.is_ascii_alphabetic()) {
                        todo!("Handle ERROR");
                    }
                }
                b'#' => {
                    //TODO: handle error
                    let prev = line.prev().unwrap();
                    let _ = line.next();
                    //TODO: handle error
                    let win = self.extract_number(line).unwrap();
                    total_round_win += win;

                    if prev == b';' {
                        last_win += win;
                    }

                    if let Some(_) = line.peek() {
                        let _ = self.consume_semicolon(line);
                    }
                }
                _ => {
                    //TODO: handle error
                    todo!()
                }
            }
        }

        if total_round_win != 2 * last_win {
            todo!()
        }

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
