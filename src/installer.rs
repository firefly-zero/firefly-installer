use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

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
    today: Option<u32>,
    expected_size: Option<u32>,
    received_size: u32,
    author_id: Option<String>,
    app_id: Option<String>,
    file: FileStatus,
    buf: VecDeque<u8>,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            protocol: None,
            today: None,
            expected_size: None,
            received_size: 0,
            author_id: None,
            app_id: None,
            file: FileStatus::Waiting,
            buf: VecDeque::new(),
        }
    }

    pub fn done(&self) -> bool {
        if !self.buf.is_empty() {
            return false;
        }
        match self.expected_size {
            Some(expected_size) => self.received_size >= expected_size,
            None => false,
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

            if self.today.is_none() {
                let Some(today) = self.pop_u32() else {
                    break;
                };
                self.today = Some(today);
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
                    let Some(author_id) = self.author_id.as_ref() else {
                        self.author_id = Some(name);
                        self.file = FileStatus::Waiting;
                        continue;
                    };
                    if self.app_id.is_none() {
                        create_rom_dir(author_id, &name);
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
                    if size as usize > self.buf.len() {
                        self.buf.reserve(size as usize - self.buf.len());
                    }
                    self.file = FileStatus::BodySize(name, size);
                }
                FileStatus::BodySize(_, size) => {
                    let Some(content) = self.pop_bytes(*size) else {
                        break;
                    };
                    let FileStatus::BodySize(name, size) = &self.file else {
                        unreachable!()
                    };
                    self.received_size += size;
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

    pub fn finalize(&self) {
        let author_id = self.author_id.as_ref().unwrap();
        let app_id = self.app_id.as_ref().unwrap();
        let data_path = alloc::format!("data/{author_id}/{app_id}");
        sudo::create_dir(&data_path);
        let etc_path = alloc::format!("{data_path}/etc");
        sudo::create_dir(&etc_path);
        let shots_path = alloc::format!("{data_path}/shots");
        sudo::create_dir(&shots_path);

        let today = self.today.unwrap();
        let today = ((today >> 16) as u16, (today >> 8) as u8, today as u8);

        // Handle changes in app stats (badges and scoreboards).
        let stats_path = alloc::format!("{data_path}/stats");
        if sudo::get_file_size(&stats_path) == 0 {
            // Create stats
        } else {
            // Update stats
        }

        let cache_path = "data/sys/launcher/etc/metas";
        sudo::remove_file(cache_path);

        // Unlike in firefly-cli, here we don't need
        // to write `/sys/new-app` or `sys/launcher`.
        // If the user launches "sys.installer",
        // we assume that they already have a working launcher installed.
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

/// Remove the old ROM (if any) and create an empty dir for the new ROM.
fn create_rom_dir(author_id: &str, app_id: &str) {
    let author_path = alloc::format!("roms/{author_id}");
    let rom_path = alloc::format!("{author_path}/{app_id}");
    let bin_path = alloc::format!("{rom_path}/_bin");
    if sudo::get_file_size(&bin_path) != 0 {
        let files = sudo::DirBuf::list_files(&rom_path);
        for file_path in files.iter() {
            sudo::remove_file(file_path);
        }
    } else {
        if author_path != "sys" {
            sudo::create_dir(&author_path);
        }
        sudo::create_dir(&rom_path);
    }
}
