//! Local network transfer: two computers on the same network hand a setup over directly, with
//! nothing in between. An app that is sharing a setup, or looking for one, announces itself with
//! a small multicast datagram every few seconds; the setup travels over one TCP connection as a
//! file encrypted under a six-digit PIN the owner reads out. The app is silent on the network
//! until the user shares or looks.
//!
//! The PIN and a random salt the sharing computer announces give, through the same Argon2id
//! derivation the transfer codes use, the name the file is served under and the key it is
//! encrypted with. A wrong PIN produces a name the server does not have; five wrong names from
//! one address within a minute and that address is refused for a minute, which keeps a million
//! PINs out of reach on a shared network.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Mutex, Notify},
    task::JoinHandle,
};

use super::transfer::{decrypt_file, derive_keys_with_salt, download_response, encrypt_file, TransferKeys, TransferProgress};

/// An administratively scoped multicast group and a port of our own for the announcements.
const DISCOVERY_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 30);
const DISCOVERY_PORT: u16 = 41230;
const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(5);
/// A peer silent for this long has gone: three missed announcements.
const PEER_TIMEOUT: Duration = Duration::from_secs(16);
const PROTOCOL: u32 = 1;
const PIN_DIGITS: usize = 6;
const SALT_BYTES: usize = 16;
/// Wrong names from one address within the window before it is refused for the window.
const WRONG_PIN_LIMIT: usize = 5;
const WRONG_PIN_WINDOW: Duration = Duration::from_secs(60);
/// A request head larger than this is not one of ours.
const REQUEST_LIMIT: usize = 4096;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DATAGRAM_LIMIT: usize = 2048;
const DEVICE_HEADER: &str = "x-blenderbase-device";

/// What a sharing computer says about its setup, in every announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanShareSummary {
    /// Hex; the receiving computer needs it to turn the PIN into the file's name and key.
    pub salt: String,
    pub size: u64,
    pub versions: usize,
    pub series: usize,
    /// RFC 3339: when the setup was read out of Blender.
    pub saved: String,
}

/// One datagram. `hello` asks everyone to answer at once (a computer that just started
/// looking); `bye` says the sender is leaving.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Announcement {
    blenderbase: u32,
    id: String,
    #[serde(default)]
    device: String,
    #[serde(default)]
    app: String,
    #[serde(default)]
    platform: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    hello: bool,
    #[serde(default)]
    bye: bool,
    #[serde(default)]
    share: Option<LanShareSummary>,
}

/// Another computer running Blenderbase on this network.
#[derive(Debug, Clone, Serialize)]
pub struct LanPeer {
    pub id: String,
    pub device: String,
    pub app_version: String,
    pub platform: String,
    pub address: String,
    pub port: u16,
    pub share: Option<LanShareSummary>,
    pub seen_seconds_ago: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanReceipt {
    pub device: String,
    pub at: String,
}

/// What this computer is sharing, for the Sync view.
#[derive(Debug, Clone, Serialize)]
pub struct LanShareStatus {
    pub pin: String,
    pub size: u64,
    pub versions: usize,
    pub series: usize,
    pub saved: String,
    pub received_by: Vec<LanReceipt>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanStatus {
    pub device: String,
    /// True while this computer announces itself: it is sharing, or the Local network tab is open.
    pub active: bool,
    pub share: Option<LanShareStatus>,
    pub peers: Vec<LanPeer>,
}

struct Share {
    pin: String,
    keys: TransferKeys,
    summary: LanShareSummary,
    file: PathBuf,
    directory: PathBuf,
    received_by: Vec<LanReceipt>,
}

struct PeerEntry {
    announcement: Announcement,
    address: IpAddr,
    seen: Instant,
}

struct Running {
    port: u16,
    tasks: Vec<JoinHandle<()>>,
    /// Wakes the announcer when the share changes, so peers hear about it at once.
    kick: Arc<Notify>,
    discovery: Arc<UdpSocket>,
}

#[derive(Default)]
struct Inner {
    browsing: bool,
    share: Option<Share>,
    peers: HashMap<String, PeerEntry>,
    wrong_names: HashMap<IpAddr, Vec<Instant>>,
    running: Option<Running>,
}

/// This computer on the local network: what it shares and whom it sees. One per app, managed
/// by Tauri; the sockets only exist while sharing or browsing.
pub struct LanHub {
    pub instance_id: String,
    pub device: String,
    pub app_version: String,
    pub platform: String,
    inner: Arc<Mutex<Inner>>,
}

impl LanHub {
    pub fn new(app_version: String) -> Self {
        Self {
            instance_id: uuid::Uuid::new_v4().to_string(),
            device: whoami::devicename().unwrap_or_else(|_| String::from("This computer")),
            app_version,
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            inner: Arc::new(Mutex::new(Inner::default())),
        }
    }

