use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub time: DateTime<Utc>,
    pub user: String,
    pub url: String,
}

pub struct Log {
    state: Vec<LogEntry>,
    file_path: PathBuf,
}

pub struct LogIter<'t> {
    log: &'t Log,
    ptr: usize,
}

impl<'t> Iterator for LogIter<'t> {
    type Item = &'t LogEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.log.state.len() {
            None
        } else {
            let val = Some(&self.log.state[self.ptr]);
            self.ptr += 1;
            val
        }
    }
}

impl Log {
    pub fn new(file: impl AsRef<Path>) -> Result<Self> {
        let mut s = Self {
            file_path: file.as_ref().to_path_buf(),
            state: Vec::new(),
        };
        s.reload()?;
        Ok(s)
    }

    pub fn reload(&mut self) -> Result<()> {
        let f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.file_path)?;
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(f);
        let res: Result<Vec<_>> = reader
            .deserialize()
            .map(|r| r.map_err(|e| e.into()))
            .collect();
        self.state = res?;
        Ok(())
    }

    pub fn _iter_from_user<T: TimeZone>(&self, start: DateTime<T>, user: Option<&str>) -> LogIter {
        LogIter {
            log: self,
            ptr: self
                .state
                .iter()
                .enumerate()
                .find(|(_n, e)| e.time > start && user.map(|u| u == e.user).unwrap_or(true))
                .map(|(n, _e)| n)
                .unwrap_or(self.state.len() + 1)
                .saturating_sub(1),
        }
    }

    pub fn iter_from_user<T: TimeZone>(&self, start: DateTime<T>, user: &str) -> LogIter {
        self._iter_from_user(start, Some(user))
    }

    pub fn log<T: TimeZone>(&mut self, time: DateTime<T>, user: &str, url: &str) -> Result<()> {
        let f = File::options()
            .append(true)
            .truncate(false)
            .create(true)
            .open(&self.file_path)?;

        let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(f);
        let entry = LogEntry {
            time: time.to_utc(),
            user: user.into(),
            url: url.into(),
        };

        // log to disk
        writer.serialize(&entry)?;
        writer.flush()?;

        // log to memory
        self.state.push(entry);

        Ok(())
    }
}
