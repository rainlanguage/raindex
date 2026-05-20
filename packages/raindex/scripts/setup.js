const fs = require("fs");
const { execSync } = require("child_process");

// exit early if already built
// for npm install from an already built and packed package
if (fs.existsSync("./dist")) return;

execSync("npm run rm-temp");
execSync("npm run rm-dist");
execSync("nix develop -c node scripts/build");
