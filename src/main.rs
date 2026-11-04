use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::ExitCode;

use crate::log_parser::LogValidator;
use crate::error::Result;

mod log_parser;
mod error;

fn main() -> ExitCode {
    let mut log_parser = LogValidator::new();
    let line = b"15;R:21:26,B:62:170,B:72:170,B:93:182,B:47:164,B:118:172,RS:30:31#0;B:45:101,B:30:110,B:34:107,B:85:118,B:3:100,W:2:3#4;#4\n";
    // let line = b"81;R:4:26,B:146:214,B:179:205,B:70:207,B:152:209,B:107:207,W:2:3#45;SN:99739:100001,HR:6:55,F:167:204,F:171:205,F:35:196,F:51:211,F:17:204#16;HR:12:55,F:190:204,F:41:205,F:168:196,F:32:211,F:15:204,J:1:24#16;HR:46:55,F:176:203,F:44:207,F:150:200,F:125:220,F:142:203,J:5:24#24;F:50:203,F:129:207,F:51:200,F:53:220,F:178:203,J:17:24#25;F:117:203,F:118:207,F:12:200,F:30:220,F:159:203,J:14:24#30;F:110:203,F:138:207,F:53:200,F:203:220,F:56:203,J:5:22#128;F:47:203,F:60:207,F:8:200,F:69:220,F:74:203,J:12:24,J:17:21#112;F:201:203,F:28:207,F:76:200,F:129:220,F:10:203,J:20:24,J:5:21#40;#436";
    if let Err(_) = log_parser.validate_single_line(line) {
        return ExitCode::FAILURE
    }

    ExitCode::SUCCESS
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
