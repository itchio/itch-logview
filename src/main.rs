use chrono::NaiveDateTime;
use clap::{App, Arg};
use colored::{Color, Colorize};
use serde::Deserialize;
use std::fmt;

mod logwatcher;
use logwatcher::LogWatcher;

// Sample log lines:
// {"time":1552019857416,"level":30,"msg":"Scheduling next game update check for Thu Mar 07 2019 23:58:57 GMT-0500 (Eastern Standard Time)","name":"m/reac/updater"}
// {"time":1552059977666,"level":30,"msg":"Regularly scheduled check for game updates...","name":"m/reac/updater"}
// {"time":1552059977672,"level":30,"msg":"Scheduling next game update check for Fri Mar 08 2019 11:09:01 GMT-0500 (Eastern Standard Time)","name":"m/reac/updater"}
// "Time" is in milliseconds

#[derive(Deserialize)]
struct LogLine {
    time: Time,
    level: Level,
    msg: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct Time(pub f64);

impl Time {
    fn time_format() -> &'static str {
        "%Y-%m-%d %H:%M:%S"
    }

    fn timestamp(&self) -> NaiveDateTime {
        let seconds = (self.0 / 1000.0) as i64;
        let millis = self.0 - (seconds as f64 * 1000.0);
        let nanos = millis * 1000.0;
        NaiveDateTime::from_timestamp(seconds, nanos as u32)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.timestamp().format(Self::time_format()))
    }
}

#[derive(Deserialize)]
struct Level(pub i32);

impl Level {
    fn color(&self) -> Color {
        match self.0 {
            // trace, debug
            0...20 => Color::Blue,
            // info
            21...30 => Color::BrightWhite,
            // warn
            31...40 => Color::Yellow,
            // error, fatal
            _ => Color::Red,
        }
    }

    fn name(&self) -> &str {
        match self.0 {
            10 => "trace",
            20 => "debug",
            30 => "info",
            40 => "warn",
            50 => "error",
            60 => "fatal",
            _ => "",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

fn main() {
    color_backtrace::install();

    let matches = App::new("logview")
        .version("1.0")
        .author("Amos Wenger <amoswenger@gmail.com>")
        .about("Pretty-prints itch v25 logs")
        .arg(
            Arg::with_name("log")
                .value_name("FILE")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("follow")
                .short("f")
                .long("follow")
                .help("Have 'tail -f'-like behavior"),
        )
        .get_matches();

    let log_file = matches.value_of("log").unwrap();
    let follow = matches.is_present("follow");

    let on_line = &|line: String| {
        let line: LogLine = match serde_json::from_str(&line) {
            Ok(line) => line,
            Err(_) => return,
        };

        print!(
            "{prefix}",
            prefix = format!(
                "{time} {level}{name}",
                time = line.time,
                level = line.level,
                name = if let Some(name) = line.name {
                    format!(" ({})", name)
                } else {
                    "".into()
                }
                .green(),
            )
            .white(),
        );

        if line.msg.contains("\n") {
            println!("{}", " {".white());
            for msg_line in line.msg.split("\n") {
                println!(
                    "{indent}{msg}",
                    indent = " ".repeat(8),
                    msg = msg_line.color(line.level.color())
                );
            }
            println!("{}", "}".white());
        } else {
            println!(" {msg}", msg = line.msg.color(line.level.color()));
        }
    };

    if follow {
        let mut log_watcher = LogWatcher::register(log_file.to_string()).unwrap();
        log_watcher.watch(on_line);
    } else {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        let f = File::open(log_file).unwrap();
        let f = BufReader::new(f);
        for line in f.lines() {
            on_line(line.unwrap());
        }
    }
}
