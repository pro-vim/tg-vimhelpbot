{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # Bulding Rust
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    # Tags sources
    vim = {
      url = "github:vim/vim";
      flake = false;
    };
  };
  outputs =
    { self
    , nixpkgs
    , flake-utils
    , vim
    , crane
    , fenix
    , advisory-db
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pname = "tg-vimhelpbot";

      pkgs = import nixpkgs { inherit system; };

      rust = fenix.packages."${system}".stable.toolchain;
      crane-lib = (crane.mkLib pkgs).overrideToolchain rust;
      src = crane-lib.cleanCargoSource ./.;
      cargoArtifacts = crane-lib.buildDepsOnly { inherit src; };
      crate = crane-lib.buildPackage { inherit src cargoArtifacts; };

      vim-helptags = derivation {
        name = "vim-helptags";
        inherit system;
        builder = pkgs.writeScript "vim-helptags-builder.sh" ''
          #!/bin/sh
          set -eu
          cp ${vim}/runtime/doc/tags $out
        '';
        PATH = "${pkgs.coreutils}/bin";
      };

      neovim-pkg = pkgs.neovim;
      neovim-helptags = derivation {
        name = "neovim-helptags";
        inherit system;
        builder = pkgs.writeScript "neovim-helptags-builder.sh" ''
          #!/bin/sh
          cp "${neovim-pkg}"/share/nvim/runtime/doc/tags $out
        '';
        PATH = "${pkgs.coreutils}/bin";
      };

      tags-env = {
        VIM_DB_PATH = vim-helptags;
        NEOVIM_DB_PATH = neovim-helptags;
        CUSTOM_DB_PATH = ./customtags;
      };
      tags-env-commands = pkgs.lib.concatStringsSep "\n" (pkgs.lib.mapAttrsToList
        (key: value:
          ''
            if [ -z "''${${key}:-}" ]; then
              export ${key}=${pkgs.lib.escapeShellArg value}
            fi
          ''
        )
        tags-env
      );
      start-script = ''
        ${tags-env-commands}
        exec ${crate}/bin/${pname}
      '';
    in
    rec {
      checks = {
        ${pname} = crate;
        "${pname}-fmt" = crane-lib.cargoFmt { inherit src; };
        "${pname}-audit" = crane-lib.cargoAudit { inherit src advisory-db; };
        "${pname}-clippy" = crane-lib.cargoClippy {
          inherit src cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
        };
      };

      packages.default = crate;
      packages.${pname} = crate;

      apps.default = flake-utils.lib.mkApp {
        drv = pkgs.writeShellScriptBin pname start-script;
      };

      nixosModules.default = with pkgs.lib; { config, ... }:
        let
          cfg = config.services.tg-vimhelpbot;
        in
        {
          options.services.tg-vimhelpbot = {
            enable = mkEnableOption "Vim :help bot for Telegram";
            envFile = mkOption {
              type = types.str;
              default = "/etc/tg-vimhelpbot.env";
            };
          };
          config = mkIf cfg.enable {
            systemd.services.tg-vimhelpbot = {
              script = start-script;
              wantedBy = [ "multi-user.target" ];
              serviceConfig.EnvironmentFile = cfg.envFile;
            };
          };
        };

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues checks;
        nativeBuildInputs = [ rust ];
      };
    }
    );
}
