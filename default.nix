{pkgs ? (import <unstable>) {}}:
with pkgs; let
  phpunwrapped = php81.unwrapped.dev.overrideAttrs (attrs: {
    configureFlags = attrs.configureFlags ++ ["--enable-zts"];
    preConfigure = ''
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
  cargo-tarpaulin = rustPlatform.buildRustPackage rec {
    pname = "cargo-tarpaulin";
    version = "0.21.0";

    src = fetchFromGitHub {
      owner = "xd009642";
      repo = "tarpaulin";
      rev = version;
      sha256 = "sha256-u6HZekrFfL+jqUh7UAo9DbgYxzS/drpt1/WdJqRFFe4=";
    };

    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs =
      [openssl]
      ++ lib.optionals stdenv.isDarwin [curl Security];

    cargoSha256 = "sha256-g3PrsyGhBiN32wPtdrIPjnQK79gaJtTfZkwv7MzYYrU=";
    #checkFlags = [ "--test-threads" "1" ];
    doCheck = false;

    meta = with lib; {
      description = "A code coverage tool for Rust projects";
      homepage = "https://github.com/xd009642/tarpaulin";
      license = with licenses; [
        mit
        /*
        or
        */
        asl20
      ];
      maintainers = with maintainers; [hugoreeves];
      platforms = lib.platforms.x86_64;
    };
  };
in
  rustPlatform.buildRustPackage rec {
    pname = "tape";
    version = "0.0.1";

    # Needed so bindgen can find libclang.so
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

    nativeBuildInputs = [rust-analyzer rustfmt clippy cargo-tarpaulin openssl openssl.dev pkg-config php phpunwrapped clang stdenv.cc.libc];
    buildInputs = [
      rustfmt
      clippy
      cargo-tarpaulin

      openssl
      php
      phpunwrapped
      clang
      stdenv.cc.libc
    ];

    src = ./.;

    cargoSha256 = "sha256-7S3UVbxj/OZ7WDRBqqGGmw6G2lSqWLGTym1DVNZLHrk=";
  }
