{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rustc-codegen-cranelift-preview"
          ];
        });
        buildInputs = with pkgs; [
            # Bevy Dependencies
            alsa-lib
            udev
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            libxkbcommon
            wayland
            rust
        ];
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
            eza
            fd
          ] ++ buildInputs;

          shellHook = ''
            alias ls=eza
            alias find=fd
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
          '';
        };
      }
    );
}
