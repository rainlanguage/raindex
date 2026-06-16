const fs = require("fs");
const { sizeCheck } = require("./sizeCheck");
const { execSync } = require("child_process");
const { buildCjs, buildEsm } = require("./buildPackage");

// create root esm.js and cjs.js files with their .d.ts
fs.writeFileSync(
  "./cjs.js",
  '"use strict";\n\nmodule.exports = require("./dist/cjs/index");\n'
);
fs.writeFileSync("./cjs.d.ts", 'export * from "./dist/cjs/index";\n');
fs.writeFileSync("./esm.js", 'export * from "./dist/esm/index";\n');
fs.writeFileSync("./esm.d.ts", 'export * from "./dist/esm/index";\n');

// build specified packages and include them in final index file
// list of packages to build can be extended by adding new package
// names to the list below
const pkgs = ["js_api"];

for (const pkg of pkgs) {
  // build for cjs and esm
  buildCjs(pkg);
  buildEsm(pkg);

  // check wasm size
  sizeCheck(pkg);
}

// rm temp folder
execSync("npm run rm-temp");