    pub async fn status(&self) -> LanStatus {
        let mut inner = self.inner.lock().await;
        let now = Instant::now();
        inner.peers.retain(|_, p| now.duration_since(p.seen) < PEER_TIMEOUT);
        let mut peers: Vec<LanPeer> = inner
            .peers
            .values()
            .map(|p| LanPeer {
                id: p.announcement.id.clone(),
                device: p.announcement.device.clone(),
                app_version: p.announcement.app.clone(),
                platform: p.announcement.platform.clone(),
                address: p.address.to_string(),
                port: p.announcement.port,
                share: p.announcement.share.clone(),
                seen_seconds_ago: now.duration_since(p.seen).as_secs(),
            })
            .collect();
        peers.sort_by(|a, b| a.device.to_lowercase().cmp(&b.device.to_lowercase()).then(a.id.cmp(&b.id)));
        LanStatus {
            device: self.device.clone(),
            active: inner.running.is_some(),
            share: inner.share.as_ref().map(|s| LanShareStatus {
                pin: s.pin.clone(),
                size: s.summary.size,
                versions: s.summary.versions,
                series: s.summary.series,
                saved: s.summary.saved.clone(),
                received_by: s.received_by.clone(),
            }),
            peers,
        }
    }

    /// Looking for other computers: on while the Local network tab is open.
    pub async fn set_browsing(&self, on: bool) -> Result<LanStatus, String> {
        {
            let mut inner = self.inner.lock().await;
            inner.browsing = on;
        }
        if on {
            self.ensure_running().await?;
        } else {
            self.stop_if_idle().await;
        }
        Ok(self.status().await)
    }

    /// Shares a setup file: encrypts it under a fresh PIN into a folder of its own and starts
    /// announcing it. A share already running is replaced.
    pub async fn start_share(&self, bundle: &Path, versions: usize, series: usize) -> Result<LanStatus, String> {
        let pin = generate_pin();
        let salt: [u8; SALT_BYTES] = rand::rng().random();
        let keys = {
            let pin = pin.clone();
            tokio::task::spawn_blocking(move || derive_keys_with_salt(pin.as_bytes(), &salt))
                .await
                .map_err(|e| format!("Failed start_share: {:?}", e))??
        };
        let directory = std::env::temp_dir().join(format!("blenderbase-lan-share-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).map_err(|e| format!("Could not create {}: {}", directory.display(), e))?;
        let file = directory.join("setup.enc");
        let size = {
            let (key, bundle, file) = (keys.key, bundle.to_path_buf(), file.clone());
            match tokio::task::spawn_blocking(move || encrypt_file(&key, &bundle, &file)).await {
                Ok(Ok(size)) => size,
                Ok(Err(e)) => {
                    let _ = std::fs::remove_dir_all(&directory);
                    return Err(e);
                }
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&directory);
                    return Err(format!("Failed start_share: {:?}", e));
                }
            }
        };
        let share = Share {
            pin,
            keys,
            summary: LanShareSummary {
                salt: hex_encode(&salt),
                size,
                versions,
                series,
                saved: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            },
            file,
            directory,
            received_by: Vec::new(),
        };
        let previous = {
            let mut inner = self.inner.lock().await;
            inner.share.replace(share)
        };
        if let Some(previous) = previous {
            let _ = std::fs::remove_dir_all(&previous.directory);
        }
        self.ensure_running().await?;
        self.announce_now().await;
        Ok(self.status().await)
    }

    pub async fn stop_share(&self) -> LanStatus {
        let previous = {
            let mut inner = self.inner.lock().await;
            inner.share.take()
        };
        if let Some(previous) = previous {
            let _ = std::fs::remove_dir_all(&previous.directory);
        }
        self.announce_now().await;
        self.stop_if_idle().await;
        self.status().await
    }

