{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainlanguage/rainix";
    rain.url = "github:rainlanguage/rain.cli";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, rainix, rain }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};
        old-pkgs = rainix.old-pkgs.${system};
      in rec {
        packages = rec {

          raindex-prelude = rainix.mkTask.${system} {
            name = "raindex-prelude";
            body = ''
              set -euxo pipefail

              mkdir -p meta;
              forge script --silent ./script/BuildAuthoringMeta.sol;
              rain meta build \
                -i <(cat ./meta/OrderBookV6SubParserAuthoringMeta.rain.meta) \
                -m authoring-meta-v2 \
                -t cbor \
                -e deflate \
                -l none \
                -o meta/OrderBookV6SubParser.rain.meta \
                ;
            '';
          };

          ob-rs-test = rainix.mkTask.${system} {
            name = "ob-rs-test";
            body = ''
              set -euxo pipefail
              cargo test --workspace
            '';
          };

          ob-ui-components-prelude = rainix.mkTask.${system} {
            name = "ob-ui-components-prelude";
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

          rainix-ob-cli-artifact = rainix.mkTask.${system} {
            name = "rainix-ob-cli-artifact";
            body = ''
              set -euxo pipefail

              OUTPUT_DIR=crates/cli/bin
              ARCHIVE_NAME=rain-orderbook-cli.tar.gz
              BINARY_NAME=rain-orderbook-cli

              TARGET_TRIPLE=x86_64-unknown-linux-gnu

              cargo build --release -p rain_orderbook_cli --target "$TARGET_TRIPLE"

              mkdir -p "$OUTPUT_DIR"
              rm -f "$OUTPUT_DIR/$ARCHIVE_NAME"

              cp "target/$TARGET_TRIPLE/release/rain_orderbook_cli" "$OUTPUT_DIR/$BINARY_NAME"
              chmod 755 "$OUTPUT_DIR/$BINARY_NAME"
              strip "$OUTPUT_DIR/$BINARY_NAME" || true

              tar -C "$OUTPUT_DIR" -czf "$OUTPUT_DIR/$ARCHIVE_NAME" "$BINARY_NAME"

              rm -f "$OUTPUT_DIR/$BINARY_NAME"
            '';
          };

          rainix-wasm-artifacts = rainix.mkTask.${system} {
            name = "rainix-wasm-artifacts";
            body = ''
              set -euxo pipefail

              cargo build --profile release-wasm --target wasm32-unknown-unknown --lib -p rain_orderbook_js_api
            '';
          };

          rainix-wasm-test = rainix.mkTask.${system} {
            name = "rainix-wasm-test";
            body = ''
              set -euxo pipefail

              CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner' cargo test --target wasm32-unknown-unknown --lib -p rain_orderbook_quote -p rain_orderbook_bindings -p rain_orderbook_js_api -p rain_orderbook_common
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
              cd packages/orderbook
              npm install --no-check
            '';
          };

          build-js-bindings = rainix.mkTask.${system} {
            name = "build-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/orderbook
              npm run build
            '';
          };

          test-js-bindings = rainix.mkTask.${system} {
            name = "test-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/orderbook
              npm install --no-check
              npm run build
              npm test
            '';
          };

        } // rainix.packages.${system};

        devShells.default = pkgs.mkShell {
          packages = [
            packages.raindex-prelude
            packages.ob-rs-test
            packages.rainix-wasm-artifacts
            packages.rainix-wasm-test
            packages.rainix-wasm-browser-test
            packages.js-install
            packages.build-js-bindings
            packages.test-js-bindings
            rain.defaultPackage.${system}
            packages.ob-ui-components-prelude
            packages.rainix-ob-cli-artifact
          ];

          shellHook = rainix.devShells.${system}.default.shellHook;
          buildInputs = rainix.devShells.${system}.default.buildInputs;
          nativeBuildInputs =
            rainix.devShells.${system}.default.nativeBuildInputs;
        };
        devShells.webapp-shell = pkgs.mkShell {
          packages = with pkgs; [ nodejs_20 ];
          buildInputs = rainix.devShells.${system}.default.buildInputs;
          nativeBuildInputs =
            rainix.devShells.${system}.default.nativeBuildInputs;
        };
      });

}
