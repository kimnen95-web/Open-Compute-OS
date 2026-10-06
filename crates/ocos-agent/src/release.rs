//! The phone image and the desktop image must advertise one Open Compute OS release.

#[cfg(test)]
mod tests {
    use ocos_protocol::{OCOS_VERSION, PROTOCOL_VERSION};
    use serde::Deserialize;
    use std::path::Path;

    #[derive(Debug, Deserialize)]
    struct ReleaseFile {
        ocos_version: String,
        protocol_version: u32,
    }

    #[test]
    fn pixel_and_desktop_images_are_the_same_release() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let phone = read_release(&root.join("os/mobile-shiba/ocos-release.json"));
        let desktop = read_release(&root.join("os/desktop-x86_64/ocos-release.json"));
        assert_eq!(phone.ocos_version, OCOS_VERSION);
        assert_eq!(desktop.ocos_version, OCOS_VERSION);
        assert_eq!(phone.ocos_version, desktop.ocos_version);
        assert_eq!(phone.protocol_version, PROTOCOL_VERSION);
        assert_eq!(desktop.protocol_version, PROTOCOL_VERSION);
    }

    fn read_release(path: &Path) -> ReleaseFile {
        let bytes = std::fs::read(path).unwrap_or_else(|error| {
            panic!("read {}: {error}", path.display());
        });
        serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            panic!("parse {}: {error}", path.display());
        })
    }
}
