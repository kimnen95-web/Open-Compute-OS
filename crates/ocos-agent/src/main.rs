//! `ocos-agent` — the process that runs on every Open Compute OS node.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ocos_agent::{
    connect_peer, handle_incoming, listen, new_pairing_code, render_link, run_demo, NodeConfig,
    RunningNode,
};
use ocos_protocol::DEFAULT_PORT;
use ocos_session::SessionSnapshot;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ocos-agent",
    version,
    about = "Open Compute OS node agent. One personal computer, many compute nodes."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Profile {
    Phone,
    Desktop,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a device identity and a phone or desktop profile.
    Init {
        /// Directory for identity.json, node.json, and trust.json.
        #[arg(long)]
        data_dir: PathBuf,
        /// Phone is the identity anchor. Desktop is the compute and display node.
        #[arg(long)]
        profile: Profile,
    },
    /// Listen for another node. Prints a pairing code until that node is trusted.
    Serve {
        #[arg(long)]
        data_dir: PathBuf,
        /// TCP address. Use the USB-NCM link-local address on a real node.
        #[arg(long, default_value_t = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT)))]
        listen: SocketAddr,
    },
    /// Connect to a node. Pass --code the first time.
    Connect {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        peer: SocketAddr,
        /// Six-digit code printed by the other node's `serve` command.
        #[arg(long)]
        code: Option<String>,
    },
    /// Run the Pixel 8 + Ryzen reference scenario on this machine.
    Demo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { data_dir, profile } => {
            let config = match profile {
                Profile::Phone => NodeConfig::phone(),
                Profile::Desktop => NodeConfig::desktop(),
            };
            let node = RunningNode::init(&data_dir, config)?;
            println!("created {}", node.identity.node_id);
            println!("profile files in {}", data_dir.display());
        }
        Command::Serve {
            data_dir,
            listen: address,
        } => {
            let mut node = RunningNode::load(&data_dir)?;
            let code = new_pairing_code();
            node.set_pairing_code(code.clone())?;
            let capability = node.host_capability();
            let session = load_session(&node)?;
            println!("Open Compute OS agent");
            println!("node {}", node.identity.node_id);
            println!("name {}", capability.name);
            println!("listen {address}");
            println!("pairing code {code}");
            println!(
                "architecture {:?}  memory {} bytes",
                capability.architecture, capability.memory.total_bytes
            );
            let listener = listen(address).await?;
            loop {
                let (stream, peer) = listener.accept().await?;
                println!("connection from {peer}");
                match handle_incoming(stream, &mut node, &capability, &session).await {
                    Ok(report) => print!("{}", render_link(&report)),
                    Err(error) => eprintln!("connection closed: {error:#}"),
                }
            }
        }
        Command::Connect {
            data_dir,
            peer,
            code,
        } => {
            let mut node = RunningNode::load(&data_dir)?;
            let capability = node.host_capability();
            let report = connect_peer(&mut node, peer, code.as_deref(), &capability).await?;
            print!("{}", render_link(&report));
        }
        Command::Demo => {
            let report = run_demo().await?;
            println!("{report}");
            if !report.mutual_trust || !report.desktop_mode || !report.pooled_memory_rejected {
                anyhow::bail!("demo did not meet the session and placement checks");
            }
            if report.notes_draft != ocos_agent::DEMO_NOTES_DRAFT || report.notes_cursor != 42 {
                anyhow::bail!("notes session was not preserved");
            }
        }
    }
    Ok(())
}

fn load_session(node: &RunningNode) -> Result<SessionSnapshot> {
    let path = node.paths.session();
    if path.exists() {
        let bytes = fs::read(&path)?;
        return Ok(serde_json::from_slice(&bytes)?);
    }
    Ok(SessionSnapshot::new("ses_local", &node.identity.node_id))
}