    /// Fetches the setup a peer shares and decrypts it to `destination`. Returns the peer's
    /// device name for the status line.
    pub async fn receive(
        &self,
        client: &reqwest::Client,
        peer_id: &str,
        pin: &str,
        destination: &Path,
        progress: &TransferProgress,
    ) -> Result<String, String> {
        let pin = normalise_pin(pin)?;
        let (device, address, port, summary) = {
            let inner = self.inner.lock().await;
            let peer = inner
                .peers
                .get(peer_id)
                .filter(|p| p.seen.elapsed() < PEER_TIMEOUT)
                .ok_or_else(|| String::from("That computer is no longer on the network"))?;
            let summary = peer
                .announcement
                .share
                .clone()
                .ok_or_else(|| format!("{} is not sharing a setup", peer.announcement.device))?;
            (peer.announcement.device.clone(), peer.address, peer.announcement.port, summary)
        };
        let salt = hex_decode(&summary.salt).ok_or_else(|| format!("{} announced a salt that cannot be read", device))?;
        progress(String::from("Checking the PIN…"));
        let keys = tokio::task::spawn_blocking(move || derive_keys_with_salt(pin.as_bytes(), &salt))
            .await
            .map_err(|e| format!("Failed receive: {:?}", e))??;
        let target = SocketAddr::new(address, port);
        // A quick connection first: reqwest would wait the OS connect timeout on a firewalled peer.
        match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(target)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(format!("Could not reach {} at {}: {}. A firewall on that computer may be blocking Blenderbase", device, target, e)),
            Err(_) => return Err(format!("Could not reach {} at {}: no answer. A firewall on that computer may be blocking Blenderbase", device, target)),
        }
        progress(format!("Receiving from {}…", device));
        let response = client
            .get(format!("http://{}/share/{}", target, keys.id))
            .header(DEVICE_HEADER, header_safe(&self.device))
            .send()
            .await
            .map_err(|e| format!("Could not fetch the setup from {}: {}", device, e))?;
        match response.status().as_u16() {
            200 => {}
            404 => return Err(format!("The PIN is wrong, or {} started a new share; check the PIN shown there", device)),
            429 => return Err(format!("Too many wrong PINs; {} refuses this computer for a minute", device)),
            other => return Err(format!("{} answered with {}", device, other)),
        }
        let encrypted = destination.with_extension("part");
        download_response(response, &encrypted, progress).await?;
        progress(String::from("Decrypting…"));
        let decrypted = {
            let (key, encrypted, destination) = (keys.key, encrypted.clone(), destination.to_path_buf());
            tokio::task::spawn_blocking(move || decrypt_file(&key, &encrypted, &destination))
                .await
                .map_err(|e| format!("Failed receive: {:?}", e))?
        };
        let _ = std::fs::remove_file(&encrypted);
        decrypted?;
        Ok(device)
    }

