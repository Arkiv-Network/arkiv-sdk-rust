#![cfg(feature = "node-bindings")]
//! Used for running a `geth` node with `arkiv` capabilities locally
//! for development and testing purposes. Constructor semantics work similarly to
//! `alloy::node_bindings::Anvil` and `alloy::node_bindings::Geth`.

use std::{
    fs,
    io::{self, BufRead},
    ops, path, process, thread, time,
};

pub use util::Tag;

/// Builder for launching a local `geth` node with `arkiv` capabilities.
///
/// # Example
///
/// Assuming a `geth` executable with `arkiv` capabilities is on `$PATH`, the following is equivalent to running
///
/// ```sh
/// geth --dev \
///   --networkid 1337 \
///   --http --http.api 'eth,web3,net,debug,arkiv' \
///   --http.addr '0.0.0.0' --http.port 8545 \
///   --http.corsdomain '*' --http.vhosts '*' \
///   --ws --ws.api 'eth,web3,net,debug,arkiv' \
///   --ws.addr '0.0.0.0' --ws.port 8546 \
///   --datadir './geth_data' --verbosity 3
/// ```
///
/// > [`Arkiv::fetch_tag`] can be used to fetch and run `geth` from the Arkiv-Network GitHub releases.
///
/// ```rust,ignore
/// use arkiv_sdk::node_bindings::Arkiv;
///
/// let arkiv = Arkiv::default().spawn()?;
///
/// drop(arkiv); // kill the child process
/// ```
#[derive(Debug, Clone)]
#[must_use = "This builder struct does nothing unless it is spawned: `.spawn()`"]
pub struct Arkiv {
    /// The path to the program to execute. Defaults to [`Self::PROGRAM`].
    program: Option<path::PathBuf>,
    /// Whether to launch the `geth` instance in `--dev` mode.
    dev: bool,
    /// Whether to pass the `--http` flag to the `geth` instance.
    http: bool,
    /// The `--http.api` which will be used when the `geth` instance is launched.
    http_api: Option<String>,
    /// The `--http.addr` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_ADDR`].
    http_addr: Option<String>,
    /// The `--http.port` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_HTTP_PORT`].
    http_port: Option<u16>,
    /// Comma separated list of domains from which to accept cross origin requests.
    http_corsdomain: Option<String>,
    /// Comma separated list of virtual hostnames from which to accept requests.
    http_vhosts: Option<String>,
    /// Whether to pass the `--ws` flag to the `geth` instance.
    ws: bool,
    /// The `--ws.api` which will be used when the `geth` instance is launched.
    ws_api: Option<String>,
    /// The `--ws.addr` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_ADDR`].
    ws_addr: Option<String>,
    /// The `--ws.port` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_WS_PORT`].
    ws_port: Option<u16>,
    /// The `--networkid` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to `0` when unset.
    networkid: Option<u64>,
    /// Data directory for the databases and keystore.
    datadir: Option<path::PathBuf>,
    /// Clears the datadir on node shutdown.
    ephemeral_datadir: bool,
    /// The `--verbosity` level which will be used when the `geth` instance is launched.
    verbosity: Option<u8>,
    /// Whether to reattach the stderr handle from the `geth` node.
    keep_stderr: bool,
    /// The URL of the tagged release to fetch.
    release_url: Option<String>,
    /// The directory to cache tagged releases once downloaded and checksum verified.
    ///
    /// Defaults to `$XDG_CONFIG_HOME/arkiv/<TAG>`.
    download_dir: Option<path::PathBuf>,
}
impl Default for Arkiv {
    /// Constructs an [`Arkiv`] builder with an ephemeral datadir and options that produce the following command:
    ///
    /// ```sh
    /// geth --dev \
    ///   --networkid 1337 \
    ///   --http --http.api 'eth,web3,net,debug,arkiv' \
    ///   --http.addr '0.0.0.0' --http.port 8545 \
    ///   --http.corsdomain '*' --http.vhosts '*' \
    ///   --ws --ws.api 'eth,web3,net,debug,arkiv' \
    ///   --ws.addr '0.0.0.0' --ws.port 8546 \
    ///   --datadir './geth_data' --verbosity 3
    /// ```
    fn default() -> Self {
        Self::new()
            .dev()
            // The networkid is set here, otherwise the networkid will be updated automatically
            // which may cause a race condition for wallets and providers expecting a non-zero chain id
            .networkid(Self::ARKIV_NETWORKID)
            .http_addr("0.0.0.0")
            .http_corsdomain("*")
            .http_vhosts("*")
            .ws_addr("0.0.0.0")
            .datadir("./geth_data")
            .ephemeral_datadir()
            .verbosity(3)
    }
}
impl Arkiv {
    pub const GITHUB_RELEASE_URL: &str =
        "https://api.github.com/repos/Arkiv-Network/arkiv-op-geth/releases";
    pub const PROGRAM: &str = "geth";
    /// This networkid is automatically set internally by the node on startup,
    /// however, we set it explicitly in [`Arkiv::default`] since this may produce
    /// a race condition for wallets, providers and txs when setting the `chain_id` field.
    pub const ARKIV_NETWORKID: u64 = 1337;
    pub const API: &str = "eth,web3,net,debug,arkiv";
    pub const DEFAULT_ADDR: &str = "localhost";
    pub const DEFAULT_HTTP_PORT: u16 = 8545;
    pub const DEFAULT_WS_PORT: u16 = 8546;
    pub const NODE_STARTUP_TIMEOUT: time::Duration = time::Duration::from_secs(10);

