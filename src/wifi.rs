use crate::*;
use alloc::{string::String, vec::Vec};

pub fn disconnect() {
    unsafe {
        bindings::disconnect();
    }
}

pub fn scan() -> Vec<String> {
    let mut buf: Vec<u8> = Vec::with_capacity(200);
    let len = unsafe { bindings::scan(buf.as_mut_ptr() as u32, buf.len() as u32) };
    let mut raw = &buf[..len as usize];

    let mut points: Vec<String> = Vec::new();
    while !raw.is_empty() {
        let len = raw[0] as usize;
        let Some(raw_name) = raw.get(1..=len) else {
            break;
        };
        // Security: do NOT return None on invalid string. Otherwise,
        // adding an invalid name into the directory would hide all files
        // that go after that.
        let name = core::str::from_utf8(raw_name).unwrap_or("");
        raw = raw.get((len + 1)..).unwrap_or(&[]);
        let name = String::from(name);
        points.push(name);
    }
    points
}
