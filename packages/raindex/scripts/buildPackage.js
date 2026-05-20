const fs = require('fs');
const { execSync } = require('child_process');

const packagePrefix = 'raindex_';

// after using opt-level on wasm build, WasmEncodedResult and WasmEncodedError
// are duplicated in the dts so we need to dedupe them
const dups = [
    "\nexport type WasmEncodedResult<T> = { value: T; error: undefined } | { value: undefined; error: WasmEncodedError };\n",
    `\nexport interface WasmEncodedError {
    msg: string;
    readableMsg: string;
}\n`
];

module.exports.buildCjs = function (pkg) {
    // generate node bindgens for cjs output
    fs.mkdirSync(`./dist/cjs`, { recursive: true });
    execSync(
        `wasm-bindgen --target nodejs ../../target/wasm32-unknown-unknown/release-wasm/${
            packagePrefix + pkg
        }.wasm --out-dir ./temp/node/${pkg} --out-name ${pkg}`
    );

    // encode wasm as base64 into a json for cjs that can be natively imported
    // in js modules in order to avoid using fetch or fs operations
    const wasmCjsBytes = fs.readFileSync(`./temp/node/${pkg}/${pkg}_bg.wasm`);
    fs.writeFileSync(
        "./dist/cjs/raindex_wbg.json",
        JSON.stringify({
            wasm: Buffer.from(wasmCjsBytes, 'binary').toString('base64')
        })
    );

    // prepare the dts
    let dts = fs.readFileSync(`./temp/node/${pkg}/${pkg}.d.ts`, {
        encoding: 'utf-8'
    });
    dts = dts.replace(
        `/* tslint:disable */
/* eslint-disable */`,
        ''
    );
    dts = '/* this file is auto-generated, do not modify */\n' + dts;
    for (const dup of dups) {
        const index = dts.indexOf(dup);
        const lastIndex = dts.lastIndexOf(dup);
        if (index === -1 || lastIndex === index) continue;
        dts = dts.replaceAll(dup, "");
        const start = dts.slice(0, index);
        const end = dts.slice(index);
        dts = start + dup + end;
    }
    fs.writeFileSync(`./dist/cjs/index.d.ts`, dts);

    // prepare cjs
    let cjs = fs.readFileSync(`./temp/node/${pkg}/${pkg}.js`, {
        encoding: 'utf-8'
    });
    cjs = cjs.replace(
        `const path = require('path').join(__dirname, '${pkg}_bg.wasm');
const bytes = require('fs').readFileSync(path);`,
        `
const { Buffer } = require('buffer');
const wasmB64 = require('./raindex_wbg.json');
const bytes = Buffer.from(wasmB64.wasm, 'base64');`
    );
    cjs = cjs.replace('const { TextEncoder, TextDecoder } = require(`util`);', '');
    cjs = '/* this file is auto-generated, do not modify */\n' + cjs;
    fs.writeFileSync(`./dist/cjs/index.js`, cjs);
}

module.exports.buildEsm = function (pkg) {
    // generate web bindgens for esm output
    fs.mkdirSync(`./dist/esm`, { recursive: true });
    execSync(
        `wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release-wasm/${
            packagePrefix + pkg
        }.wasm --out-dir ./temp/web/${pkg} --out-name ${pkg}`
    );

    // encode wasm as base64 into a json for esm that can be natively imported
    // in js modules in order to avoid using fetch or fs operations
    const wasmEsmBytes = fs.readFileSync(`./temp/web/${pkg}/${pkg}_bg.wasm`);
    fs.writeFileSync(
        `./dist/esm/raindex_wbg.json`,
        JSON.stringify({
            wasm: Buffer.from(wasmEsmBytes, 'binary').toString('base64')
        })
    );

    // prepare the dts
    let dts = fs.readFileSync(`./temp/web/${pkg}/${pkg}.d.ts`, {
        encoding: 'utf-8'
    });
    dts = dts.replace(
        `/* tslint:disable */
/* eslint-disable */`,
        ''
    );
    dts = '/* this file is auto-generated, do not modify */\n' + dts;
    for (const dup of dups) {
        const index = dts.indexOf(dup);
        const lastIndex = dts.lastIndexOf(dup);
        if (index === -1 || index === lastIndex) continue;
        dts = dts.replaceAll(dup, "");
        const start = dts.slice(0, index);
        const end = dts.slice(index);
        dts = start + dup + end;
    }
    fs.writeFileSync(`./dist/esm/index.d.ts`, dts);

    // prepare esm .js
    let esm = fs.readFileSync(`./temp/web/${pkg}/${pkg}.js`, {
        encoding: 'utf-8'
    });

    // remove unused wasm import as we import the wasm through json
    esm = esm.replaceAll(`module_or_path = new URL('${pkg}_bg.wasm', import.meta.url);`, "");

    esm = esm.replace(`export { initSync };
export default __wbg_init;`,
        `import { Buffer } from 'buffer';
import wasmB64 from './raindex_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
initSync(bytes);`
        );
    esm = '/* this file is auto-generated, do not modify */\n' + esm;
    fs.writeFileSync(`./dist/esm/index.js`, esm);
}
