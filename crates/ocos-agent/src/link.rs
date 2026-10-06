//! One TCP session between two agents.
//!
//! The listener is normally the phone. The connector is normally the desktop.
//! Roles come from the capability descriptor, not from which side dialed.

use crate::auth::verify_hello;
use crate::config::RunningNode;
use anyhow::{anyhow, bail, Context, Result};
use ocos_capability::Capability;
use ocos_identity::{pairing_codes_match, TrustedPeer};
use ocos_protocol::{
    decode_body, encode_frame, frame_body_len, DesktopMode, Message, Pair, PairResult,
    SessionAccept, SessionOffer,
};
use ocos_session::SessionSnapshot;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, PartialEq)]
pub struct LinkReport {
    pub peer_node_id: String,
    pub peer_name: String,
    pub trusted: bool,
    pub desktop_mode: bool,
    pub session: Option<SessionSnapshot>,
}

/// Accept a single connection. `serve` calls this once per TCP session.
pub async fn handle_incoming(
    stream: TcpStream,
    node: &mut RunningNode,
    capability: &Capability,
    session: &SessionSnapshot,
) -> Result<LinkReport> {
    let (mut read, mut write) = stream.into_split();
    let peer_hello = expect_hello(recv(&mut read).await?)?;
    verify_hello(&peer_hello)?;
    send(
        &mut write,
        &Message::Hello(Box::new(node.hello(capability))),
    )
    .await?;

    if !node.trust.contains(&peer_hello.node_id) {
        let pair = expect_pair(recv(&mut read).await?)?;
        let expected = node
            .pairing_code
            .as_deref()
            .ok_or_else(|| anyhow!("this node is not accepting a pairing code"))?;
        if !pairing_codes_match(expected, &pair.code) {
            send(
                &mut write,
                &Message::PairResult(PairResult {
                    accepted: false,
                    reason: "pairing code rejected".into(),
                }),
            )
            .await?;
            bail!("rejected pairing attempt from {}", peer_hello.node_id);
        }
        remember_peer(
            node,
            &peer_hello.node_id,
            &peer_hello.public_key,
            &peer_hello.capability.name,
        )?;
        send(
            &mut write,
            &Message::PairResult(PairResult {
                accepted: true,
                reason: "trusted".into(),
            }),
        )
        .await?;
    }

    let mut report = LinkReport {
        peer_node_id: peer_hello.node_id.clone(),
        peer_name: peer_hello.capability.name.clone(),
        trusted: true,
        desktop_mode: false,
        session: Some(session.clone()),
    };

    if capability.is_identity_anchor() && peer_hello.capability.is_desktop_target() {
        send(
            &mut write,
            &Message::SessionOffer(Box::new(SessionOffer {
                snapshot: session.clone(),
            })),
        )
        .await?;
        let accept = expect_accept(recv(&mut read).await?)?;
        if !accept.preserved || accept.session_id != session.session_id {
            bail!("desktop did not preserve session {}", session.session_id);
        }
        let mode = expect_desktop(recv(&mut read).await?)?;
        if !mode.active || mode.session_id != session.session_id {
            bail!("desktop mode was not activated");
        }
        if mode.display_node_id != peer_hello.node_id {
            bail!("desktop mode names a different node than the peer");
        }
        report.desktop_mode = true;
        report.session = Some(session.open_on_desktop(&peer_hello.node_id));
    }

    Ok(report)
}

