{
  description = "nix flake for egui-greeter";
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

          cargoLock = {
            lockFile = "${greetd-stub}/Cargo.lock";
          };
        };

        egui-greeter = pkgs.rustPlatform.buildRustPackage rec {
          version = "1.0";
          pname = "egui-greeter";

          src = ./.;

          buildInputs = with pkgs; [
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

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          fixupPhase = ''
            patchelf --set-rpath ${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)} $out/bin/egui-greeter
          '';

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

      in
      {
        packages.egui-greeter = egui-greeter;

        nixosModules.egui-greeter =
          {
            pkgs,
            config,
            lib,
            ...
          }:
          with builtins;
          with lib;
          {
            options.programs.egui-greeter = {
              enable = mkOption {
                description = "enable egui-greeter";
                type = lib.types.bool;
                default = false;
              };

              default_session_name = mkOption {
                description = "default session name";
                type = lib.types.str;
              };

              default_session_command = mkOption {
                description = "default session command";
                type = lib.types.str;
              };

              user = mkOption {
                description = "default session command";
                type = lib.types.str;
              };
            };

            config = mkIf config.programs.egui-greeter.enable {
              environment.etc = {
                "greetd/egui-greeter.json".source = pkgs.writeText "config.json" (
                  lib.generators.toJSON { } {
                    default_session_name = config.mysystem.egui-greeter.default_session_name;
                    default_session_command = config.mysystem.egui-greeter.default_session_command;
                    user = config.mysystem.egui-greeter.user;
                  }
                );
              };

              services.seatd.enable = true;

              services.greetd = {
                enable = true;
                settings = {
                  default_session = {
                    command = "${pkgs.cage}/bin/cage -s -- ${egui-greeter}/bin/egui-greeter";
                    user = "greeter";
                  };
                };
              };
            };
          };

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
