const os = require("os");
const fs = require("fs");
const path = require("path");

const PLATFORMS = {
  "darwin-arm64": "@aitop/darwin-arm64",
  "darwin-x64": "@aitop/darwin-x64",
  "linux-x64": "@aitop/linux-x64",
  "linux-arm64": "@aitop/linux-arm64",
};

const key = `${os.platform()}-${os.arch()}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  console.warn(`aitop: unsupported platform ${key}, binary will not be available`);
  process.exit(0);
}

try {
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  const binary = path.join(pkgDir, "bin", "aitop");
  if (!fs.existsSync(binary)) {
    console.warn(`aitop: binary not found at ${binary}`);
    console.warn("You may need to reinstall or use a different install method.");
    process.exit(0);
  }
} catch {
  console.warn(`aitop: optional dependency ${pkg} not installed for ${key}`);
  console.warn("You may need to reinstall or use a different install method.");
  process.exit(0);
}
