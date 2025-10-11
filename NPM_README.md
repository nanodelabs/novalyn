# novalyn - npm Package

> 💅 Beautiful Changelogs using Conventional Commits - Rust-powered npm package

This is the npm distribution of novalyn, providing the same powerful changelog generation capabilities as the Rust CLI but accessible from JavaScript/TypeScript projects.

## Installation

```bash
npm install @nanodelabs/novalyn
# or
yarn add @nanodelabs/novalyn
# or
pnpm add @nanodelabs/novalyn
```

## Quick Start

### TypeScript

```typescript
import { show, generate, release } from '@nanodelabs/novalyn';

// Show the next inferred version
const versionResult = await show({ 
  cwd: process.cwd() 
});
console.log(`Next version: ${versionResult.version} (${versionResult.bumpType})`);

// Generate changelog markdown
const genResult = await generate({ 
  cwd: process.cwd(),
  hideAuthorEmail: true 
});
console.log(genResult.markdown);

// Full release with custom version
const releaseResult = await release({
  cwd: process.cwd(),
  newVersion: '1.2.0'
});
console.log(`Released ${releaseResult.version} with ${releaseResult.commitCount} commits`);
```

### JavaScript

```javascript
const { show, generate, release, getCurrentVersion } = require('@nanodelabs/novalyn');

(async () => {
  // Show next version
  const result = await show();
  console.log(`Next version: ${result.version}`);

  // Get current version from Cargo.toml
  const current = getCurrentVersion();
  console.log(`Current version: ${current}`);

  // Generate changelog
  const changelog = await generate({
    from: 'v1.0.0',
    to: 'HEAD'
  });
  console.log(changelog.markdown);
})();
```

## API Reference

### `show(options?)`

Show the next inferred version based on conventional commits.

**Parameters:**
- `options` (optional): Configuration options
  - `cwd` (string): Working directory (defaults to current directory)
  - `from` (string): Starting reference (commit/tag/branch)
  - `to` (string): Ending reference (commit/tag/branch)
  - `newVersion` (string): Override the new version
  - `excludeAuthors` (string[]): Exclude authors by name or email
  - `hideAuthorEmail` (boolean): Hide author emails in output
  - `noAuthors` (boolean): Suppress all author attribution
  - `githubToken` (string): GitHub token for API access
  - `noGithubAlias` (boolean): Disable GitHub handle aliasing

**Returns:** `Promise<JsVersionResult>`
- `version` (string): The inferred version
- `bumpType` (string): Type of version bump (major, minor, patch, or none)

### `generate(options?)`

Generate a changelog block for commits since the last release.

**Parameters:** Same as `show()`

**Returns:** `Promise<JsGenerateResult>`
- `markdown` (string): The generated markdown content
- `version` (string): The version
- `commitCount` (number): Number of commits processed

### `release(options?)`

Run a full release: infer version, generate changelog, and optionally tag.

**Parameters:** Same as `show()`

**Returns:** `Promise<JsReleaseResult>`
- `version` (string): The version released
- `commitCount` (number): Number of commits processed
- `changelogUpdated` (boolean): Whether the changelog was updated
- `tagCreated` (boolean): Whether a git tag was created

### `getCurrentVersion(cwd?)`

Get the current version from Cargo.toml.

**Parameters:**
- `cwd` (optional string): Working directory (defaults to current directory)

**Returns:** `string` - The current version

## Configuration

You can configure novalyn behavior using a `novalyn.toml` file in your project root:

```toml
# Customize commit types
[types.feat]
title = "✨ Features"
semver = "minor"

[types.fix]
title = "🐛 Bug Fixes"
semver = "patch"

# Scope mapping
[scopeMap]
"ui" = "frontend"
"api" = "backend"

# GitHub token for release syncing
[tokens]
github = "${GITHUB_TOKEN}"
```

Or use `[package.metadata.novalyn]` in `Cargo.toml` for Rust projects.

## Platform Support

This package includes pre-built native binaries for:

- **Linux**: x64 (gnu/musl), arm64 (gnu/musl), arm, riscv64, s390x
- **macOS**: x64, arm64 (Apple Silicon), universal
- **Windows**: x64, ia32, arm64
- **FreeBSD**: x64
- **Android**: arm64, arm

If your platform is not listed, the installation will fail. Please file an issue on GitHub.

## Performance

Being written in Rust, novalyn offers significant performance benefits over JavaScript-based alternatives:

- **~10x faster** parsing for large commit histories
- **Lower memory usage** for processing thousands of commits
- **Parallel processing** enabled automatically for large repositories
- **Static binary** with no Node.js runtime overhead (when using CLI)

## Comparison with JavaScript Version

This npm package provides a Node.js binding to the Rust implementation, offering:

| Feature                  | JavaScript changelogen | Rust novalyn (npm) | Notes                              |
| ------------------------ | ---------------------- | ------------------ | ---------------------------------- |
| **Output**               | ✅                     | ✅                 | Identical markdown output          |
| **Configuration**        | JSON/JS                | TOML               | Different format, same options     |
| **Performance**          | Baseline               | ~10x faster        | Especially for large repos         |
| **Memory usage**         | Baseline               | ~50% lower         | Rust's efficient memory management |
| **Installation size**    | ~50MB                  | ~5MB               | Native binary vs Node.js           |
| **Parallel processing**  | ❌                     | ✅                 | Auto-enabled for 50+ commits       |
| **TypeScript types**     | ✅                     | ✅                 | Full type definitions included     |

## Contributing

See [CONTRIBUTING.md](https://github.com/nanodelabs/novalyn/blob/main/CONTRIBUTING.md) in the main repository.

## CLI Version

For command-line usage, install the Rust CLI:

```bash
cargo install novalyn
```

Or use the CLI from this npm package:

```bash
npx @nanodelabs/novalyn --help
```

## License

MIT - See [LICENSE](https://github.com/nanodelabs/novalyn/blob/main/LICENSE) for details

## Links

- [GitHub Repository](https://github.com/nanodelabs/novalyn)
- [Documentation](https://github.com/nanodelabs/novalyn#readme)
- [Issue Tracker](https://github.com/nanodelabs/novalyn/issues)
- [Changelog](https://github.com/nanodelabs/novalyn/blob/main/CHANGELOG.md)

## Acknowledgments

This is a Rust port of [@unjs/changelogen](https://github.com/unjs/changelogen) by the UnJS team.
