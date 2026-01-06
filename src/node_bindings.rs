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
                .expect("Failed to parse tagged release url"),
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
                .expect("Failed to parse latest release url"),
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
        if let Some(url) = self.fetch_url
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
                                "Config directory does not exist: https://docs.rs/dirs/6.0.0/dirs/fn.config_dir.html",
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
    //! Handlers for downloading, verifying and managing release binaries used by [`crate::node_bindings::Arkiv`].

    use std::{env, fs, io, path};

    use alloy::signers::k256::sha2::{Digest, Sha256};
    use serde::Deserialize;

    use crate::node_bindings::Arkiv;

    #[derive(Deserialize)]
    pub(in crate::node_bindings) struct Release {
        tag_name: String,
        assets: Vec<Asset>,
    }
    impl Release {
        const ASSET_NAME: &str = "golembase-op-geth";

        /// Checks if the tag exists otherwise downloads the release into a temp directory and verifies the checksum.
        /// Once verified, extracts the tag and moves the contents to `download_dir/tag` and returns the path to the `geth` program.
        pub(in crate::node_bindings) fn download(
            download_dir: &path::PathBuf,
            url: url::Url,
        ) -> io::Result<path::PathBuf> {
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(url)
                .header(reqwest::header::USER_AGENT, "arkiv-sdk")
                .send()
                .map_err(io::Error::other)?;

            if !response.status().is_success() {
                return Err(io::Error::other(format!(
                    "Failed to fetch release information via github api: {}",
                    response.status()
                )));
            }

            let release = response.json::<Self>().map_err(io::Error::other)?;

            let download_dir = download_dir.join(&release.tag_name);
            {
                // Skip download if this tag already exists
                let geth = download_dir.join(Arkiv::DEFAULT_PROGRAM);
                if geth.exists() {
                    return Ok(geth);
                }
            }

            let os = match env::consts::OS {
                "macos" => "darwin",
                "linux" => "linux",
                "windows" => "windows",
                other => panic!("unsupported OS: {other}"),
            };
            let arch = match env::consts::ARCH {
                "x86_64" => "amd64",
                "aarch64" => "arm64",
                other => panic!("unsupported arch: {other}"),
            };
            let asset_name = |ext: &str| {
                format!(
                    "{}-{}-{os}-{arch}.{ext}",
                    Self::ASSET_NAME,
                    release.tag_name
                )
            };

            let tarname = asset_name("tar.gz");
            let tarball = release
                .assets
                .iter()
                .find(|asset| asset.name == tarname)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "Failed to find tarball asset {tarname}:\nAvailable assets: {:?}",
                            release.assets
                        ),
                    )
                })?;

            let shaname = asset_name("tar.gz.sha256");
            let checksum = release
                .assets
                .iter()
                .find(|asset| asset.name == shaname)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "Failed to find checksum asset {shaname}:\nAvailable assets: {:?}",
                            release.assets
                        ),
                    )
                })?;

            Asset::download(client, tarball, checksum, download_dir)
        }
    }

    #[derive(Debug, Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
    }
    impl Asset {
        /// Download, verify and unpack an asset, placing it in the download directory and returning its path.
        fn download(
            client: reqwest::blocking::Client,
            tarball: &Asset,
            checksum: &Asset,
            download_dir: path::PathBuf,
        ) -> io::Result<path::PathBuf> {
            let download_tarball = client
                .get(&tarball.browser_download_url)
                .send()
                .map_err(io::Error::other)?;

            if !download_tarball.status().is_success() {
                return Err(io::Error::other(format!(
                    "Failed to download tarball {}: {}",
                    tarball.browser_download_url,
                    download_tarball.status()
                )));
            }

            let tmp = tempfile::tempdir()?;
            let tarpath = tmp.path().join(&tarball.name);
            let mut tarfile = fs::File::create(&tarpath)?;
            let mut content = io::Cursor::new(download_tarball.bytes().map_err(io::Error::other)?);
            io::copy(&mut content, &mut tarfile)?;

            let download_checksum = client
                .get(&checksum.browser_download_url)
                .send()
                .map_err(io::Error::other)?;

            if !download_checksum.status().is_success() {
                return Err(io::Error::other(format!(
                    "Failed to download checksum {}: {}",
                    checksum.browser_download_url,
                    download_checksum.status()
                )));
            }

            let checksum = download_checksum.text().map_err(io::Error::other)?;

            let mut tarfile = fs::File::open(tarpath)?;
            let mut hasher = Sha256::new();
            io::copy(&mut tarfile, &mut hasher)?;

            let hash = hex::encode(hasher.finalize());

            if checksum != hash {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Checksum mismatch! This could be an indicator of a man-in-the-middle attack.\nExpected: {checksum}\nGot:     {hash}"
                    ),
                ));
            }

            let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarfile));
            archive.unpack(&download_dir)?;

            Ok(download_dir.join(Arkiv::DEFAULT_PROGRAM))
        }
    }
}