    /// Construct an [`Arkiv`] builder with all options unset.
    #[doc(alias = "builder")]
    pub fn new() -> Self {
        Self {
            program: None,
            dev: false,
            http: false,
            http_api: None,
            http_addr: None,
            http_port: None,
            http_corsdomain: None,
            http_vhosts: None,
            ws: false,
            ws_api: None,
            ws_addr: None,
            ws_port: None,
            networkid: None,
            datadir: None,
            ephemeral_datadir: false,
            verbosity: None,
            keep_stderr: false,
            release_url: None,
            download_dir: None,
        }
    }

    /// Sets the option to fetch a prebuilt `op-geth` tagged release from <https://github.com/Arkiv-Network/arkiv-op-geth/releases>.
    /// The release is unpacked at the specified directory provided by [`Arkiv::download_dir`],
    /// otherwise defaulting to `$XDG_CONFIG_HOME/arkiv`, appended by the tag.
    /// The download is not triggered until [`Arkiv::spawn`], and will avoid needless network calls. Successive
    /// calls will not trigger re-downloads unless the tag directory is deleted.
    ///
    /// > NOTE: [`Arkiv::program`] will override this setting, which is useful when developing in offline mode
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
    pub fn fetch_tag(mut self, tag: Tag) -> Self {
        self.release_url = Some(match tag {
            Tag::Latest => format!("{}/latest", Self::GITHUB_RELEASE_URL),
            Tag::Name(tag) => format!("{}/tags/{tag}", Self::GITHUB_RELEASE_URL),
        });
        self
    }

    /// Sets the directory to download an arkiv `op-geth` release.
    ///
    /// Setting this option alone will have no effect. See [`Arkiv::fetch_tag`] for details.
    pub fn download_dir<T: Into<path::PathBuf>>(mut self, download_dir: T) -> Self {
        self.download_dir = Some(download_dir.into());
        self
    }

    /// Overrides the path to the `geth` program.
    pub fn program<T: Into<path::PathBuf>>(mut self, path: T) -> Self {
        self.program = Some(path.into());
        self
    }

    /// Whether to launch the `geth` instance in `--dev` mode.
    pub fn dev(mut self) -> Self {
        self.dev = true;
        self
    }

    /// Whether to pass the `--http` flag to the `geth` instance.
    pub fn http(mut self) -> Self {
        self.http = true;
        self
    }

    /// The `--http.api` which will be used when the `geth` instance is launched.
    pub fn http_api<T: Into<String>>(mut self, api: T) -> Self {
        self.http_api = Some(api.into());
        self
    }

    /// Sets the `--http.addr` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_ADDR`].
    pub fn http_addr<T: Into<String>>(mut self, addr: T) -> Self {
        self.http_addr = Some(addr.into());
        self
    }

    /// Sets the `--http.port` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_HTTP_PORT`].
    pub fn http_port<T: Into<u16>>(mut self, port: T) -> Self {
        self.http_port = Some(port.into());
        self
    }

    /// Comma separated list of domains from which to accept cross origin requests.
    pub fn http_corsdomain<T: Into<String>>(mut self, cors: T) -> Self {
        self.http_corsdomain = Some(cors.into());
        self
    }

    /// Comma separated list of virtual hostnames from which to accept requests.
    pub fn http_vhosts<T: Into<String>>(mut self, vhosts: T) -> Self {
        self.http_vhosts = Some(vhosts.into());
        self
    }

    /// Whether to pass the `--ws` flag to the `geth` instance.
    pub fn ws(mut self) -> Self {
        self.ws = true;
        self
    }

    /// The `--ws.api` which will be used when the `geth` instance is launched.
    pub fn ws_api<T: Into<String>>(mut self, api: T) -> Self {
        self.ws_api = Some(api.into());
        self
    }

    /// Sets the `--ws.addr` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_ADDR`].
    pub fn ws_addr<T: Into<String>>(mut self, addr: T) -> Self {
        self.ws_addr = Some(addr.into());
        self
    }

