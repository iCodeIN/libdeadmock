// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `libdeadmock` utilities
use slog::trace;
use slog::Logger;
use slog_try::try_trace;
use std::convert::TryFrom;
use tokio::net::TcpStream;

#[allow(clippy::cast_precision_loss)]
fn as_mebibytes(val: usize) -> f64 {
    (val as f64) / 1_048_576.
}

crate fn socket_info(socket: &TcpStream, stdout: &Option<Logger>) {
    let local_addr = socket
        .local_addr()
        .ok()
        .map_or_else(|| "Unknown".to_string(), |v| v.to_string());
    let peer_addr = socket
        .peer_addr()
        .ok()
        .map_or_else(|| "Unknown".to_string(), |v| v.to_string());
    let tcp_nodelay = socket.nodelay().unwrap_or(false);
    let recv_size = socket.recv_buffer_size().map(as_mebibytes).unwrap_or(0.);
    let send_size = socket.send_buffer_size().map(as_mebibytes).unwrap_or(0.);
    let ttl = socket.ttl().unwrap_or(0);
    let linger =
        u64::try_from(socket.linger().unwrap_or(None).map_or(0, |v| v.as_millis())).unwrap_or(0);
    let keepalive = u64::try_from(
        socket
            .keepalive()
            .unwrap_or(None)
            .map_or(0, |v| v.as_millis()),
    )
    .unwrap_or(0);

    try_trace!(
        stdout,
        "Accepting connection";
        "SO_SNDBUF" => format!("{:.3} MiB", send_size),
        "SO_RCVBUF" => format!("{:.3} MiB", recv_size),
        "SO_LINGER" => linger,
        "SO_KEEPALIVE" => keepalive,
        "IP_TTL" => ttl,
        "TCP_NODELAY" => tcp_nodelay,
        "local_addr" => local_addr,
        "peer_addr" => peer_addr,
    );
}
