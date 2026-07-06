use alloc::str;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{boxed::Box, collections::VecDeque};
use firefly_sudo::sudo;
use firefly_types::{BadgeProgress, BoardScores, Encode, FriendScore, Stats};
use sha2::{Digest, Sha256};

use crate::*;

const KB: u32 = 1024;
/// Keep partially downloaded file in RAM until it reaches at least that many bytes.
const FLUSH_EVERY: u32 = 64 * KB;

enum FileStatus {
    /// Less than 4 bytes received, cannot get file name length.
    Waiting,
    /// Received file name length, waiting for the file name to fully arrive.
    NameLen(u32),
    /// Received file name, waiting for the file size.
    Name(String),
    /// Got file size, waiting for the file body to arrive.
    BodySize {
        name: String,
        /// The total file size.
        size: u32,
        /// How much is yet to be dumped on the disk.
        /// Includes both buffered and not-yet-downloaded bytes.
        left: u32,
    },
}

pub struct Installer {
    pub headers: Option<RespHeaders>,
    pub received_size: u32,
    file: FileStatus,
    buf: VecDeque<u8>,
    hasher: Sha256,
    pub has_manual: bool,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            headers: None,
            received_size: 0,
            file: FileStatus::Waiting,
            buf: VecDeque::new(),
            hasher: Sha256::new(),
            has_manual: false,
        }
    }

    pub fn done(&self) -> bool {
        if !self.buf.is_empty() {
            return false;
        }
        match &self.headers {
            Some(headers) => self.received_size >= headers.expected_size,
            None => false,
        }
    }

    pub fn get_id(&self) -> (&str, &str) {
        let headers = self.headers.as_ref().unwrap();
        (&headers.author_id, &headers.app_id)
    }

    /// Add the chunk to the buffer and parse the parts of the buffer that can be parsed.
    pub fn update(&mut self, chunk: &[u8]) -> Result<(), &'static str> {
        self.buf.extend(chunk);
        loop {
            if self.headers.is_none() {
                let data = self.buf.make_contiguous();
                let Some(idx) = find_subslice(data, b"\r\n\r\n") else {
                    break;
                };
                let size = idx as u32 + 4;
                let headers = self.pop_bytes(size).unwrap();
                let headers = RespHeaders::parse(&headers)?;
                create_rom_dir(&headers.author_id, &headers.app_id);
                self.headers = Some(headers);
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
                    if name != "_hash" {
                        self.hasher.update("\x00");
                        self.hasher.update(name.as_bytes());
                        self.hasher.update("\x00");
                    }
                    self.file = FileStatus::Name(name);
                }
                FileStatus::Name(name) => {
                    let name = name.clone();
                    let Some(size) = self.pop_u32() else {
                        break;
                    };
                    let required_buf = u32::min(size, FLUSH_EVERY + 200) as usize;
                    if required_buf > self.buf.len() {
                        self.buf.reserve(required_buf - self.buf.len());
                    }
                    if name == "_manual" {
                        self.has_manual = true;
                    }
                    self.file = FileStatus::BodySize {
                        name,
                        size,
                        left: size,
                    };
                }
                FileStatus::BodySize { left, .. } => {
                    let chunk_size = (*left).min(FLUSH_EVERY);
                    let Some(content) = self.pop_bytes(chunk_size) else {
                        break;
                    };
                    let FileStatus::BodySize { name, size, left } = &self.file else {
                        unreachable!()
                    };
                    if name != "_hash" {
                        self.hasher.update(&content);
                    }
                    self.received_size += chunk_size;

                    let h = self.headers.as_ref().unwrap();
                    let path = alloc::format!("roms/{}/{}/{name}", h.author_id, h.app_id);
                    if *left == *size {
                        sudo::dump_file(&path, &content);
                    } else {
                        sudo::append_file(&path, &content);
                    }

                    let left = left - chunk_size;
                    if left == 0 {
                        self.file = FileStatus::Waiting;
                    } else {
                        self.file = FileStatus::BodySize {
                            name: name.to_string(),
                            size: *size,
                            left,
                        };
                    }
                }
            }
        }
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<(), &'static str> {
        let headers = self.headers.as_ref().unwrap();
        let author_id = &headers.author_id;
        let app_id = &headers.app_id;
        let rom_path = alloc::format!("roms/{author_id}/{app_id}");
        check_rom(&rom_path)?;

        // Validate hash.
        {
            let hash_path = alloc::format!("{rom_path}/_hash");
            let Some(exp_hash) = sudo::load_file_buf(&hash_path) else {
                return Err("failed to read ROM hash");
            };
            let exp_hash = &exp_hash.into_bytes()[..];
            let act_hash = self.hasher.finalize_reset();
            let act_hash: &[u8] = &act_hash;
            if act_hash != exp_hash {
                return Err("ROM is corrupted (hashsum mismatch)");
            }
        }

        // Create data directories.
        let data_path = alloc::format!("data/{author_id}/{app_id}");
        sudo::create_dir(&data_path);
        let etc_path = alloc::format!("{data_path}/etc");
        sudo::create_dir(&etc_path);
        let shots_path = alloc::format!("{data_path}/shots");
        sudo::create_dir(&shots_path);

        // Ensure that the data dir is created and writable.
        let tmp_path = alloc::format!("{etc_path}/_tmp");
        sudo::dump_file(&tmp_path, &[1]);
        if sudo::get_file_size(&tmp_path) == 0 {
            return Err("failed to create app data dir");
        }
        sudo::remove_file(&tmp_path);
        if sudo::get_file_size(&tmp_path) != 0 {
            return Err("failed to clean up app data dir");
        }

        // Create or update stats file.
        write_stats(rom_path, data_path, headers.today)?;

        // Clear launcher cache.
        let cache_path = "data/sys/launcher/etc/metas";
        sudo::remove_file(cache_path);
        if sudo::get_file_size(cache_path) != 0 {
            return Err("failed to reset launcher cache");
        }

        // Unlike in firefly-cli, here we don't need
        // to write `/sys/new-app` or `sys/launcher`.
        // If the user launches "sys.installer",
        // we assume that they already have a working launcher installed.

        Ok(())
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