    /// Sets the `--ws.port` which will be used when the `geth` instance is launched.
    ///
    /// Defaults to [`Self::DEFAULT_WS_PORT`].
    pub fn ws_port<T: Into<u16>>(mut self, port: T) -> Self {
        self.ws_port = Some(port.into());
        self
    }

    /// Sets the `--networkid` which will be used when the `geth` instance is launched.
    #[doc(alias = "chain_id")]
    pub fn networkid(mut self, networkid: u64) -> Self {
        self.networkid = Some(networkid);
        self
    }

    /// Sets the `--datadir` which will be used when the `geth` instance is launched.
    pub fn datadir<T: Into<path::PathBuf>>(mut self, datadir: T) -> Self {
        self.datadir = Some(datadir.into());
        self
    }

    /// Whether to keep the geth data directory.
    pub fn ephemeral_datadir(mut self) -> Self {
        self.ephemeral_datadir = true;
        self
    }

    /// The `--verbosity` level which will be used when the `geth` instance is launched.
    pub fn verbosity(mut self, verbosity: u8) -> Self {
        self.verbosity = Some(verbosity);
        self
    }

    /// Keep the handle to geth's stderr in order to read from it.
    ///
    /// Caution: if the stderr handle isn't used, this can end up blocking.
    pub const fn keep_stderr(mut self) -> Self {
        self.keep_stderr = true;
        self
    }

    /// Consumes the builder and spawns an [`ArkivInstance`].
    ///
    /// By default, it's expected that `geth` is on `$PATH`. The default
    /// behavior can be overriden with [`Arkiv::program`], or [`Arkiv::fetch_tag`].
    /// See [`process::Command::new`] for further details.
    #[track_caller]
    pub fn spawn(mut self) -> io::Result<ArkivInstance> {
        if let Some(url) = self.release_url
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

        let mut cmd = process::Command::new(
            self.program
                .as_deref()
                .map_or_else(|| Self::PROGRAM.as_ref(), path::Path::as_os_str),
        );
        cmd.stderr(process::Stdio::piped());

        if self.dev {
            cmd.arg("--dev");
        }
        if let Some(networkid) = self.networkid {
            cmd.args(["--networkid", networkid.to_string().as_str()]);
        }

        let http_enabled = self.http || self.http_addr.is_some() || self.http_port.is_some();
        let http_addr = self
            .http_addr
            .take()
            .unwrap_or_else(|| Self::DEFAULT_ADDR.to_string());
        let http_port = self.http_port.unwrap_or(Self::DEFAULT_HTTP_PORT);
        if http_enabled {
            cmd.args([
                "--http",
                "--http.api",
                self.http_api.as_deref().unwrap_or(Self::API),
                "--http.addr",
                http_addr.as_str(),
                "--http.port",
                http_port.to_string().as_str(),
            ]);
            if let Some(http_corsdomain) = self.http_corsdomain {
                cmd.args(["--http.corsdomain", http_corsdomain.as_str()]);
            }
            if let Some(http_vhosts) = self.http_vhosts {
                cmd.args(["--http.vhosts", http_vhosts.as_str()]);
            }
        }

        let ws_enabled = self.ws || self.ws_addr.is_some() || self.ws_port.is_some();
        let ws_addr = self
            .ws_addr
            .take()
            .unwrap_or_else(|| Self::DEFAULT_ADDR.to_string());
        let ws_port = self.ws_port.unwrap_or(Self::DEFAULT_WS_PORT);
        if ws_enabled {
            cmd.args([
                "--ws",
                "--ws.api",
                self.ws_api.as_deref().unwrap_or(Self::API),
                "--ws.addr",
                ws_addr.as_str(),
                "--ws.port",
                ws_port.to_string().as_str(),
            ]);
        }

        if let Some(ref datadir) = self.datadir {
            cmd.args(["--datadir".as_ref(), datadir.as_os_str()]);
        }

        if let Some(verbosity) = self.verbosity {
            cmd.args(["--verbosity", verbosity.to_string().as_str()]);
        }

        let cmdstr = format!(
            "{} {}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" "),
        );
        eprintln!("arkiv-node-bindings: executing commmand `{cmdstr}`\n");

        let mut instance = cmd
            .spawn()
            .map(|process| ArkivInstance {
                process,
                http_addr,
                http_port,
                ws_addr,
                ws_port,
                networkid: self.networkid.unwrap_or_default(),
                datadir: self.datadir,
                ephemeral_datadir: self.ephemeral_datadir,
            })
            .map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("arkiv-node-bindings: failed to execute command `{cmdstr}`: {err}"),
                )
            })?;

        let stderr = instance
            .stderr
            .take()
            .ok_or(io::Error::other("failed to get geth stderr handle"))?;

        let start = time::Instant::now();
        let mut reader = io::BufReader::new(stderr);

        let mut ports_started = false;

        loop {
            if start + Self::NODE_STARTUP_TIMEOUT <= time::Instant::now() {
                drop(instance);
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "geth node took too long to start",
                ));
            }

            let mut line = String::with_capacity(120);
            reader.read_line(&mut line)?;

            if line.contains("HTTP server started") && !line.contains("auth=true") {
                ports_started = true;
            }

            if line.contains("Fatal:") {
                drop(instance);
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("geth reported a fatal error: {line}"),
                ));
            }

            // If all ports have started we are ready to be queried.
            if ports_started {
                break;
            }
        }

        if self.keep_stderr {
            // re-attach the stderr handle if requested
            instance.stderr = Some(reader.into_inner());
        } else {
            // We need to consume the stderr otherwise geth is non-responsive and RPC server results
            // in connection refused.
            // See: <https://github.com/alloy-rs/alloy/issues/2091#issuecomment-2676134147>
            thread::spawn(move || {
                let mut buf = String::new();
                loop {
                    let _ = reader.read_line(&mut buf);
                }
            });
        }

        Ok(instance)
    }
}

