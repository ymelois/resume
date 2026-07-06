{
  description = "my resume framework development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      forAllSystems =
        systems: f:
        lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ fenix.overlays.default ];
            }
          )
        );

      mkRustToolchain =
        pkgs:
        pkgs.fenix.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-Di+IXIUa+MEPYM7pUUjYmgR25SLFbGF3SEsK4DSoY6c=";
        };
    in
    {
      devShells = forAllSystems supportedSystems (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = [
            (mkRustToolchain pkgs)

            # Web server for development
            pkgs.miniserve
          ];
        };
      });
    };
}
