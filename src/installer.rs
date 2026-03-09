use alloc::alloc::alloc;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::{self, Vec};

use crate::*;

enum FileStatus {
    /// Less than 4 bytes received, cannot get file name length.
    Waiting,
    /// Received file name length, waiting for the file name to fully arrive.
    NameLen(u32),
    /// Received file name, waiting for the file size.
    Name(String),
    /// Got file size, waiting for the file body to arrive.
    BodySize(String, u32),
}

pub struct Installer {
    protocol: Option<u8>,
    expected_size: Option<u32>,
    author_id: Option<String>,
    app_id: Option<String>,
    file: FileStatus,
    buf: VecDeque<u8>,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            protocol: None,
            expected_size: None,
            author_id: None,
            app_id: None,
            file: FileStatus::Waiting,
            buf: VecDeque::new(),
        }
    }

    /// Add the chunk to the buffer and parse the parts of the buffer that can be parsed.
    pub fn update(&mut self, chunk: &[u8]) {
        self.buf.extend(chunk);
        loop {
            // Parse protocol version.
            if self.protocol.is_none() {
                let Some(protocol) = self.buf.pop_front() else {
                    break;
                };
                self.protocol = Some(protocol);
                continue;
            }

            // Parse the expected total size of files.
            if self.expected_size.is_none() {
                let Some(expected_size) = self.pop_u32() else {
                    break;
                };
                self.expected_size = Some(expected_size);
                continue;
            }

            match &self.file {
                FileStatus::Waiting => {
                    let Some(size) = self.pop_u32() else {
                        break;
                    };
                    self.file = FileStatus::NameLen(size);
                }
                FileStatus::NameLen(size) => {
                    let Some(name) = self.pop_string(*size) else {
                        break;
                    };
                    if self.author_id.is_none() {
                        self.author_id = Some(name);
                        self.file = FileStatus::Waiting;
                        continue;
                    }
                    if self.app_id.is_none() {
                        self.app_id = Some(name);
                        self.file = FileStatus::Waiting;
                        continue;
                    }
                    self.file = FileStatus::Name(name);
                }
                FileStatus::Name(name) => {
                    let name = name.clone();
                    let Some(size) = self.pop_u32() else {
                        break;
                    };
                    self.file = FileStatus::BodySize(name, size);
                }
                FileStatus::BodySize(_, size) => {
                    let Some(content) = self.pop_bytes(*size) else {
                        break;
                    };
                    let FileStatus::BodySize(name, _) = &self.file else {
                        unreachable!()
                    };
                    let path = alloc::format!(
                        "rom/{}/{}/{name}",
                        self.author_id.as_ref().unwrap(),
                        self.app_id.as_ref().unwrap()
                    );
                    sudo::dump_file(&path, &content);
                    self.file = FileStatus::Waiting;
                }
            }
        }
    }

    fn pop_u32(&mut self) -> Option<u32> {
        if self.buf.len() < 4 {
            return None;
        }
        let n = u32::from_le_bytes([
            self.buf.pop_front().unwrap(),
            self.buf.pop_front().unwrap(),
            self.buf.pop_front().unwrap(),
            self.buf.pop_front().unwrap(),
        ]);
        Some(n)
    }

    fn pop_string(&mut self, size: u32) -> Option<String> {
        let raw = self.pop_bytes(size)?;
        let s = String::from_utf8(raw).unwrap_or_default();
        Some(s)
    }

    fn pop_bytes(&mut self, size: u32) -> Option<Vec<u8>> {
        let size = size as usize;
        if self.buf.len() < size {
            return None;
        }
        let mut res = Vec::with_capacity(size);
        for _ in 0..size {
            res.push(self.buf.pop_front().unwrap());
        }
        Some(res)
    }
}
