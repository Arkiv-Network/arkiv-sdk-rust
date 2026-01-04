//! Used for running an `op-geth` node with `arkiv` capabilities locally
//! for development and testing purposes. Constructor semantics work similarly to
//! `alloy::node_bindings::Anvil` and `alloy::node_bindings::AnvilInstance`.

use std::{fs, io, ops, path, process};

/// Builder for launching a local `op-geth` node with `arkiv` capabilities.
#[derive(Debug, Clone, Default)]
pub struct Arkiv {
    /// The path to the program to execute. Defaults to [`Self::DEFAULT_PROGRAM`].
    program: Option<path::PathBuf>,
    /// The URL of the tagged release to fetch.
    fetch_url: Option<url::Url>,
    /// The directory to cache tagged releases once downloaded and checksum verified.
    /// Defaults to `$XDG_CONFIG_HOME/arkiv/<TAG>`.
    download_dir: Option<path::PathBuf>,
}
impl Arkiv {
    pub const DEFAULT_PROGRAM: &str = "geth";
    pub const GITHUB_RELEASE_URL: &str =
        "https://api.github.com/repos/Golem-Base/golembase-op-geth/releases";

    /// Sets the option to fetch a prebuilt `op-geth` tagged release from <https://github.com/Golem-Base/golembase-op-geth/releases>.
    /// The binary is placed at the specified directory if provided, otherwise defaulting to `$XDG_CONFIG_HOME/arkiv/<TAG>`.
    /// The download is not triggered until [`Arkiv::spawn`], and will avoid needless network calls. Successive
    /// calls will not trigger re-downloads unless the tag directory is deleted.
    ///
    /// > NOTE: [`Arkiv::path`] will override this setting, which is useful when developing in offline mode
    /// > or testing in sandboxed environments that cannot make network calls.
    ///
    /// If the checksum cannot be verified, the fetched binary will not be executed
    /// and an error will be returned from [`Arkiv::spawn`].
    ///
    /// # Errors
    ///
    /// - If the download directory does not exist, or if the final component of
    /// the path is not a directory. See [`fs::canonicalize`] for further details.
    /// - If the home config directory does not exist, or could not be created. See
    /// [`dirs::config_dir`] and [`fs::create_dir_all`] for further details.
    fn fetch_url(mut self, download_dir: Option<path::PathBuf>, url: url::Url) -> Self {
        self.download_dir = download_dir;
        self.fetch_url = Some(url);
        self
    }

    /// Sets the option to fetch a prebuilt `op-geth` tagged release from <https://github.com/Golem-Base/golembase-op-geth/releases>.
    /// The binary is placed at the specified directory if provided, otherwise defaulting to `$XDG_CONFIG_HOME/arkiv/<TAG>`.
    /// The download is not triggered until [`Arkiv::spawn`], and will avoid needless network calls. Successive
    /// calls will not trigger re-downloads unless the tag directory is deleted.
    ///
    /// > NOTE: [`Arkiv::path`] will override this setting, which is useful when developing in offline mode
    /// > or testing in sandboxed environments that cannot make network calls.
    ///
    /// If the checksum cannot be verified, the fetched binary will not be executed
    /// and an error will be returned from [`Arkiv::spawn`].
    ///
    /// # Errors
    ///
    /// - If the download directory does not exist, or if the final component of
    /// the path is not a directory. See [`fs::canonicalize`] for further details.
    /// - If the home config directory does not exist, or could not be created. See
    /// [`dirs::config_dir`] and [`fs::create_dir_all`] for further details.
    pub fn fetch_tag(self, download_dir: Option<path::PathBuf>, tag: &str) -> Self {
        self.fetch_url(
            download_dir,
            url::Url::parse(&format!("{}/tags/{tag}", Self::GITHUB_RELEASE_URL))
                .expect("failed to parse tagged release url"),
        )
    }

    /// Sets the option to fetch the latest prebuilt `op-geth` tagged release from <https://github.com/Golem-Base/golembase-op-geth/releases>.
    /// The binary is placed at the specified directory if provided, otherwise defaulting to `$XDG_CONFIG_HOME/arkiv/<TAG>`.
    /// The download is not triggered until [`Arkiv::spawn`], and will avoid needless network calls. Successive
    /// calls will not trigger re-downloads unless the tag directory is deleted.
    ///
    /// > NOTE: [`Arkiv::path`] will override this setting, which is useful when developing in offline mode
    /// > or testing in sandboxed environments that cannot make network calls.
    ///
    /// If the checksum cannot be verified, the fetched binary will not be executed
    /// and an error will be returned from [`Arkiv::spawn`].
    ///
    /// # Errors
    ///
    /// - If the download directory does not exist, or if the final component of
    /// the path is not a directory. See [`fs::canonicalize`] for further details.
    /// - If the home config directory does not exist, or could not be created. See
    /// [`dirs::config_dir`] and [`fs::create_dir_all`] for further details.
    pub fn fetch_latest(self, download_dir: Option<path::PathBuf>) -> Self {
        self.fetch_url(
            download_dir,
            url::Url::parse(&format!("{}/latest", Self::GITHUB_RELEASE_URL))
                .expect("failed to parse latest release url"),
        )
    }

