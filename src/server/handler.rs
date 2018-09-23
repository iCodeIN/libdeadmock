// Copyright (c) 2018 libdeadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Request/Response handling for the async runtime.
use cached::{cached_key_result, UnboundCache};
use crate::config;
use crate::matcher::{Enabled, Matcher};
use crate::server::codec;
use crate::server::header;
use crate::util::{self, FutResponse};
use failure::Error;
use futures::{future, Future, Sink, Stream};
use http::{Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper::{Client, Request as HyperRequest};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use slog::Logger;
use slog::{b, error, info, kv, log, record, record_static, trace};
use slog_try::{try_error, try_info, try_trace};
use std::fs::File;
use std::io::{self, BufReader, ErrorKind, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::await;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::FutureExt;
use tokio_codec::Decoder;
use typed_headers::Credentials;

/// Request/Response handler for the async runtime.
#[derive(Clone, Debug)]
pub struct Handler {
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    proxy_config: config::Proxy,
    files_path: PathBuf,
    enabled: Enabled,
    static_mappings: config::Mappings,
    dynamic_mappings: Arc<Mutex<config::Mappings>>,
}

impl Handler {
    /// Create a new `Handler` from the given config and tokio stream.
    pub fn new(
        enabled: Enabled,
        static_mappings: config::Mappings,
        proxy_config: config::Proxy,
        files_path: PathBuf,
    ) -> Self {
        Self {
            stdout: None,
            stderr: None,
            proxy_config,
            files_path,
            enabled,
            static_mappings,
            dynamic_mappings: Arc::new(Mutex::new(config::Mappings::default())),
        }
    }

    /// Add a stdout slog logger to this handler.
    pub fn stdout(mut self, stdout: Option<Logger>) -> Self {
        self.stdout = stdout;
        self
    }

    /// Add a stderr slog logger to this handler.
    pub fn stderr(mut self, stderr: Option<Logger>) -> Self {
        self.stderr = stderr;
        self
    }
}

/// Spawn a task onto the event loop to handle the request.
#[allow(box_pointers)]
pub fn handle(handler: Handler, stream: TcpStream) {
    // Frame the socket using the `Http` protocol. This maps the TCP socket
    // to a Stream + Sink of HTTP frames.
    // This splits a single `Stream + Sink` value into two separate handles
    // that can be used independently (even on different tasks or threads).
    let (tx, rx) = codec::Http.framed(stream).split();

    // Clone all the things....
    let response_stderr_1 = handler.stderr.clone();

    // Map all requests into responses and send them back to the client.
    let task = tx
        .send_all(rx.and_then(move |req| {
            respond(handler.clone(), &req).map_err(|e| io::Error::new(ErrorKind::Other, e))
        })).then(move |res| {
            if let Err(e) = res {
                try_error!(response_stderr_1, "failed to process the request: {}", e);
            }

            Ok(())
        });

    // Spawn the task that handles the connection.
    let _ = tokio::spawn(task);
}

#[allow(box_pointers)]
fn respond(handler: Handler, request: &Request<()>) -> FutResponse {
    let matcher = Matcher::new(
        handler.enabled,
        handler.stdout.clone(),
        handler.stderr.clone(),
    );

    if let Ok(mapping) = matcher.get_match(&request, &handler.static_mappings) {
        try_trace!(handler.stdout, "{}", mapping);
        http_response(handler, &request, mapping.response())
    } else {
        let dynamic_mappings = handler.dynamic_mappings.clone();
        let locked_dynamic_mappings = match dynamic_mappings.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Ok(mapping) = matcher.get_match(&request, &locked_dynamic_mappings) {
            try_trace!(handler.stdout, "{}", mapping);
            http_response(handler, &request, mapping.response())
        } else {
            try_error!(handler.stderr, "No mapping found");
            util::error_response_fut("No mapping found".to_string(), StatusCode::NOT_FOUND)
        }
    }
}

#[allow(box_pointers)]
fn http_response(
    handler: Handler,
    request: &Request<()>,
    response_config: &config::Response,
) -> FutResponse {
    if let Some(proxy_base_url) = response_config.proxy_base_url() {
        let full_url = format!("{}{}", proxy_base_url, request.uri());
        let (tx, rx) = futures::sync::mpsc::unbounded();
        let headers = response_config.additional_proxy_request_headers().clone();
        let proxy_config = handler.proxy_config.clone();
        tokio::spawn_async(
            async move {
                if *proxy_config.use_proxy() {
                    if let Some(url_str) = proxy_config.proxy_url() {
                        let proxy_uri = url_str.parse().expect("Unable to parse proxy URI");
                        let mut proxy = Proxy::new(Intercept::All, proxy_uri);
                        if let Some(username) = proxy_config.proxy_username() {
                            if let Some(password) = proxy_config.proxy_password() {
                                if let Ok(creds) = Credentials::basic(username, password) {
                                    proxy.set_authorization(creds);
                                }
                            }
                        }

                        let connector = HttpConnector::new(4);
                        let proxy_connector = ProxyConnector::from_proxy(connector, proxy)
                            .expect("Unable to create proxy connector!");
                        let client = Client::builder()
                            .set_host(true)
                            .build::<_, hyper::Body>(proxy_connector);
                        await!(run_request(
                            client,
                            tx,
                            full_url,
                            handler.stdout.clone(),
                            handler.stderr.clone(),
                            headers
                        ));
                    } else {
                        panic!("Unable to determine proxy url!");
                    }
                } else if full_url.starts_with("https") {
                    let https_connector =
                        HttpsConnector::new(4).expect("TLS initialization failed");
                    let client = Client::builder()
                        .set_host(true)
                        .build::<_, hyper::Body>(https_connector);
                    await!(run_request(
                        client,
                        tx,
                        full_url,
                        handler.stdout.clone(),
                        handler.stderr.clone(),
                        headers
                    ));
                } else {
                    let http_connector = HttpConnector::new(4);
                    let client = Client::builder()
                        .set_host(true)
                        .build::<_, hyper::Body>(http_connector);
                    await!(run_request(
                        client,
                        tx,
                        full_url,
                        handler.stdout,
                        handler.stderr,
                        headers
                    ));
                }
            },
        );

        Box::new(
            rx.fold(String::new(), |mut buffer, res| {
                match res {
                    Ok(val) => buffer.push_str(&val),
                    Err(e) => buffer.push_str(&e.to_string()),
                }
                futures::future::ok(buffer)
            }).map_err(|_| "Error processing upstream response".to_string())
            .map(Response::new),
        )
    } else {
        let mut response_builder = Response::builder();
        if let Some(headers) = response_config.headers() {
            for header in headers {
                let _ = response_builder.header(&header.key()[..], &header.value()[..]);
            }
        }

        if let Some(status) = response_config.status() {
            let _ = response_builder.status(if let Ok(status) = StatusCode::from_u16(*status) {
                status
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            });
        } else {
            let _ = response_builder.status(StatusCode::OK);
        }

        let body = if let Some(body_file_name) = response_config.body_file_name() {
            match load(handler.files_path, body_file_name) {
                Ok(body) => body,
                Err(e) => e.to_string(),
            }
        } else {
            "Unable to process body".to_string()
        };

        match response_builder.body(body) {
            Ok(response) => Box::new(future::ok(response)),
            Err(e) => util::error_response_fut(format!("{}", e), StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

async fn run_request<C>(
    client: Client<C, hyper::Body>,
    tx: futures::sync::mpsc::UnboundedSender<Result<String, String>>,
    url: String,
    stdout: Option<Logger>,
    stderr: Option<Logger>,
    headers: Option<Vec<config::Header>>,
) where
    C: hyper::client::connect::Connect + Sync + 'static,
{
    match await!({
        try_trace!(stdout, "Making request to {}", url);
        let mut request_builder = HyperRequest::get(url);

        if let Some(headers) = headers {
            for header in headers {
                let _ = request_builder.header(&header.key()[..], &header.value()[..]);
            }
        }
        let body = request_builder
            .body(hyper::Body::empty())
            .expect("Unable to create upstream request");
        client
            .request(body)
            .timeout(std::time::Duration::from_secs(10))
    }) {
        Ok(response) => {
            let body = await!({
                response
                    .into_body()
                    .map_err(|_| ())
                    .fold(Vec::new(), |mut v, chunk| {
                        v.extend_from_slice(&chunk);
                        futures::future::ok(v)
                    })
            });

            if let Ok(body) = body {
                let body_str = String::from_utf8_lossy(&body).into_owned();
                tx.unbounded_send(Ok(body_str))
                    .expect("Unable to send upstream response!");
            } else {
                try_error!(stderr, "Unable to process upstream response!");
                tx.unbounded_send(Err("Unable to process upstream response!".to_string()))
                    .expect("Unable to send upstream response!");
            }
        }
        Err(e) => {
            try_error!(stderr, "Unable to process upstream response! {}", e);
            tx.unbounded_send(Err(format!("Unable to process upstream response! {}", e)))
                .expect("Unable to send upstream response!");
        }
    }
}

cached_key_result!{
    STATIC_RESPONSE: UnboundCache<String, String> = UnboundCache::new();
    Key = { filename.to_string() };
    fn load(files_path: PathBuf, filename: &str) -> Result<String, &str> = {
        let mut buffer = String::new();
        let mut found = false;

        util::visit_dirs(&files_path, &mut |entry| -> Result<(), Error> {
            if let Some(fname) = entry.path().file_name() {
                if fname.to_string_lossy() == filename {
                    let f = File::open(entry.path())?;
                    let mut reader = BufReader::new(f);
                    let _ = reader.read_to_string(&mut buffer)?;
                    found = true;
                }
            }
            Ok(())
        }).map_err(|_| "Body file not found!")?;

        if found {
            Ok(buffer)
        } else {
            Err("Body file not found!")
        }
    }
}

/// Start the async runtime handling.
pub fn run(socket_addr: &SocketAddr, handler: Handler) -> Result<(), Error> {
    let listener = TcpListener::bind(&socket_addr)?;

    // Run the server.
    // try_trace!(handler.stdout, "{:?}", current);
    try_info!(handler.stdout, "Listening on '{}'", socket_addr);

    let map_stderr = handler.stderr.clone();
    let process_stdout = handler.stdout.clone();

    tokio::run({
        listener
            .incoming()
            .map_err(move |e| try_error!(map_stderr, "Failed to accept socket: {}", e))
            .for_each(move |socket| {
                header::socket_info(&socket, &process_stdout);
                handle(handler.clone(), socket);
                Ok(())
            })
    });

    Ok(())
}
