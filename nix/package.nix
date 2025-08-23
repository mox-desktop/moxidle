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
  inherit (cargoToml.package) version;
  cargoLock.lockFile = ../Cargo.lock;

  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      let
        relPath = lib.removePrefix (toString ../. + "/") (toString path);
      in
      lib.any (p: lib.hasPrefix p relPath) [
        "src"
        "Cargo.toml"
        "Cargo.lock"
        "doc"
        "contrib"
        "completions"
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

  postInstall = ''
    for f in doc/*.scd; do
      local page="doc/$(basename "$f" .scd)"
      scdoc < "$f" > "$page"
      installManPage "$page"
    done

    installShellCompletion --cmd moxidle \
      --bash completions/moxidle.bash \
      --fish completions/moxidle.fish \
      --zsh completions/_moxidle
  '';

  meta = {
    description = "Idle daemon with conditional listeners and built-in audio inhibitor";
    mainProgram = "moxidle";
    homepage = "https://github.com/mox-desktop/moxidle";
    license = lib.licenses.mit;
    maintainers = builtins.attrValues { inherit (lib.maintainers) unixpariah; };
    platforms = lib.platforms.unix;
  };
}
