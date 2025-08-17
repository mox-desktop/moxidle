{
  pkg-config,
  lua5_4,
  libpulseaudio,
  lib,
  rustPlatform,
  installShellFiles,
  scdoc,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "moxidle";
  inherit (cargoToml.workspace.package) version;
  cargoLock.lockFile = ../Cargo.lock;

  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      let
        relPath = lib.removePrefix (toString ../. + "/") (toString path);
      in
      lib.any (p: lib.hasPrefix p relPath) [
        "daemon"
        "ctl"
        "contrib"
        "Cargo.toml"
        "Cargo.lock"
      ];
  };

  nativeBuildInputs = [
    pkg-config
    scdoc
  ];

  buildInputs = [
    lua5_4
    libpulseaudio
    installShellFiles
  ];

  buildPhase = ''
    cargo build --release --workspace
  '';

  installPhase = ''
    install -Dm755 target/release/daemon $out/bin/moxidled
    install -Dm755 target/release/ctl $out/bin/moxidlectl
  '';

  postFixup = ''
    mkdir -p $out/share/systemd/user
    substitute $src/contrib/systemd/moxidle.service.in $out/share/systemd/user/moxidle.service --replace-fail '@bindir@' "$out/bin"
    chmod 0644 $out/share/systemd/user/moxidle.service
  '';

  meta = {
    description = "Idle daemon with conditional listeners and built-in audio inhibitor";
    mainProgram = "moxidled";
    homepage = "https://github.com/mox-desktop/moxidle";
    license = lib.licenses.mit;
    maintainers = builtins.attrValues { inherit (lib.maintainers) unixpariah; };
    platforms = lib.platforms.unix;
  };
}