/// Dial `peer`, pair if `code` is set, and adopt Desktop Mode when the far
/// side is the identity anchor and this node is a desktop target.
pub async fn connect_peer(
    node: &mut RunningNode,
    peer: SocketAddr,
    code: Option<&str>,
    capability: &Capability,
) -> Result<LinkReport> {
    let stream = TcpStream::connect(peer)
        .await
        .with_context(|| format!("connect to {peer}"))?;
    let (mut read, mut write) = stream.into_split();
    send(
        &mut write,
        &Message::Hello(Box::new(node.hello(capability))),
    )
    .await?;
    let peer_hello = expect_hello(recv(&mut read).await?)?;
    verify_hello(&peer_hello)?;

    if !node.trust.contains(&peer_hello.node_id) {
        let code = code.ok_or_else(|| {
            anyhow!(
                "peer {} is not trusted; pass the pairing code shown on that node",
                peer_hello.node_id
            )
        })?;
        send(
            &mut write,
            &Message::Pair(Pair {
                code: code.to_string(),
            }),
        )
        .await?;
        let result = expect_pair_result(recv(&mut read).await?)?;
        if !result.accepted {
            bail!("pairing refused: {}", result.reason);
        }
        remember_peer(
            node,
            &peer_hello.node_id,
            &peer_hello.public_key,
            &peer_hello.capability.name,
        )?;
    }

    let mut report = LinkReport {
        peer_node_id: peer_hello.node_id.clone(),
        peer_name: peer_hello.capability.name.clone(),
        trusted: true,
        desktop_mode: false,
        session: None,
    };

    if peer_hello.capability.is_identity_anchor() && capability.is_desktop_target() {
        let offer = expect_offer(recv(&mut read).await?)?;
        let opened = offer.snapshot.open_on_desktop(&capability.node_id);
        let body = serde_json::to_vec_pretty(&opened)?;
        std::fs::write(node.paths.session(), body)
            .with_context(|| format!("store session in {}", node.paths.session().display()))?;
        send(
            &mut write,
            &Message::SessionAccept(SessionAccept {
                session_id: opened.session_id.clone(),
                preserved: true,
            }),
        )
        .await?;
        send(
            &mut write,
            &Message::DesktopMode(DesktopMode {
                active: true,
                session_id: opened.session_id.clone(),
                anchor_node_id: peer_hello.node_id.clone(),
                display_node_id: capability.node_id.clone(),
            }),
        )
        .await?;
        report.desktop_mode = true;
        report.session = Some(opened);
    }

    Ok(report)
}

pub async fn listen(bind: SocketAddr) -> Result<TcpListener> {
    TcpListener::bind(bind)
        .await
        .with_context(|| format!("listen on {bind}"))
}

fn remember_peer(
    node: &mut RunningNode,
    node_id: &str,
    public_key: &str,
    name: &str,
) -> Result<()> {
    node.trust.insert(TrustedPeer {
        node_id: node_id.to_string(),
        public_key: public_key.to_string(),
        name: name.to_string(),
    });
    node.save_trust()
}

async fn send(write: &mut OwnedWriteHalf, message: &Message) -> Result<()> {
    let frame = encode_frame(message)?;
    write.write_all(&frame).await.context("write frame")?;
    write.flush().await.context("flush frame")?;
    Ok(())
}

async fn recv(read: &mut OwnedReadHalf) -> Result<Message> {
    let mut header = [0u8; 4];
    read.read_exact(&mut header)
        .await
        .context("read frame header")?;
    let len = frame_body_len(header)?;
    let mut body = vec![0u8; len];
    read.read_exact(&mut body)
        .await
        .context("read frame body")?;
    Ok(decode_body(&body)?)
}

fn expect_hello(message: Message) -> Result<ocos_protocol::Hello> {
    match message {
        Message::Hello(hello) => Ok(*hello),
        other => bail!("expected hello, got {other:?}"),
    }
}

fn expect_pair(message: Message) -> Result<Pair> {
    match message {
        Message::Pair(pair) => Ok(pair),
        other => bail!("expected pair, got {other:?}"),
    }
}

fn expect_pair_result(message: Message) -> Result<PairResult> {
    match message {
        Message::PairResult(result) => Ok(result),
        other => bail!("expected pair result, got {other:?}"),
    }
}

fn expect_offer(message: Message) -> Result<SessionOffer> {
    match message {
        Message::SessionOffer(offer) => Ok(*offer),
        other => bail!("expected session offer, got {other:?}"),
    }
}

fn expect_accept(message: Message) -> Result<SessionAccept> {
    match message {
        Message::SessionAccept(accept) => Ok(accept),
        other => bail!("expected session accept, got {other:?}"),
    }
}

fn expect_desktop(message: Message) -> Result<DesktopMode> {
    match message {
        Message::DesktopMode(mode) => Ok(mode),
        other => bail!("expected desktop mode, got {other:?}"),
    }
}
