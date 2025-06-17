{
  description = "DevShell for egui";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    greetd-stub = {
      url = "github:apognu/greetd-stub";
      flake = false;
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      greetd-stub,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        stub-server = pkgs.rustPlatform.buildRustPackage rec {
          version = "1.0";
          pname = "greetd-stub";

          src = "${greetd-stub}";

          # buildInputs = with pkgs; [
          #   cairo
          #   gtk4
          #   atk
          #   glib
          #   gobject-introspection
          #   pango
          #   gdk-pixbuf
          #   graphene
          #   gtk4-layer-shell
          # ]; // missing some dependencies but one of the would require an actual fingerprint scanner which I don't have

          # nativeBuildInputs = with pkgs; [
          #   pkg-config
          # ];

          # buildFeatures = [ "fingerprint" ];

          cargoLock = {
            lockFile = "${greetd-stub}/Cargo.lock";
          };
        };

      in
      {
        devShells.default =
          with pkgs;
          mkShell rec {
            buildInputs = [
              pkg-config
              rustc
              rust-analyzer
              rustfmt
              cargo

              stub-server

              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              libGL
              libxkbcommon
              xorg.libXi
              xorg.libxcb
              libxkbcommon
              vulkan-loader
              wayland
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
            '';
          };
      }
    );
}
