{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainlanguage/rainix";
    rain.url = "github:rainlanguage/rain.cli";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      flake-utils,
      rainix,
      rain,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = rainix.pkgs.${system};
      in
      rec {
        packages = rec {

          raindex-prelude = rainix.mkTask.${system} {
            name = "raindex-prelude";
            body = ''
              set -euxo pipefail

              mkdir -p meta;
              forge script --silent ./script/BuildAuthoringMeta.sol;
              rain meta build \
                -i <(cat ./meta/RaindexV6SubParserAuthoringMeta.rain.meta) \
                -m authoring-meta-v2 \
                -t cbor \
                -e deflate \
                -l none \
                -o meta/RaindexV6SubParser.rain.meta \
                ;
            '';
          };

          raindex-rs-test = rainix.mkTask.${system} {
            name = "raindex-rs-test";
            body = ''
              set -euxo pipefail
              cargo test --workspace
            '';
          };

          raindex-ui-components-prelude = rainix.mkTask.${system} {
            name = "raindex-ui-components-prelude";
            body = ''
              set -euxo pipefail

              # Fix linting of generated types
              cd packages/ui-components && npm i && npm run lint
            '';
            additionalBuildInputs = [
              pkgs.wasm-bindgen-cli
              rainix.rust-toolchain.${system}
              rainix.rust-build-inputs.${system}
            ];
          };

          raindex-cli-artifact = rainix.mkTask.${system} {
            name = "raindex-cli-artifact";
            body = ''
              set -euxo pipefail

              OUTPUT_DIR=crates/cli/bin
              ARCHIVE_NAME=raindex-cli.tar.gz
              BINARY_NAME=raindex-cli

              TARGET_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"

              case "$TARGET_TRIPLE" in
                aarch64-apple-darwin|x86_64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
                  ;;
                *)
                  echo "Unsupported host target: $TARGET_TRIPLE" >&2
                  exit 1
                  ;;
              esac

              cargo build --release -p raindex_cli --target "$TARGET_TRIPLE"

              mkdir -p "$OUTPUT_DIR"
              rm -f "$OUTPUT_DIR/$ARCHIVE_NAME"

              cp "target/$TARGET_TRIPLE/release/raindex_cli" "$OUTPUT_DIR/$BINARY_NAME"
              chmod 755 "$OUTPUT_DIR/$BINARY_NAME"
              strip "$OUTPUT_DIR/$BINARY_NAME" || true

              tar -C "$OUTPUT_DIR" -czf "$OUTPUT_DIR/$ARCHIVE_NAME" "$BINARY_NAME"
            '';
          };

          rainix-wasm-artifacts = rainix.mkTask.${system} {
            name = "rainix-wasm-artifacts";
            body = ''
              set -euxo pipefail

              cargo build --profile release-wasm --target wasm32-unknown-unknown --lib -p raindex_js_api
            '';
          };

          rainix-wasm-test = rainix.mkTask.${system} {
            name = "rainix-wasm-test";
            body = ''
              set -euxo pipefail

              CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner' cargo test --target wasm32-unknown-unknown --lib -p raindex_quote -p raindex_bindings -p raindex_js_api -p raindex_common
            '';
          };

          rainix-wasm-browser-test = rainix.mkTask.${system} {
            name = "rainix-wasm-browser-test";
            body = ''
              set -euxo pipefail

              cd crates/common
              wasm-pack test --headless --chrome --features browser-tests -- leadership::wasm_tests
              wasm-pack test --headless --chrome --features browser-tests -- scheduler::wasm::wasm_tests
              wasm-pack test --headless --chrome --features browser-tests -- status::wasm::wasm_tests
              wasm-pack test --headless --chrome --features browser-tests -- retry::wasm_tests
              wasm-pack test --headless --chrome --features browser-tests -- raindex_client::local_db::wasm_tests
            '';
            additionalBuildInputs = [
              pkgs.wasm-pack
            ];
          };

          js-install = rainix.mkTask.${system} {
            name = "js-install";
            body = ''
              set -euxo pipefail
              cd packages/raindex
              npm install --no-check
            '';
          };

          build-js-bindings = rainix.mkTask.${system} {
            name = "build-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/raindex
              npm run build
            '';
          };

          test-js-bindings = rainix.mkTask.${system} {
            name = "test-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/raindex
              npm install --no-check
              npm run build
              npm test
            '';
          };

        }
        // rainix.packages.${system};

        devShells.default = pkgs.mkShell {
          packages = [
            packages.raindex-prelude
            packages.raindex-rs-test
            packages.rainix-wasm-artifacts
            packages.rainix-wasm-test
            packages.rainix-wasm-browser-test
            packages.js-install
            packages.build-js-bindings
            packages.test-js-bindings
            rain.defaultPackage.${system}
            packages.raindex-ui-components-prelude
            packages.raindex-cli-artifact
          ];

          inherit (rainix.devShells.${system}.default) shellHook buildInputs nativeBuildInputs;
        };
        devShells.webapp-shell = pkgs.mkShell {
          packages = with pkgs; [ nodejs_20 ];
          inherit (rainix.devShells.${system}.default) buildInputs nativeBuildInputs;
        };
      }
    );

}