/// An `op-geth` instance with `arkiv` capabilities. The instance is closed on [`ops::Drop::drop`].
///
/// Construct this using the [`Arkiv`] builder.
#[derive(Debug)]
pub struct ArkivInstance {
    process: process::Child,
    http_addr: String,
    http_port: u16,
    ws_addr: String,
    ws_port: u16,
    networkid: u64,
    datadir: Option<path::PathBuf>,
    ephemeral_datadir: bool,
}
impl ops::Deref for ArkivInstance {
    type Target = process::Child;
    fn deref(&self) -> &Self::Target {
        &self.process
    }
}
impl ops::DerefMut for ArkivInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.process
    }
}
impl ops::Drop for ArkivInstance {
    fn drop(&mut self) {
        if let Err(err) = self.kill() {
            eprintln!(
                "arkiv-node-bindings: failed to kill arkiv process ({}): {}\n",
                self.id(),
                err
            );
        }
        if let Some(ref datadir) = self.datadir
            && self.ephemeral_datadir
        {
            fs::remove_dir_all(datadir).expect("failed to remove datadir");
        }
    }
}
impl ArkivInstance {
    /// Returns the network id of this instance
    #[doc(alias = "chain_id")]
    pub fn networkid(&self) -> u64 {
        self.networkid
    }

    /// Returns the HTTP endpoint of this instance
    #[doc(alias = "http_endpoint")]
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.http_addr, self.http_port)
    }

    /// Returns the Websocket endpoint of this instance
    pub fn ws_endpoint(&self) -> String {
        format!("ws://{}:{}", self.ws_addr, self.ws_port)
    }

    /// Returns the HTTP endpoint url of this instance
    #[doc(alias = "http_endpoint_url")]
    pub fn endpoint_url(&self) -> reqwest::Url {
        reqwest::Url::parse(&self.endpoint()).unwrap()
    }

    /// Returns the Websocket endpoint url of this instance
    pub fn ws_endpoint_url(&self) -> reqwest::Url {
        reqwest::Url::parse(&self.ws_endpoint()).unwrap()
    }
}

mod util {
    //! Handlers for downloading, verifying and managing release binaries used by [`crate::node_bindings::Arkiv`].
    use std::{env, fs, io, path};

    use alloy::signers::k256::sha2::{Digest, Sha256};
    use serde::Deserialize;

    use crate::node_bindings::Arkiv;

    /// The release tag to fetch from GitHub.
    ///
    /// A list of tags can be found at <https://github.com/Arkiv-Network/arkiv-op-geth/tags>.
    #[derive(Debug, Clone)]
    pub enum Tag {
        /// The latest available tag.
        Latest,
        /// Specify a tag by name.
        Name(String),
    }

    #[derive(Deserialize)]
    pub(in crate::node_bindings) struct Release {
        tag_name: String,
        assets: Vec<Asset>,
    }
    impl Release {
        const ASSET_NAME: &str = "arkiv-op-geth";

        /// Checks if the tag exists otherwise downloads the release into a temp directory and verifies the checksum.
        /// Once verified, extracts the tag and moves the contents to `download_dir/tag` and returns the path to the `geth` program.
        pub(in crate::node_bindings) fn download<U: reqwest::IntoUrl>(
            download_dir: &path::PathBuf,
            url: U,
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
                let geth = download_dir.join(Arkiv::PROGRAM);
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

            Ok(download_dir.join(Arkiv::PROGRAM))
        }
    }
}
