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
    pub const GITHUB_RELEASE_URL: &str = "https://github.com/Golem-Base/golembase-op-geth/releases";

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
            url::Url::parse(&format!("{}/tag/{tag}", Self::GITHUB_RELEASE_URL))
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

    /// Checks if the tag exists otherwise downloads the release into a temp directory and verifies the checksum.
    /// Once verified, extracts the tag and moves the contents to `download_dir/tag` and returns the path to the `geth` program.
    fn download_release(download_dir: &path::PathBuf, url: &url::Url) -> io::Result<path::PathBuf> {
        todo!(
            "Check if the tag exists otherwise download the release into a temp dir and verify checksum.
            Once verified, extract the tag and move the contents to download_dir/tag and return the path to the geth program."
        )
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
            self.program = Some(Self::download_release(
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

        cmd.spawn().map(|child| ArkivInstance { child })
    }
}

/// An `op-geth` instance with `arkiv` capabilities. The instance is closed on [`ops::Drop::drop`].
///
/// Construct this using the [`Arkiv`] builder.
#[derive(Debug)]
pub struct ArkivInstance {
    child: process::Child,
}
impl ops::Deref for ArkivInstance {
    type Target = process::Child;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl ops::Drop for ArkivInstance {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            // Attempts SIGTERM before `std::process::Child::kill` which is SIGKILL
            if let Ok(out) = process::Command::new("kill")
                .arg("-SIGTERM")
                .arg(self.child.id().to_string())
                .output()
            {
                if out.status.success() {
                    return;
                }
            }
        }
        if let Err(err) = self.child.kill() {
            eprintln!(
                "arkiv-node-bindings: failed to kill arkiv process ({}): {}",
                self.child.id(),
                err
            );
        }
    }
}
