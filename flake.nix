{
  description = "A Rust SDK for interacting with Arkiv.";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";

    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    crane.url = "github:ipetkov/crane";

    arkiv-op-geth.url = "github:arkiv-network/arkiv-op-geth";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
      arkiv-op-geth,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        inherit (pkgs) lib;
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./.rust-toolchain.toml;
          sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src =
          let
            markdownFilter = path: _type: builtins.match ".*md$" path != null;
            jsonFilter = path: _type: builtins.match ".*json" path != null;
            sourceFilter = path: type:
              (markdownFilter path type)
              || (jsonFilter path type)
              || (craneLib.filterCargoSources path type)
              ;
          in
          lib.cleanSourceWith {
            src = ./.;
            filter = sourceFilter;
            name = "source";
          };

        # Run a cargo example from the examples directory.
        runExample = { cargoArtifacts, example, ... }@args:
          craneLib.mkCargoDerivation (args // {
            inherit cargoArtifacts;
            pnameSuffix = "-${example}";
            nativeBuildInputs = (args.nativeBuildInputs or []) ++ [ arkiv-op-geth.packages.${system}.default ];
            buildPhaseCargoCommand = "cargo run --example ${example}";
          });

        commonArgs = {
          inherit src;
          strictDeps = true;

          # TODO: Check dependencies for rustls, we can potentially
          # remove the dependency on pkg-config and openssl
          nativeBuildInputs = with pkgs; [ pkg-config ];

          buildInputs = with pkgs; [ openssl ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        arkiv-sdk = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false; # The tests don't work in the nix sandbox
          }
        );
      in
      {
        # Additional cargo checks can be found at https://github.com/ipetkov/crane.
        #
        # These derivations are inherited by the `devShell`.
        checks = {
          inherit arkiv-sdk;

          cargo-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          cargo-fmt = craneLib.cargoFmt {
            inherit src;
          };

          taplo-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
          };

          cargo-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoDocExtraArgs = "--no-deps --workspace";
              # This can be commented out or tweaked as necessary, e.g. set to
              # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          cargo-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );

          transactions-example = runExample (commonArgs // {
            inherit cargoArtifacts;
            example = "transactions";
          });

          keystore-signer-example = runExample (commonArgs // {
            inherit cargoArtifacts;
            example = "keystore_signer";
          });

          subscriptions-example = runExample (commonArgs // {
            inherit cargoArtifacts;
            example = "subscriptions";
          });
        };

        packages = {
          inherit arkiv-sdk;
          default = arkiv-sdk;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages =
            with pkgs;
            [
              pre-commit
              nixfmt-rfc-style
              mitmproxy
            ]
            ++ lib.optionals (!pkgs.stdenv.isDarwin) [
              nil # currently requires compiling the world
            ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
