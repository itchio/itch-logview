// ripped from https://crates.io/crates/logwatcher (also MIT-licensed)
// and replaced inode comparison with ctime (file creation time) comparison
// so it's cross-platform.

// not happy with that code, but didn't want to spend any more time on it.

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub struct LogWatcher {
    filename: String,
    created: SystemTime,
    pos: u64,
    reader: BufReader<File>,
    finish: bool,
}

impl LogWatcher {
    pub fn register(filename: String) -> Result<LogWatcher, io::Error> {
        let f = match File::open(filename.clone()) {
            Ok(x) => x,
            Err(err) => return Err(err),
        };

        let metadata = match f.metadata() {
            Ok(x) => x,
            Err(err) => return Err(err),
        };

        let mut reader = BufReader::new(f);

        let pos = {
            let len = metadata.len();
            let initial_bytes = 1024;
            if len > initial_bytes {
                // show last few lines of log
                len - initial_bytes
            } else {
                // start from beginning if too short
                0
            }
        };

        reader.seek(SeekFrom::Start(pos)).unwrap();
        Ok(LogWatcher {
            filename: filename,
            created: metadata
                .created()
                .unwrap_or_else(|_| SystemTime::UNIX_EPOCH),
            pos: pos,
            reader: reader,
            finish: false,
        })
    }

    fn reopen_if_log_rotated<F: ?Sized>(&mut self, callback: &F)
    where
        F: Fn(String),
    {
        loop {
            match File::open(self.filename.clone()) {
                Ok(x) => {
                    let f = x;
                    let metadata = match f.metadata() {
                        Ok(m) => m,
                        Err(_) => {
                            sleep(Duration::new(1, 0));
                            continue;
                        }
                    };

                    let created = metadata
                        .created()
                        .unwrap_or_else(|_| SystemTime::UNIX_EPOCH);
                    if created != self.created {
                        self.finish = true;
                        self.watch(callback);
                        self.finish = false;
                        println!("reloading log file");
                        self.reader = BufReader::new(f);
                        self.pos = 0;
                        self.created = created;
                    } else {
                        sleep(Duration::new(1, 0));
                    }
                    break;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        sleep(Duration::new(1, 0));
                        continue;
                    }
                }
            };
        }
    }

    pub fn watch<F: ?Sized>(&mut self, callback: &F)
    where
        F: Fn(String),
    {
        loop {
            let mut line = String::new();
            let resp = self.reader.read_line(&mut line);
            match resp {
                Ok(len) => {
                    if len > 0 {
                        self.pos += len as u64;
                        self.reader.seek(SeekFrom::Start(self.pos)).unwrap();
                        callback(line.replace("\n", ""));
                        line.clear();
                    } else {
                        if self.finish {
                            break;
                        } else {
                            self.reopen_if_log_rotated(callback);
                            self.reader.seek(SeekFrom::Start(self.pos)).unwrap();
                        }
                    }
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }
}
