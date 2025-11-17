#!/usr/bin/env node

/**
 * Version Synchronization Script
 *
 * Reads version from package.json and updates:
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 *
 * This ensures all version numbers stay in sync across the project.
 */

const fs = require('fs');
const path = require('path');

// Paths
const packageJsonPath = path.join(__dirname, '..', 'package.json');
const cargoTomlPath = path.join(__dirname, '..', 'src-tauri', 'Cargo.toml');
const tauriConfPath = path.join(__dirname, '..', 'src-tauri', 'tauri.conf.json');

// Read version from package.json
function getPackageVersion() {
  try {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    return packageJson.version;
  } catch (error) {
    console.error('❌ Failed to read package.json:', error.message);
    process.exit(1);
  }
}

// Update Cargo.toml
function updateCargoToml(version) {
  try {
    let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');

    // Update version line in [package] section
    cargoToml = cargoToml.replace(
      /^version\s*=\s*"[^"]*"/m,
      `version = "${version}"`
    );

    fs.writeFileSync(cargoTomlPath, cargoToml, 'utf8');
    console.log(`✅ Updated Cargo.toml to version ${version}`);
  } catch (error) {
    console.error('❌ Failed to update Cargo.toml:', error.message);
    process.exit(1);
  }
}

// Update tauri.conf.json
function updateTauriConf(version) {
  try {
    const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    tauriConf.version = version;

    fs.writeFileSync(
      tauriConfPath,
      JSON.stringify(tauriConf, null, 2) + '\n',
      'utf8'
    );
    console.log(`✅ Updated tauri.conf.json to version ${version}`);
  } catch (error) {
    console.error('❌ Failed to update tauri.conf.json:', error.message);
    process.exit(1);
  }
}

// Main execution
function main() {
  console.log('🔄 Synchronizing version numbers...\n');

  const version = getPackageVersion();
  console.log(`📦 Source version (package.json): ${version}\n`);

  updateCargoToml(version);
  updateTauriConf(version);

  console.log('\n✨ Version synchronization complete!\n');
}

main();
