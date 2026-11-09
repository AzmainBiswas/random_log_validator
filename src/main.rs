use std::env;
use std::io::{BufWriter, Write};
use std::process::ExitCode;

use crate::log_validator::validate_random_log;
use crate::screen_validator::validate_screens;

mod error;
mod log_validator;
mod screen_validator;

fn usage(program: &str) {
    println!("USAGE: {} <OPTIONS> [file_path]", program);
    println!("     OPTIONS:");
    println!("             -r    validate random log file.");
    println!("             -s    validate screen log file.");
    println!("             -h    help");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(option) = args.get(1) else {
        eprintln!("ERROR: provide a option");
        usage(args.get(0).unwrap());
        return ExitCode::FAILURE;
    };
    let Some(file_path) = args.get(2) else {
        eprintln!("ERROR: provide a file path");
        usage(args.get(0).unwrap());
        return ExitCode::FAILURE;
    };

    match option.as_str() {
        "-h" => {
            usage(args.get(0).unwrap());
        },
        "-r" => {
            return validate_random_log(&file_path, 1 << 20);
        },
        "-s" => {
            return validate_screens(file_path, 1 << 20);
        },
        _ => {
            eprintln!("ERROR: provide a valid option");
            usage(args.get(0).unwrap());
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}

struct ProgressBar {
    bar_size: usize,

    total: usize,
    current: usize,
    last_persentage: i32,
}

impl ProgressBar {
    fn new(total: usize, bar_size: usize) -> Self {
        Self {
            bar_size: bar_size,
            total: total,
            current: 0,
            last_persentage: -1,
        }
    }

    fn process_bar(
        &mut self,
        out: &mut BufWriter<std::io::StdoutLock>,
        current: usize,
        precision: usize,
    ) {
        self.current = current;

        let scale = 10usize.pow(precision as u32);
        let pct = self.current * 100 * scale / self.total;
        if self.last_persentage >= pct as i32 {
            return;
        }

        let fill = self.current * self.bar_size / self.total;
        let bar = "=".repeat(fill) + &" ".repeat(self.bar_size - fill);
        write!(
            out,
            "\x1b[2KProgress [{b}] {p:.precision$}%\r",
            b = bar,
            p = pct as f32 / scale as f32
        )
        .expect("ERROR: unable to write to stdout");
        if pct == 100 * scale {
            write!(out, "\n").expect("ERROR: unable to write to stdout");
        }
        out.flush().expect("ERROR: unable to flush the stdout.");
        self.last_persentage = pct as i32;
    }
}
