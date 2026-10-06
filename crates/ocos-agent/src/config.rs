//! On-disk node identity, profile, and trust store.

use crate::auth::make_hello;
use anyhow::{Context, Result};
use ocos_capability::{from_host, Capability, GpuInfo, PowerSource, Role, StorageInfo};
use ocos_identity::{DeviceIdentity, TrustStore};
use ocos_protocol::Hello;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NodePaths {
    pub dir: PathBuf,
}

impl NodePaths {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn identity(&self) -> PathBuf {
        self.dir.join("identity.json")
    }

    pub fn config(&self) -> PathBuf {
        self.dir.join("node.json")
    }

    pub fn trust(&self) -> PathBuf {
        self.dir.join("trust.json")
    }

    pub fn pairing(&self) -> PathBuf {
        self.dir.join("pairing.txt")
    }

    pub fn session(&self) -> PathBuf {
        self.dir.join("session.json")
    }
}

/// Operator-facing description of this node. Hardware numbers are probed
/// at runtime unless a GPU or storage device is declared here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub profile: String,
    pub display_name: String,
    pub roles: Vec<Role>,
    pub power: PowerSource,
    pub gpu: Option<GpuInfo>,
    pub storage: Option<StorageInfo>,
    pub display_outputs: u32,
}

impl NodeConfig {
    pub fn phone() -> Self {
        Self {
            profile: "phone".into(),
            display_name: "Pixel 8".into(),
            roles: vec![Role::Identity, Role::Input, Role::Display, Role::Sensors],
            power: PowerSource::Battery { percent: 100 },
            gpu: None,
            storage: None,
            display_outputs: 1,
        }
    }

    pub fn desktop() -> Self {
        Self {
            profile: "desktop".into(),
            display_name: "Desktop".into(),
            roles: vec![Role::Compute, Role::Display, Role::Storage],
            power: PowerSource::Ac,
            gpu: None,
            storage: None,
            display_outputs: 1,
        }
    }
}

pub struct RunningNode {
    pub paths: NodePaths,
    pub identity: DeviceIdentity,
    pub config: NodeConfig,
    pub trust: TrustStore,
    pub pairing_code: Option<String>,
}

impl RunningNode {
    pub fn init(dir: &Path, config: NodeConfig) -> Result<Self> {
        fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        let paths = NodePaths::new(dir);
        let identity = DeviceIdentity::generate();
        identity
            .save(&paths.identity())
            .context("save device identity")?;
        let json = serde_json::to_vec_pretty(&config)?;
        fs::write(paths.config(), json).context("save node.json")?;
        let trust = TrustStore::default();
        trust.save(&paths.trust()).context("save trust store")?;
        Ok(Self {
            paths,
            identity,
            config,
            trust,
            pairing_code: None,
        })
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let paths = NodePaths::new(dir);
        let identity = DeviceIdentity::load(&paths.identity())
            .with_context(|| format!("load identity from {}", paths.identity().display()))?;
        let config_bytes = fs::read(paths.config())
            .with_context(|| format!("load {}", paths.config().display()))?;
        let config: NodeConfig = serde_json::from_slice(&config_bytes)?;
        let trust = TrustStore::load(&paths.trust()).context("load trust store")?;
        let pairing_code = fs::read_to_string(paths.pairing())
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Ok(Self {
            paths,
            identity,
            config,
            trust,
            pairing_code,
        })
    }

    pub fn save_trust(&self) -> Result<()> {
        self.trust
            .save(&self.paths.trust())
            .context("save trust store")?;
        Ok(())
    }

    pub fn set_pairing_code(&mut self, code: String) -> Result<()> {
        fs::write(self.paths.pairing(), &code).context("write pairing code")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(self.paths.pairing(), fs::Permissions::from_mode(0o600));
        }
        self.pairing_code = Some(code);
        Ok(())
    }

    pub fn host_capability(&self) -> Capability {
        from_host(
            &self.identity.node_id,
            &self.config.display_name,
            self.config.roles.clone(),
            self.config.power,
            self.config.gpu.clone(),
            self.config.storage,
            self.config.display_outputs,
        )
    }

    pub fn hello(&self, capability: &Capability) -> Hello {
        make_hello(&self.identity, capability)
    }
}