    /// The app is exiting: drops the share and tells the network at once, without waiting on
    /// anything (Tauri's exit callback is not an async context).
    pub fn shutdown_blocking(&self) {
        let Ok(mut inner) = self.inner.try_lock() else {
            return;
        };
        inner.browsing = false;
        if let Some(share) = inner.share.take() {
            let _ = std::fs::remove_dir_all(&share.directory);
        }
        let Some(running) = inner.running.take() else {
            return;
        };
        let bye = Announcement {
            blenderbase: PROTOCOL,
            id: self.instance_id.clone(),
            device: self.device.clone(),
            app: self.app_version.clone(),
            platform: self.platform.clone(),
            port: running.port,
            hello: false,
            bye: true,
            share: None,
        };
        if let (Ok(payload), Ok(socket)) = (serde_json::to_vec(&bye), std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))) {
            let _ = socket.set_broadcast(true);
            let _ = socket.set_multicast_loop_v4(true);
            let _ = socket.send_to(&payload, (DISCOVERY_GROUP, DISCOVERY_PORT));
            for (_, broadcast) in local_interfaces() {
                let _ = socket.send_to(&payload, (broadcast.unwrap_or(Ipv4Addr::BROADCAST), DISCOVERY_PORT));
            }
        }
        for task in running.tasks {
            task.abort();
        }
    }

    /// Tells the network this computer is leaving; the async form for tests.
    pub async fn shutdown(&self) {
        let previous = {
            let mut inner = self.inner.lock().await;
            inner.browsing = false;
            inner.share.take()
        };
        if let Some(previous) = previous {
            let _ = std::fs::remove_dir_all(&previous.directory);
        }
        self.stop_if_idle().await;
    }

    async fn announce_now(&self) {
        let inner = self.inner.lock().await;
        if let Some(running) = inner.running.as_ref() {
            running.kick.notify_one();
        }
    }

    async fn ensure_running(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        if inner.running.is_some() {
            return Ok(());
        }
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
            .await
            .map_err(|e| format!("Could not open a port for the local network: {}", e))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("Could not read the local port: {}", e))?
            .port();
        let discovery = Arc::new(bind_discovery_socket().map_err(|e| {
            format!("Could not listen for other computers on UDP port {}: {}", DISCOVERY_PORT, e)
        })?);
        let kick = Arc::new(Notify::new());
        let identity = Identity {
            instance_id: self.instance_id.clone(),
            device: self.device.clone(),
            app_version: self.app_version.clone(),
            platform: self.platform.clone(),
            port,
        };
        let tasks = vec![
            tokio::spawn(accept_loop(listener, Arc::clone(&self.inner))),
            tokio::spawn(receive_loop(Arc::clone(&discovery), Arc::clone(&self.inner), identity.clone())),
            tokio::spawn(announce_loop(Arc::clone(&discovery), Arc::clone(&self.inner), identity, Arc::clone(&kick))),
        ];
        inner.running = Some(Running { port, tasks, kick, discovery });
        Ok(())
    }

    /// Closes the sockets once nothing needs them: no share, and nobody looking.
    async fn stop_if_idle(&self) {
        let running = {
            let mut inner = self.inner.lock().await;
            if inner.browsing || inner.share.is_some() {
                return;
            }
            inner.peers.clear();
            inner.running.take()
        };
        if let Some(running) = running {
            let bye = Announcement {
                blenderbase: PROTOCOL,
                id: self.instance_id.clone(),
                device: self.device.clone(),
                app: self.app_version.clone(),
                platform: self.platform.clone(),
                port: running.port,
                hello: false,
                bye: true,
                share: None,
            };
            send_everywhere(&running.discovery, &bye).await;
            for task in running.tasks {
                task.abort();
            }
        }
    }
}

impl Drop for LanHub {
    fn drop(&mut self) {
        if let Ok(inner) = self.inner.try_lock() {
            if let Some(share) = inner.share.as_ref() {
                let _ = std::fs::remove_dir_all(&share.directory);
            }
        }
    }
}

#[derive(Clone)]
struct Identity {
    instance_id: String,
    device: String,
    app_version: String,
    platform: String,
    port: u16,
}

fn announcement_for(identity: &Identity, inner: &Inner, hello: bool) -> Announcement {
    Announcement {
        blenderbase: PROTOCOL,
        id: identity.instance_id.clone(),
        device: identity.device.clone(),
        app: identity.app_version.clone(),
        platform: identity.platform.clone(),
        port: identity.port,
        hello,
        bye: false,
        share: inner.share.as_ref().map(|s| s.summary.clone()),
    }
}

/// Six digits from the OS random source, leading zeros kept.
pub fn generate_pin() -> String {
    format!("{:06}", rand::rng().random_range(0..1_000_000u32))
}

/// The PIN as typed, digits only: spaces and dashes between groups are fine.
pub fn normalise_pin(typed: &str) -> Result<String, String> {
    let digits: String = typed.chars().filter(|c| c.is_ascii_digit()).collect();
    let others = typed.chars().any(|c| !c.is_ascii_digit() && !c.is_whitespace() && c != '-' && c != '.');
    if digits.len() != PIN_DIGITS || others {
        return Err(format!("The PIN is {} digits, as shown on the other computer", PIN_DIGITS));
    }
    Ok(digits)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || text.is_empty() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// A device name as a header value: printable ASCII only.
fn header_safe(device: &str) -> String {
    let safe: String = device
        .chars()
        .filter(|c| c.is_ascii() && !c.is_ascii_control())
        .take(80)
        .collect();
    if safe.trim().is_empty() {
        String::from("Another computer")
    } else {
        safe
    }
}

/// The IPv4 addresses of this computer's network interfaces, loopback left out.
fn local_interfaces() -> Vec<(Ipv4Addr, Option<Ipv4Addr>)> {
    let mut interfaces = Vec::new();
    if let Ok(list) = if_addrs::get_if_addrs() {
        for interface in list {
            if let if_addrs::IfAddr::V4(v4) = interface.addr {
                if !v4.ip.is_loopback() {
                    interfaces.push((v4.ip, v4.broadcast));
                }
            }
        }
    }
    interfaces
}

/// The socket every instance listens on: the port is shared, so two apps on one computer (or
/// an app and a test) both hear the announcements, and the group is joined on every interface.
fn bind_discovery_socket() -> std::io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT)).into())?;
    let _ = socket.join_multicast_v4(&DISCOVERY_GROUP, &Ipv4Addr::UNSPECIFIED);
    for (address, _) in local_interfaces() {
        let _ = socket.join_multicast_v4(&DISCOVERY_GROUP, &address);
    }
    UdpSocket::from_std(socket.into())
}