    /// Overrides the path to the `geth` program.
    pub fn path<T: Into<path::PathBuf>>(mut self, path: T) -> Self {
        self.program = Some(path.into());
        self
    }

    /// Consumes the builder and spawns an [`ArkivInstance`].
    ///
    /// By default, it's expected that `geth` is on `$PATH`. The default
    /// behavior can be overriden with [`Arkiv::path`], [`Arkiv::fetch_tag`] or [`Arkiv::fetch_latest`].
    /// See [`process::Command::new`] for further details.
    pub fn spawn(mut self) -> io::Result<ArkivInstance> {
        if let Some(url) = self.fetch_url.as_ref()
            && self.program.is_none()
        {
            self.program = Some(util::Release::download(
                &self.download_dir
                    .map_or_else(
                        || match dirs::config_dir() {
                            Some(config_dir) => {
                                let default = config_dir.join("arkiv");
                                fs::create_dir_all(&default).map(|_| default)
                            }
                            None => Err(io::Error::new(
                                io::ErrorKind::NotFound,
                                "config directory does not exist: https://docs.rs/dirs/6.0.0/dirs/fn.config_dir.html",
                            )),
                        },
                        fs::canonicalize,
                    )?,
                url,
            )?);
        }

        let mut cmd = self.program.map_or_else(
            || process::Command::new(Self::DEFAULT_PROGRAM),
            process::Command::new,
        );
        cmd.stdout(process::Stdio::piped())
            .stderr(process::Stdio::inherit());

        cmd.spawn().map(ArkivInstance)
    }
}

/// An `op-geth` instance with `arkiv` capabilities. The instance is closed on [`ops::Drop::drop`].
///
/// Construct this using the [`Arkiv`] builder.
#[derive(Debug)]
pub struct ArkivInstance(process::Child);
impl ops::Deref for ArkivInstance {
    type Target = process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for ArkivInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl ops::Drop for ArkivInstance {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            // Attempts SIGTERM before `std::process::Child::kill` which is SIGKILL
            if let Ok(out) = process::Command::new("kill")
                .arg("-SIGTERM")
                .arg(self.id().to_string())
                .output()
            {
                if out.status.success() {
                    return;
                }
            }
        }
        if let Err(err) = self.kill() {
            eprintln!(
                "arkiv-node-bindings: failed to kill arkiv process ({}): {}",
                self.id(),
                err
            );
        }
    }
}

mod util {
    use serde::Deserialize;
    use std::{io, path};

    #[derive(Deserialize)]
    pub(in crate::node_bindings) struct Release {
        tag_name: String,
        assets: Vec<Asset>,
    }
    impl Release {
        /// Checks if the tag exists otherwise downloads the release into a temp directory and verifies the checksum.
        /// Once verified, extracts the tag and moves the contents to `download_dir/tag` and returns the path to the `geth` program.
        pub fn download(download_dir: &path::PathBuf, url: &url::Url) -> io::Result<path::PathBuf> {
            let client = reqwest::blocking::Client::new();
            let resp = client
                .get(url.as_str())
                .header(reqwest::header::USER_AGENT, "arkiv-sdk")
                .send()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let release = resp
                .json::<Self>()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let asset = release
                .assets
                .iter()
                .find(|asset| {
                    let os = match std::env::consts::OS {
                        "macos" => "darwin",
                        "linux" => "linux",
                        "windows" => "windows",
                        other => panic!("unsupported OS: {other}"),
                    };
                    let arch = match std::env::consts::ARCH {
                        "x86_64" => "amd64",
                        "aarch64" => "arm64",
                        other => panic!("unsupported arch: {other}"),
                    };

                    asset.name.ends_with(".tar.gz")
                        && asset.name.contains(os)
                        && asset.name.contains(arch)
                })
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "No matching tarball found in latest release",
                    )
                })?;

            todo!(
            "Check if the tag exists otherwise download the release into a temp dir and verify checksum.
            Once verified, extract the tag and move the contents to download_dir/tag and return the path to the geth program."
        )
        }
    }

    #[derive(Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
    }
}

