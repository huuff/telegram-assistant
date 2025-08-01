{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/x86_64-linux";
    utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-checks = {
      url = "github:huuff/nix-checks";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
      treefmt,
      pre-commit,
      nix-checks,
      naersk,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustPkgs = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
          targets = [ "x86_64-unknown-linux-musl" ];
        };
        treefmt-build = (treefmt.lib.evalModule pkgs ./treefmt.nix).config.build;
        pre-commit-check = pre-commit.lib.${system}.run {
          src = ./.;
          hooks = import ./pre-commit.nix {
            inherit pkgs rustPkgs;
            treefmt = treefmt-build.wrapper;
          };
        };
        naersk' = pkgs.callPackage naersk {
          cargo = rustPkgs;
          rustc = rustPkgs;
          clippy = rustPkgs;
        };
        inherit (nix-checks.lib.${system}) checks rustChecks;
      in
      {
        checks = {
          formatting = treefmt-build.check self;
          statix = checks.statix ./.;
          deadnix = checks.deadnix ./.;
          flake-checker = checks.flake-checker ./.;
          clippy = (rustChecks { toolchain = rustPkgs; }).clippy ./.;
        };

        # for nix fmt
        formatter = treefmt-build.wrapper;

        packages.default = naersk'.buildPackage {
          src = ./.;
          mode = "build";
        };

        devShells.default =
          with pkgs;
          mkShell {
            inherit (pre-commit-check) shellHook;

            nativeBuildInputs = [
              pkg-config
            ];

            buildInputs = pre-commit-check.enabledPackages ++ [
              openssl # nothing works otherwise

              # rust
              rustPkgs
              cargo-expand # see macro expansions
              cargo-insta # for snapshot tests

              # nix
              nil
              nixfmt-rfc-style

            ];
          };
      }
    )
    // {
      nixosModules = {
        # TODO: this only works for my architecture! That's pretty bad, should I pass the package in an overlay?
        default = import ./service.nix { telegram-assistant = self.packages.x86_64-linux.default; };
      };
    };
}