/// A socket that sends out of one interface: multicast to the group and broadcast on the
/// interface's subnet, so a network that filters one still carries the other.
fn bind_sender(interface: Ipv4Addr) -> std::io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_nonblocking(true)?;
    socket.set_multicast_if_v4(&interface)?;
    socket.set_multicast_ttl_v4(1)?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_broadcast(true)?;
    socket.bind(&SocketAddr::V4(SocketAddrV4::new(interface, 0)).into())?;
    UdpSocket::from_std(socket.into())
}

/// One announcement out of every interface, to the group and to the subnet.
async fn send_everywhere(discovery: &UdpSocket, announcement: &Announcement) {
    let Ok(payload) = serde_json::to_vec(announcement) else {
        return;
    };
    let group = SocketAddrV4::new(DISCOVERY_GROUP, DISCOVERY_PORT);
    let mut sent_any = false;
    for (address, broadcast) in local_interfaces() {
        let _ = discovery.join_multicast_v4(DISCOVERY_GROUP, address);
        let Ok(sender) = bind_sender(address) else {
            continue;
        };
        if sender.send_to(&payload, group).await.is_ok() {
            sent_any = true;
        }
        let broadcast = broadcast.unwrap_or(Ipv4Addr::BROADCAST);
        let _ = sender.send_to(&payload, SocketAddrV4::new(broadcast, DISCOVERY_PORT)).await;
    }
    if !sent_any {
        // No interface to speak of: the group over the default route still reaches this computer's own instances.
        let _ = discovery.send_to(&payload, group).await;
    }
}

async fn announce_loop(discovery: Arc<UdpSocket>, inner: Arc<Mutex<Inner>>, identity: Identity, kick: Arc<Notify>) {
    let mut hello = true;
    loop {
        let announcement = {
            let inner = inner.lock().await;
            announcement_for(&identity, &inner, hello)
        };
        send_everywhere(&discovery, &announcement).await;
        hello = false;
        tokio::select! {
            _ = tokio::time::sleep(ANNOUNCE_INTERVAL) => {}
            _ = kick.notified() => {}
        }
    }
}

async fn receive_loop(discovery: Arc<UdpSocket>, inner: Arc<Mutex<Inner>>, identity: Identity) {
    let mut buffer = vec![0u8; DATAGRAM_LIMIT];
    loop {
        let (length, from) = match discovery.recv_from(&mut buffer).await {
            Ok(v) => v,
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }
        };
        let Ok(announcement) = serde_json::from_slice::<Announcement>(&buffer[..length]) else {
            continue;
        };
        if announcement.blenderbase != PROTOCOL || announcement.id == identity.instance_id || announcement.id.is_empty() {
            continue;
        }
        let reply = {
            let mut inner = inner.lock().await;
            if announcement.bye {
                inner.peers.remove(&announcement.id);
                None
            } else {
                let hello = announcement.hello;
                inner.peers.insert(
                    announcement.id.clone(),
                    PeerEntry { announcement, address: from.ip(), seen: Instant::now() },
                );
                hello.then(|| announcement_for(&identity, &inner, false))
            }
        };
        if let Some(reply) = reply {
            if let Ok(payload) = serde_json::to_vec(&reply) {
                let _ = discovery.send_to(&payload, from).await;
            }
        }
    }
}

async fn accept_loop(listener: TcpListener, inner: Arc<Mutex<Inner>>) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                tokio::spawn(serve_connection(stream, peer, Arc::clone(&inner)));
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    }
}

/// The request head, as far as this server cares: method, path, and the device header.
#[derive(Debug, PartialEq)]
struct RequestHead {
    method: String,
    path: String,
    device: String,
}

fn parse_request_head(head: &str) -> Option<RequestHead> {
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next()?.split_ascii_whitespace();
    let method = request_line.next()?.to_string();
    let path = request_line.next()?.to_string();
    let version = request_line.next()?;
    if !version.starts_with("HTTP/1.") {
        return None;
    }
    let mut device = String::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case(DEVICE_HEADER) {
                device = header_safe(value.trim());
            }
        }
    }
    Some(RequestHead { method, path, device })
}

