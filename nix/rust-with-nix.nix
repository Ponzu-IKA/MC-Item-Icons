{ makeRustPlatform, rust-bin }: let
  toolchain = rust-bin.stable.latest.default;
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in rustPlatform.buildRustPackage {
  pname = "mc-item-icons";
  version = "0.1.0";

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
}