// use flate2::read::GzDecoder;
// use reqwest::blocking::Client;
// use sha2::{Digest, Sha256};
// use std::{
//     fs,
//     fs::File,
//     io::{self, Read},
//     path::{Path, PathBuf},
// };
// use tar::Archive;
// use tempfile::tempdir;
// use url::Url;

// /// Download a release into `download_dir/<tag>/` and return the path to the geth executable
// pub fn download_release(download_dir: &Path, url: &Url) -> io::Result<PathBuf> {
//     fs::create_dir_all(download_dir)?;

//     let file_name = url
//         .path_segments()
//         .and_then(|s| s.last())
//         .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid URL: missing filename"))?;

//     let tag = extract_tag_from_filename(file_name)
//         .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Unable to extract tag"))?;

//     // `${download_dir}/v1.2.3`
//     let tag_dir = download_dir.join(&tag);

//     if tag_dir.exists() {
//         return Ok(tag_dir.join("geth"));
//     }

//     let tmp = tempdir()?;
//     let tar_path = tmp.path().join(file_name);

//     download_to_file(url, &tar_path)?;
//     verify_checksum(&tar_path, &url.clone().into_string())?; // checksum URL logic is placeholder
//     extract_tarball(&tar_path, &tag_dir)?;

//     Ok(tag_dir.join("geth"))
// }

// /// Download URL → local file
// fn download_to_file(url: &Url, dest: &Path) -> io::Result<()> {
//     let response = Client::new()
//         .get(url.clone())
//         .send()
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("HTTP Error: {e}")))?;

//     if !response.status().is_success() {
//         return Err(io::Error::new(
//             io::ErrorKind::Other,
//             format!("Failed to download: {}", response.status()),
//         ));
//     }

//     let mut file = File::create(dest)?;
//     let mut content = io::Cursor::new(response.bytes().map_err(|e| {
//         io::Error::new(io::ErrorKind::Other, format!("Read bytes failed: {e}"))
//     })?);
//     io::copy(&mut content, &mut file)?;
//     Ok(())
// }

// /// Verify checksum against a checksum URL or embedded logic
// fn verify_checksum(path: &Path, release_url: &str) -> io::Result<()> {
//     // Example assumption:
//     // If release_url = "https://…/op-geth-v1.2.3.tar.gz" then checksum URL might be "…/op-geth-v1.2.3.sha256"
//     let checksum_url = format!("{release_url}.sha256");

//     let checksum = download_checksum(&checksum_url)?;

//     let mut file = File::open(path)?;
//     let mut hasher = Sha256::new();
//     let mut buffer = [0u8; 4096];

//     loop {
//         let n = file.read(&mut buffer)?;
//         if n == 0 { break; }
//         hasher.update(&buffer[..n]);
//     }
//     let digest = hasher.finalize();
//     let hex_digest = hex::encode(digest);

//     if hex_digest != checksum {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidData,
//             format!("Checksum mismatch: expected {checksum}, got {hex_digest}"),
//         ));
//     }
//     Ok(())
// }

// fn download_checksum(url: &str) -> io::Result<String> {
//     let resp = Client::new()
//         .get(url)
//         .send()
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Checksum HTTP Error: {e}")))?;

//     if !resp.status().is_success() {
//         return Err(io::Error::new(
//             io::ErrorKind::Other,
//             format!("Failed to get checksum: {}", resp.status()),
//         ));
//     }

//     let text = resp.text().map_err(|e| {
//         io::Error::new(io::ErrorKind::Other, format!("Checksum read failed: {e}"))
//     })?;

//     Ok(text.trim().to_string())
// }

// /// Extract tarball to directory
// fn extract_tarball(tarball: &Path, dest: &Path) -> io::Result<()> {
//     fs::create_dir_all(dest)?;
//     let file = File::open(tarball)?;
//     let mut archive = Archive::new(GzDecoder::new(file));
//     archive.unpack(dest)?;
//     Ok(())
// }

// /// extremely naive tag extraction
// fn extract_tag_from_filename(name: &str) -> Option<String> {
//     // Example: op-geth-v1.2.3-darwin-arm64.tar.gz → v1.2.3
//     name.split('-')
//         .find(|s| s.starts_with('v') && s.chars().nth(1).map(|c| c.is_numeric()).unwrap_or(false))
//         .map(|v| v.to_string())
// }