async fn write_response(stream: &mut TcpStream, status: u16, reason: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        reason,
        body.len()
    );
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(body).await;
    let _ = stream.shutdown().await;
}

/// Whether an address has used up its wrong names; records this one when `wrong`.
fn refused(inner: &mut Inner, address: IpAddr, wrong: bool) -> bool {
    let now = Instant::now();
    let attempts = inner.wrong_names.entry(address).or_default();
    attempts.retain(|t| now.duration_since(*t) < WRONG_PIN_WINDOW);
    if attempts.len() >= WRONG_PIN_LIMIT {
        return true;
    }
    if wrong {
        attempts.push(now);
    }
    false
}

async fn serve_connection(mut stream: TcpStream, peer: SocketAddr, inner: Arc<Mutex<Inner>>) {
    let mut head = Vec::with_capacity(512);
    let mut chunk = [0u8; 512];
    let complete = tokio::time::timeout(REQUEST_TIMEOUT, async {
        loop {
            let read = match stream.read(&mut chunk).await {
                Ok(0) | Err(_) => return false,
                Ok(n) => n,
            };
            head.extend_from_slice(&chunk[..read]);
            if head.windows(4).any(|w| w == b"\r\n\r\n") {
                return true;
            }
            if head.len() > REQUEST_LIMIT {
                return false;
            }
        }
    })
    .await
    .unwrap_or(false);
    if !complete {
        return;
    }
    let Some(request) = parse_request_head(&String::from_utf8_lossy(&head)) else {
        write_response(&mut stream, 400, "Bad Request", b"").await;
        return;
    };
    if request.method != "GET" {
        write_response(&mut stream, 405, "Method Not Allowed", b"").await;
        return;
    }
    let Some(name) = request.path.strip_prefix("/share/") else {
        write_response(&mut stream, 404, "Not Found", b"").await;
        return;
    };
    let served = {
        let mut inner = inner.lock().await;
        let matched = inner.share.as_ref().map(|share| share.keys.id == name);
        match matched {
            None => Err((404, "Not Found")),
            Some(true) => {
                if refused(&mut inner, peer.ip(), false) {
                    Err((429, "Too Many Requests"))
                } else {
                    let share = inner.share.as_ref().expect("checked above");
                    Ok((share.file.clone(), share.summary.size))
                }
            }
            Some(false) => {
                if refused(&mut inner, peer.ip(), true) {
                    Err((429, "Too Many Requests"))
                } else {
                    Err((404, "Not Found"))
                }
            }
        }
    };
    let (file, size) = match served {
        Ok(v) => v,
        Err((status, reason)) => {
            write_response(&mut stream, status, reason, b"").await;
            return;
        }
    };
    let Ok(mut file) = tokio::fs::File::open(&file).await else {
        write_response(&mut stream, 500, "Internal Server Error", b"").await;
        return;
    };
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        size
    );
    if stream.write_all(head.as_bytes()).await.is_err() {
        return;
    }
    let delivered = matches!(tokio::io::copy(&mut file, &mut stream).await, Ok(n) if n == size);
    let _ = stream.shutdown().await;
    if delivered {
        let mut inner = inner.lock().await;
        if let Some(share) = inner.share.as_mut() {
            let device = if request.device.is_empty() { peer.ip().to_string() } else { request.device };
            share.received_by.retain(|r| r.device != device);
            share.received_by.push(LanReceipt {
                device,
                at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pin_is_six_digits_however_it_is_typed() {
        assert_eq!(normalise_pin("483921").unwrap(), "483921");
        assert_eq!(normalise_pin(" 483 921 ").unwrap(), "483921");
        assert_eq!(normalise_pin("483-921").unwrap(), "483921");
        assert_eq!(normalise_pin("000001").unwrap(), "000001");
        assert!(normalise_pin("48392").is_err());
        assert!(normalise_pin("4839211").is_err());
        assert!(normalise_pin("48392a").is_err());
        assert!(normalise_pin("").is_err());
        for _ in 0..50 {
            let pin = generate_pin();
            assert_eq!(pin.len(), PIN_DIGITS);
            assert!(pin.bytes().all(|b| b.is_ascii_digit()));
        }
    }

    #[test]
    fn salts_survive_the_hex_trip_and_bad_hex_is_refused() {
        let salt = [0u8, 1, 2, 254, 255, 16, 32, 64, 128, 7, 9, 11, 13, 17, 19, 23];
        assert_eq!(hex_decode(&hex_encode(&salt)).unwrap(), salt.to_vec());
        assert!(hex_decode("abc").is_none());
        assert!(hex_decode("zz").is_none());
        assert!(hex_decode("").is_none());
    }

    #[test]
    fn the_request_parser_reads_what_the_server_needs() {
        let head = "GET /share/abc HTTP/1.1\r\nHost: x\r\nX-Blenderbase-Device: Studio PC\r\n\r\n";
        assert_eq!(
            parse_request_head(head).unwrap(),
            RequestHead { method: String::from("GET"), path: String::from("/share/abc"), device: String::from("Studio PC") }
        );
        assert!(parse_request_head("nonsense").is_none());
        assert!(parse_request_head("GET /x SMTP/1\r\n\r\n").is_none());
        assert_eq!(header_safe("Ünïcode\u{7}名前"), "ncode");
        assert_eq!(header_safe("\u{1F600}"), "Another computer");
    }

    #[test]
    fn five_wrong_names_from_one_address_lock_it_out_for_the_window() {
        let mut inner = Inner::default();
        let address: IpAddr = "192.168.0.9".parse().unwrap();
        let other: IpAddr = "192.168.0.10".parse().unwrap();
        for _ in 0..WRONG_PIN_LIMIT {
            assert!(!refused(&mut inner, address, true));
        }
        assert!(refused(&mut inner, address, true));
        assert!(refused(&mut inner, address, false), "the right name is refused too while locked out");
        assert!(!refused(&mut inner, other, false));
        // The window passing frees the address again.
        inner.wrong_names.get_mut(&address).unwrap().iter_mut().for_each(|t| *t -= WRONG_PIN_WINDOW + Duration::from_secs(1));
        assert!(!refused(&mut inner, address, false));
    }

    #[test]
    fn announcements_are_small_and_tolerate_missing_fields() {
        let full = Announcement {
            blenderbase: PROTOCOL,
            id: uuid::Uuid::new_v4().to_string(),
            device: String::from("A device name of a plausible length"),
            app: String::from("1.2.8"),
            platform: String::from("windows-x86_64"),
            port: 65535,
            hello: true,
            bye: false,
            share: Some(LanShareSummary {
                salt: hex_encode(&[0xab; SALT_BYTES]),
                size: u64::MAX,
                versions: 99,
                series: 99,
                saved: chrono::Utc::now().to_rfc3339(),
            }),
        };
        let bytes = serde_json::to_vec(&full).unwrap();
        assert!(bytes.len() < DATAGRAM_LIMIT / 2, "{} bytes", bytes.len());
        let minimal: Announcement = serde_json::from_str(r#"{"blenderbase":1,"id":"x"}"#).unwrap();
        assert_eq!(minimal.port, 0);
        assert!(minimal.share.is_none());
        assert!(!minimal.bye);
    }

    /// Two instances in one process, on this computer's real interfaces: one shares a file, the
    /// other finds it and fetches it with the PIN, then fails with a wrong PIN and is locked out
    /// after five. Needs multicast or broadcast on loopback, so it is not part of the normal run.
    #[tokio::test]
    #[ignore]
    async fn two_instances_on_one_computer_find_each_other_and_hand_a_file_over() {
        let work = std::env::temp_dir().join(format!("blenderbase-lan-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&work).unwrap();
        let bundle = work.join("setup.bbsetup");
        let content: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&bundle, &content).unwrap();

        let sharer = LanHub::new(String::from("test-sharer"));
        let browser = LanHub::new(String::from("test-browser"));
        let status = sharer.start_share(&bundle, 3, 2).await.unwrap();
        let pin = status.share.as_ref().unwrap().pin.clone();
        browser.set_browsing(true).await.unwrap();

        let mut found = None;
        for _ in 0..100 {
            let status = browser.status().await;
            if let Some(peer) = status.peers.iter().find(|p| p.id == sharer.instance_id) {
                found = Some(peer.clone());
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let peer = found.expect("the browsing instance sees the sharing one within ten seconds");
        assert_eq!(peer.share.as_ref().unwrap().versions, 3);
        assert_eq!(peer.share.as_ref().unwrap().size, sharer.status().await.share.unwrap().size);

        let client = reqwest::Client::new();
        let progress: TransferProgress = Arc::new(|line| println!("  {}", line));
        let received = work.join("received.bbsetup");
        let device = browser.receive(&client, &peer.id, &pin, &received, &progress).await.unwrap();
        assert_eq!(device, sharer.device);
        assert_eq!(std::fs::read(&received).unwrap(), content);
        let receipts = sharer.status().await.share.unwrap().received_by;
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].device, header_safe(&browser.device));

        let wrong = if pin == "000000" { "000001" } else { "000000" };
        for attempt in 0..WRONG_PIN_LIMIT {
            let error = browser.receive(&client, &peer.id, wrong, &received, &progress).await.unwrap_err();
            assert!(error.contains("PIN is wrong"), "attempt {}: {}", attempt, error);
        }
        let error = browser.receive(&client, &peer.id, wrong, &received, &progress).await.unwrap_err();
        assert!(error.contains("Too many wrong PINs"), "{}", error);
        let error = browser.receive(&client, &peer.id, &pin, &received, &progress).await.unwrap_err();
        assert!(error.contains("Too many wrong PINs"), "the right PIN is refused too while locked out: {}", error);

        sharer.stop_share().await;
        for _ in 0..50 {
            if browser.status().await.peers.iter().all(|p| p.id != sharer.instance_id) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert!(browser.status().await.peers.iter().all(|p| p.id != sharer.instance_id), "a bye removes the peer at once");
        browser.set_browsing(false).await.unwrap();
        let _ = std::fs::remove_dir_all(&work);
    }

    /// Shares a file for as long as `BLENDERBASE_TEST_SHARE_SECONDS` says (default 120) and
    /// prints the PIN, so the app can be tried against another instance on this computer.
    #[tokio::test]
    #[ignore]
    async fn share_a_file_for_a_while() {
        let seconds: u64 = std::env::var("BLENDERBASE_TEST_SHARE_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
        let bundle = std::env::var("BLENDERBASE_TEST_SHARE_FILE").expect("BLENDERBASE_TEST_SHARE_FILE names the .bbsetup to share");
        let hub = LanHub::new(String::from("test-sharer"));
        let status = hub.start_share(Path::new(&bundle), 1, 1).await.unwrap();
        println!("SHARING pin={} device={}", status.share.as_ref().unwrap().pin, hub.device);
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(seconds) {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let status = hub.status().await;
            println!(
                "peers={} received_by={:?}",
                status.peers.iter().map(|p| format!("{}@{}:{}", p.device, p.address, p.port)).collect::<Vec<_>>().join(","),
                status.share.as_ref().map(|s| s.received_by.iter().map(|r| r.device.clone()).collect::<Vec<_>>())
            );
        }
        hub.shutdown().await;
    }

    /// Looks for a sharing instance for up to `BLENDERBASE_TEST_SHARE_SECONDS` seconds and
    /// receives with `BLENDERBASE_TEST_PIN`; the mirror image of the test above for the app's share.
    #[tokio::test]
    #[ignore]
    async fn receive_from_whoever_shares() {
        let seconds: u64 = std::env::var("BLENDERBASE_TEST_SHARE_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
        let pin = std::env::var("BLENDERBASE_TEST_PIN").expect("BLENDERBASE_TEST_PIN is the PIN the sharing app shows");
        let hub = LanHub::new(String::from("test-receiver"));
        hub.set_browsing(true).await.unwrap();
        let started = Instant::now();
        let mut sharing = None;
        while started.elapsed() < Duration::from_secs(seconds) {
            let status = hub.status().await;
            if let Some(peer) = status.peers.iter().find(|p| p.share.is_some()) {
                sharing = Some(peer.clone());
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let peer = sharing.expect("a sharing instance shows up");
        println!("FOUND {} at {}:{} sharing {:?}", peer.device, peer.address, peer.port, peer.share);
        let destination = std::env::temp_dir().join(format!("blenderbase-lan-received-{}.bbsetup", uuid::Uuid::new_v4()));
        let progress: TransferProgress = Arc::new(|line| println!("  {}", line));
        let device = hub.receive(&reqwest::Client::new(), &peer.id, &pin, &destination, &progress).await.unwrap();
        println!("RECEIVED from {} into {} ({} bytes)", device, destination.display(), std::fs::metadata(&destination).map(|m| m.len()).unwrap_or(0));
        hub.shutdown().await;
    }
}
