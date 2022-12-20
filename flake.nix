{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    flake-utils,
    nixpkgs,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        (import rust-overlay)
        (self: super: {
          rustToolchain = let
            rust = super.rust-bin;
          in
            if builtins.pathExists ./rust-toolchain.toml
            then rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain
            then rust.fromRustupToolchainFile ./rust-toolchain
            else rust.stable.latest.default;
        })
      ];

      pkgs = import nixpkgs {inherit system overlays;};

      phpunwrapped = pkgs.php81.unwrapped.dev.overrideAttrs (attrs: {
        configureFlags = attrs.configureFlags ++ ["--enable-zts"];
        preConfigure =
          ''
            for i in main/build-defs.h.in scripts/php-config.in; do
                     substituteInPlace $i \
                       --replace '@CONFIGURE_COMMAND@' '(omitted)' \
                       --replace '@PHP_LDFLAGS@' ""
                   done
                   export EXTENSION_DIR=$out/lib/php/extensions
                   for i in $(find . -type f -name "*.m4"); do
                     substituteInPlace $i \
                       --replace 'test -x "$PKG_CONFIG"' 'type -P "$PKG_CONFIG" >/dev/null'
                   done
                   ./buildconf --copy --force
                   if test -f $src/genfiles; then
                     ./genfiles
                   fi
          ''
          + pkgs.lib.optionalString (system == flake-utils.lib.system.aarch64-darwin) ''
            substituteInPlace configure --replace "-lstdc++" "-lc++"
          '';
      });

      php = phpunwrapped.buildEnv {
        extensions = {
          enabled,
          all,
        }:
          enabled
          ++ (with all; [
            redis
            pcov
            dom
            mbstring
            tokenizer
            xmlwriter
            xmlreader
          ]);
        extraConfig = "memory_limit = -1";
      };
    in {
      devShells.default = pkgs.mkShell {
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

        nativeBuildInputs = with pkgs; [
          php
          php81Packages.composer
          phpunwrapped
          stdenv.cc.libc
          clang
          rustToolchain
          openssl
          pkg-config
          cargo-deny
          cargo-edit
          cargo-watch
          rust-analyzer
        ];

        buildInputs = with pkgs; [
          php
          phpunwrapped
          stdenv.cc.libc
          clang
        ];

        shellHook = ''
          ${pkgs.rustToolchain}/bin/cargo --version
        '';
      };
    });
}
