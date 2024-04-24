{
  description = "Bevy bevy bevy";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit overlays system; };
        rust = pkgs.rust-bin.stable.latest.default;
        deps = with pkgs; [
            pkg-config
            udev alsa-lib vulkan-loader
            # To use the x11 feature
            xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr 
            # To use the wayland feature
            libxkbcommon wayland
            # Fater linking
            lld
        ];
      in {
        # The rust package, use `nix build` to build
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "yeet";
          version = "0.0.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = deps;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;
        };

        # This makes sure we can build for WASM
        # Remember to add necessary changes made in defaultPackage to devShell
        devShell = pkgs.mkShell {
          packages = [
            rust
            pkgs.ldtk
          ] ++ deps;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;
        };
      }
    );
}
