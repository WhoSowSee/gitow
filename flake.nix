{
  description = "Open Git repository pages in your browser";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
          gitow = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && !(builtins.elem (baseNameOf path) [
                  "artifacts"
                  "dist"
                  "result"
                  "target"
                ]);
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            nativeCheckInputs = [ pkgs.git ];
            postInstall = ''
              wrapProgram "$out/bin/gitow" \
                --prefix PATH : "${lib.makeBinPath [ pkgs.git pkgs.xdg-utils ]}"
            '';
            meta = {
              description = cargoToml.package.description;
              homepage = cargoToml.package.repository;
              license = lib.licenses.mit;
              mainProgram = "gitow";
              platforms = systems;
            };
          };
        in
        {
          default = gitow;
          inherit gitow;
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/gitow";
        };
      });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rustc
              pkgs.rustfmt
              pkgs.git
              pkgs.xdg-utils
            ];
          };
        });
    };
}
