use crate::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::net::SocketAddrV4;

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Connected,
    Disconnected,
    Error,
    Other,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TcpStatus {
    Error,
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Unknown,
}

/// Connect to a Wi-Fi access point.
pub fn connect(ssid: &str, pass: &str) {
    unsafe {
        bindings::connect(
            ssid.as_ptr() as u32,
            ssid.len() as u32,
            pass.as_ptr() as u32,
            pass.len() as u32,
        )
    };
}

pub fn status() -> Status {
    let status = unsafe { bindings::status() };
    match status {
        0 => Status::Error,
        1 => Status::Connected,
        2 => Status::Disconnected,
        _ => Status::Other,
    }
}

/// Close Wi-Fi connection.
pub fn disconnect() {
    unsafe {
        bindings::disconnect();
    }
}

/// List SSIDs of the top few Wi-Fi access points.
pub fn scan() -> Vec<String> {
    let mut buf: Vec<u8> = vec![0; 200];
    let len = unsafe { bindings::scan(buf.as_mut_ptr() as u32, buf.len() as u32) };
    let mut raw = &buf[..len as usize];

    let mut points: Vec<String> = Vec::new();
    while !raw.is_empty() {
        let len = raw[0] as usize;
        let Some(raw_name) = raw.get(1..=len) else {
            break;
        };
        let name = core::str::from_utf8(raw_name).unwrap_or("");
        raw = raw.get((len + 1)..).unwrap_or(&[]);
        let name = String::from(name);
        points.push(name);
    }
    points
}

pub fn tcp_open(addr: SocketAddrV4) {
    let ip: u32 = u32::from_be_bytes(addr.ip().octets());
    let port = u32::from(addr.port());
    unsafe {
        bindings::tcp_open(ip, port);
    }
}

pub fn tcp_status() -> TcpStatus {
    let status = unsafe { bindings::tcp_status() };
    match status {
        0 => TcpStatus::Error,
        1 => TcpStatus::Closed,
        2 => TcpStatus::Listen,
        3 => TcpStatus::SynSent,
        4 => TcpStatus::SynReceived,
        5 => TcpStatus::Established,
        6 => TcpStatus::FinWait1,
        7 => TcpStatus::FinWait2,
        8 => TcpStatus::CloseWait,
        9 => TcpStatus::Closing,
        10 => TcpStatus::LastAck,
        11 => TcpStatus::TimeWait,
        _ => TcpStatus::Unknown,
    }
}

pub fn tcp_send(data: &[u8]) {
    unsafe {
        bindings::tcp_send(data.as_ptr() as u32, data.len() as u32);
    }
}

pub fn tcp_close() {
    unsafe {
        bindings::tcp_close();
    }
}