fn find_subslice<T: PartialEq>(data: &[T], needle: &[T]) -> Option<usize> {
    // https://github.com/rust-lang/rust/issues/54961
    data.windows(needle.len())
        .enumerate()
        .find(|&(_, w)| w == needle)
        .map(|(i, _)| i)
}

fn write_stats(
    rom_path: String,
    data_path: String,
    today: (u16, u8, u8),
) -> Result<(), &'static str> {
    let default_path = alloc::format!("{rom_path}/_stats");
    let Some(raw) = sudo::load_file_buf(&default_path) else {
        return Err("cannot access achievements info from the app");
    };
    let raw = raw.into_bytes();
    let Ok(default) = Stats::decode(&raw) else {
        return Err("failed to parse the app achievements info");
    };
    let stats_path = alloc::format!("{data_path}/stats");
    let stats = if sudo::get_file_size(&stats_path) == 0 {
        Stats {
            minutes: [0; 4],
            longest_play: [0; 4],
            launches: [0; 4],
            installed_on: today,
            updated_on: today,
            launched_on: (0, 0, 0),
            xp: 0,
            badges: default.badges,
            scores: default.scores,
        }
    } else {
        update_stats(&default, &stats_path, today)?
    };
    let Ok(raw) = stats.encode_vec() else {
        return Err("failed to serialize achievements info");
    };
    sudo::dump_file(&stats_path, &raw);
    if sudo::get_file_size(&stats_path) == 0 {
        return Err("failed to bootstrap app stats");
    }
    Ok(())
}

/// Check that the given ROM directory has all required files.
fn check_rom(rom_path: &str) -> Result<(), &'static str> {
    let bin_path = alloc::format!("{rom_path}/_bin");
    if sudo::get_file_size(&bin_path) == 0 {
        return Err("failed to write ROM");
    }
    let bin_path = alloc::format!("{rom_path}/_meta");
    if sudo::get_file_size(&bin_path) == 0 {
        return Err("ROM has no metadata");
    }
    let bin_path = alloc::format!("{rom_path}/_hash");
    if sudo::get_file_size(&bin_path) == 0 {
        return Err("ROM has no hash");
    }
    let bin_path = alloc::format!("{rom_path}/_stats");
    if sudo::get_file_size(&bin_path) == 0 {
        return Err("ROM has no achievements info");
    }
    Ok(())
}

/// Generate stats from the default template provided by the ROM.
fn update_stats(
    default: &Stats,
    stats_path: &str,
    today: (u16, u8, u8),
) -> Result<Stats, &'static str> {
    let Some(raw) = sudo::load_file_buf(stats_path) else {
        return Err("failed to read app stats");
    };
    let Ok(old_stats) = Stats::decode(&raw.into_bytes()) else {
        return Err("failed to parse app stats");
    };

    // The current date might be behind the current date on the device,
    // and it might be reflected in the dates recorded in the stats.
    // If that happens, try to stay closer to the device time.
    let today = today
        .max(old_stats.installed_on)
        .max(old_stats.launched_on)
        .max(old_stats.updated_on);

    let mut badges = Vec::new();
    for (i, default_badge) in default.badges.iter().enumerate() {
        let new_badge = if let Some(old_badge) = old_stats.badges.get(i) {
            BadgeProgress {
                new: old_badge.new,
                done: old_badge.done.min(default_badge.goal),
                goal: default_badge.goal,
            }
        } else {
            BadgeProgress {
                new: false,
                done: 0,
                goal: default_badge.goal,
            }
        };
        badges.push(new_badge);
    }

    let mut scores = Vec::from(old_stats.scores);
    scores.truncate(default.scores.len());
    for _ in scores.len()..default.scores.len() {
        let fs = FriendScore { index: 0, score: 0 };
        let new_score = BoardScores {
            me: Box::new([0i16; 8]),
            friends: Box::new([fs; 8]),
        };
        scores.push(new_score);
    }

    let new_stats = Stats {
        minutes: old_stats.minutes,
        longest_play: old_stats.longest_play,
        launches: old_stats.launches,
        installed_on: old_stats.installed_on,
        updated_on: today,
        launched_on: old_stats.launched_on,
        xp: old_stats.xp.min(1000),
        badges: badges.into_boxed_slice(),
        scores: scores.into_boxed_slice(),
    };
    Ok(new_stats)
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
