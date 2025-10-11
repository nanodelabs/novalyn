#!/usr/bin/env bun
/**
 * Script to copy the novalyn CLI binary to the npm package
 * This runs during prepublishOnly to include the binary with the npm distribution
 */

import { mkdir, copyFile, chmod, exists } from "node:fs/promises";
import { join, dirname } from "node:path";
import { platform, arch } from "node:os";

const BINARY_NAME = platform() === "win32" ? "novalyn.exe" : "novalyn";
const TARGET_DIR = join(import.meta.dir, "..", "cli");
const SOURCE_DIR = join(import.meta.dir, "..", "..", "..", "target", "release");

async function copyCliBinary() {
  try {
    // Ensure cli directory exists
    await mkdir(TARGET_DIR, { recursive: true });

    const sourcePath = join(SOURCE_DIR, BINARY_NAME);
    const targetPath = join(TARGET_DIR, "novalyn");

    // Check if source binary exists
    if (!(await exists(sourcePath))) {
      console.error(`❌ Binary not found: ${sourcePath}`);
      console.error("💡 Build the binary first: cd ../.. && cargo build --release");
      process.exit(1);
    }

    // Copy the binary
    await copyFile(sourcePath, targetPath);
    
    // Make it executable on Unix systems
    if (platform() !== "win32") {
      await chmod(targetPath, 0o755);
    }

    console.log(`✅ Copied CLI binary to ${targetPath}`);
  } catch (error) {
    console.error("❌ Failed to copy CLI binary:", error);
    process.exit(1);
  }
}

await copyCliBinary();
