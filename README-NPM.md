# changelogen

💅 **Beautiful Changelogs using Conventional Commits**

[![npm version](https://img.shields.io/npm/v/changelogen.svg)](https://www.npmjs.com/package/changelogen)
[![npm downloads](https://img.shields.io/npm/dm/changelogen.svg)](https://www.npmjs.com/package/changelogen)

High-performance changelog generator powered by Rust, compatible with the JavaScript ecosystem. This package provides Node.js bindings via NAPI-RS for the [changelogen-rs](https://github.com/MuntasirSZN/changelogen-rs) project.

## Features

- ✅ **Fast & Efficient** - Native Rust performance with JavaScript ease of use
- ✅ **Conventional Commits** - Full support for conventional commit parsing
- ✅ **Semantic Versioning** - Automatic version inference and bumping
- ✅ **Zero Dependencies** - Native binary with no runtime dependencies
- ✅ **Cross-Platform** - Works on Linux, macOS, and Windows
- ✅ **TypeScript Support** - Full type definitions included

## Installation

```bash
npm install changelogen
# or
yarn add changelogen
# or
pnpm add changelogen
```

## Quick Start

### Generate Changelog

```javascript
import { generate } from 'changelogen';

// Generate changelog and print to console
const result = await generate({
  from: 'v1.0.0',
  to: 'HEAD'
});

console.log(result.content);
console.log(`Processed ${result.commits} commits`);

// Generate and write to CHANGELOG.md
await generate({
  from: 'v1.0.0',
  write: true
});
```

### Full Release

```javascript
import { release } from 'changelogen';

// Perform full release (version bump + changelog + tag)
const result = await release({
  dryRun: true,  // Preview without making changes
  yes: true      // Skip confirmation prompts
});

console.log(`Release ${result.newVersion} (from ${result.previousVersion})`);
console.log(`Tag created: ${result.tagCreated}`);
```

### Show Next Version

```javascript
import { showVersion } from 'changelogen';

// Get the next semantic version
const nextVersion = await showVersion({
  from: 'v1.0.0'
});

console.log(`Next version: ${nextVersion}`);
```

## API Reference

### `generate(options?)`

Generate changelog from git history.

**Parameters:**

- `options` (optional): `GenerateOptions`
  - `cwd?: string` - Working directory (defaults to current directory)
  - `from?: string` - Starting git reference (tag, branch, or commit)
  - `to?: string` - Ending git reference (defaults to HEAD)
  - `output?: string` - Path to output file (defaults to CHANGELOG.md)
  - `write?: boolean` - Whether to write to file or return as string
  - `excludeAuthors?: string[]` - Exclude authors from changelog
  - `noAuthors?: boolean` - Don't include authors section
  - `dryRun?: boolean` - Dry run mode (no file system changes)

**Returns:** `Promise<GenerateResult>`

- `content: string` - Generated changelog markdown
- `commits: number` - Number of commits processed
- `version?: string` - Version tag used

### `release(options?)`

Perform full release: version bump, changelog generation, and git tag creation.

**Parameters:**

- `options` (optional): `ReleaseOptions`
  - `cwd?: string` - Working directory
  - `from?: string` - Starting git reference
  - `to?: string` - Ending git reference
  - `newVersion?: string` - Explicit version (overrides inference)
  - `output?: string` - Path to output file
  - `sign?: boolean` - Sign git tag
  - `excludeAuthors?: string[]` - Exclude authors
  - `noAuthors?: boolean` - Don't include authors section
  - `dryRun?: boolean` - Dry run mode
  - `yes?: boolean` - Skip confirmation prompts

**Returns:** `Promise<ReleaseResult>`

- `content: string` - Generated changelog markdown
- `commits: number` - Number of commits processed
- `previousVersion: string` - Previous version
- `newVersion: string` - New version created
- `tagCreated: boolean` - Whether a tag was created

### `showVersion(options?)`

Get the next semantic version based on commits.

**Parameters:**

- `options` (optional): `GenerateOptions`

**Returns:** `Promise<string>` - Next version string

## Configuration

Create a `.changelogrc.json` or add configuration to your `package.json`:

```json
{
  "changelogen": {
    "types": {
      "feat": {
        "title": "✨ Features",
        "semver": "minor"
      },
      "fix": {
        "title": "🐛 Bug Fixes",
        "semver": "patch"
      }
    },
    "scopeMap": {
      "ui": "frontend",
      "api": "backend"
    }
  }
}
```

For Rust projects, you can also use `changelogen.toml` or `Cargo.toml` metadata.

## Environment Variables

```bash
GITHUB_TOKEN=xxx        # GitHub API token for release sync
RUST_LOG=debug         # Enable debug logging
```

## Use in Scripts

Add to your `package.json`:

```json
{
  "scripts": {
    "changelog": "changelogen generate --write",
    "release": "changelogen release",
    "version": "changelogen show"
  }
}
```

Then run:

```bash
npm run changelog
npm run release
npm run version
```

## Comparison with JavaScript Version

This package provides native Rust performance while maintaining full compatibility with the JavaScript ecosystem:

| Feature | JS Version | Rust Version (this package) |
|---------|------------|------------------------------|
| **Speed** | ~1000 commits/sec | ~10000+ commits/sec |
| **Binary Size** | Node.js required (~50MB+) | Native binary (~5MB) |
| **Memory** | Higher (V8 overhead) | Lower (native code) |
| **API** | JavaScript only | JavaScript + Rust |
| **Distribution** | npm only | npm + Cargo |

Output format and behavior match the JavaScript version exactly.

## Requirements

- Node.js >= 16

## Performance

The native Rust implementation provides significant performance benefits:

- **10x faster** commit parsing for large repositories
- **Lower memory usage** compared to JavaScript implementation
- **Parallel processing** automatically enabled for repositories with 50+ commits

## Related Projects

- [changelogen-rs](https://github.com/MuntasirSZN/changelogen-rs) - The Rust implementation (this is a binding to it)
- [@unjs/changelogen](https://github.com/unjs/changelogen) - Original JavaScript version

## License

MIT - See [LICENSE](LICENSE) for details

## Contributing

See [CONTRIBUTING.md](https://github.com/MuntasirSZN/changelogen-rs/blob/main/CONTRIBUTING.md) for development guidelines.

## Support

- 🐛 [Report issues](https://github.com/MuntasirSZN/changelogen-rs/issues)
- 💬 [Discuss ideas](https://github.com/MuntasirSZN/changelogen-rs/discussions)
- 📖 [Documentation](https://github.com/MuntasirSZN/changelogen-rs)
