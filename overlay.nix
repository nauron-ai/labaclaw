final: prev: {
  zeroclaw-web = final.callPackage ./web/package.nix { };

  labaclaw = final.callPackage ./package.nix {
    rustToolchain = final.fenix.stable.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
    ];
  };

  zeroclaw = final.labaclaw;
}
