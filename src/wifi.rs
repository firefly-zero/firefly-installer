use crate::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::net::SocketAddrV4;

/// Wi-Fi connection status.
#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Error,
    /// Unknown status.
    Other,
    /// Not connected (or failed to connect) to an Access Point.
    Disconnected,
    /// Connected to the Access Point, obtaining IP address.
    Initializing,
    /// IP address is obtained, ready to go.
    Connected,
}

/// TCP connection status.
///
/// The descriptions for each status are from [IBM docs][ref].
///
/// [ref]: https://www.ibm.com/docs/en/zos/2.1.0?topic=SSLTBW_2.1.0/com.ibm.zos.v2r1.halu101/constatus.html
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TcpStatus {
    /// TCP status cannot be retrieved.
    Error,

    /// Represents no connection state at all.
    Closed,

    /// Waiting for a connection request from a remote TCP application.
    ///
    /// This is the state in which you can find the listening socket
    /// of a local TCP server.
    Listen,

    /// Waiting for an acknowledgment from the remote endpoint
    /// after having sent a connection request. Results after step 1
    /// of the three-way TCP handshake.
    SynSent,

    /// This endpoint has received a connection request and sent an acknowledgment.
    ///
    /// This endpoint is waiting for final acknowledgment that the other endpoint
    /// did receive this endpoint's acknowledgment of the original connection
    /// request. Results after step 2 of the three-way TCP handshake.
    SynReceived,

    /// Represents a fully established connection.
    ///
    /// This is the normal state for the data transfer phase of the connection.
    Established,

    /// Waiting for an acknowledgment of the connection termination
    /// request or for a simultaneous connection termination request
    /// from the remote TCP. This state is normally of short duration.
    FinWait1,

    /// Waiting for a connection termination request from the remote
    /// TCP after this endpoint has sent its connection termination request.
    /// This state is normally of short duration, but if the remote socket
    /// endpoint does not close its socket shortly after it has received
    /// information that this socket endpoint closed the connection, then
    /// it might last for some time. Excessive FIN-WAIT-2 states can indicate
    /// an error in the coding of the remote application.
    FinWait2,

    /// This endpoint has received a close request from the remote endpoint
    /// and this TCP is now waiting for a connection termination request
    /// from the local application.
    CloseWait,

    /// Waiting for a connection termination request acknowledgment from the remote TCP.
    ///
    /// This state is entered when this endpoint receives a close request
    /// from the local application, sends a termination request to
    /// the remote endpoint, and receives a termination request before
    /// it receives the acknowledgment from the remote endpoint.
    Closing,

    /// Waiting for an acknowledgment of the connection termination
    /// request previously sent to the remote TCP. This state is entered
    /// when this endpoint received a termination request before it sent
    /// its termination request.
    LastAck,

    /// Waiting for enough time to pass to be sure the remote TCP
    /// received the acknowledgment of its connection termination request.
    TimeWait,
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
        2 => Status::Disconnected,
        3 => Status::Initializing,
        4 => Status::Connected,
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

pub fn tcp_connect(addr: SocketAddrV4) {
    let ip: u32 = u32::from_be_bytes(addr.ip().octets());
    let port = u32::from(addr.port());
    unsafe {
        bindings::tcp_connect(ip, port);
    }
}

pub fn tcp_status() -> TcpStatus {
    let status = unsafe { bindings::tcp_status() };
    match status {
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
        _ => TcpStatus::Error,
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
