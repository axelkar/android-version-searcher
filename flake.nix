{
  description = "android-version-searcher devel and build";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs =
    { nixpkgs, ... }:
    let
      # System types to support.
      targetSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs targetSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          #pkgsCrossStatic = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic.buildPackages;
          #pkgsCrossAndroid = pkgs.pkgsCross.aarch64-android.buildPackages;
          # or just legacyPackages.aarch64-linux.pkgsStatic
          pkgsCrossEmuStaticT = nixpkgs.legacyPackages.aarch64-linux.pkgsStatic;
          pkgsCrossEmuStatic = nixpkgs.legacyPackages.aarch64-linux.pkgsStatic.buildPackages;
        in
        {
          default = pkgs.mkShellNoCC {
            strictDeps = true;
            # TODO: We don't even need cross rustc because aarch64 Android is a "built-in target"
            #pkgsCrossStatic_rustc = pkgsCrossStatic.rustc;
            #pkgsCrossAndroid_rustc = pkgsCrossAndroid.rustc;
            #pkgsCrossEmu_rustc = pkgsCrossEmuStatic.rustc;

            # TODO: Not working?? Uses system ld for whatever reason..
            CARGO_BUILD_TARGET = pkgsCrossEmuStaticT.stdenv.hostPlatform.rust.rustcTargetSpec;
            "CARGO_TARGET_${pkgsCrossEmuStaticT.stdenv.hostPlatform.rust.cargoEnvVarTarget}_LINKER" = "${pkgsCrossEmuStaticT.stdenv.cc}/bin/${pkgsCrossEmuStaticT.stdenv.cc.targetPrefix}cc";
            nativeBuildInputs = with pkgsCrossEmuStatic; [
              cargo
              rustc # rustfmt rust-analyzer
            ];
          };
        }
      );
    };
}
