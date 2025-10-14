## v0.1.0

### 🚀 Enhancements

- **config:** Add configuration layer stage 1 (tasks 6-14) with defaults, file loading, env precedence ([7f4615c](https://github.com/nanodelabs/novalyn/commit/7f4615c))
- **repo:** Auto-detect repository and expose parsed provider + URLs (tasks 15-18) ([3b8caae](https://github.com/nanodelabs/novalyn/commit/3b8caae))
- **repo,ci:** Add reference formatting helpers and integrate cargo-nextest (task 19) ([f84d55b](https://github.com/nanodelabs/novalyn/commit/f84d55b))
- **git:** Implement git layer (tasks 20-26) with tests ([01dd885](https://github.com/nanodelabs/novalyn/commit/01dd885))
- **parse,version,interp:** Implement stages 4-6 (parsing, semver inference, interpolation) with tests ([3723fbc](https://github.com/nanodelabs/novalyn/commit/3723fbc))
- **parse,config,ci:** Enhanced footer parsing, scope mapping, nextest CI, default patch bump ([74aa9f9](https://github.com/nanodelabs/novalyn/commit/74aa9f9))
- Add authors aggregation, rendering, changelog pipeline, CLI flags, tagging, and update tasks.md ([dfd3c79](https://github.com/nanodelabs/novalyn/commit/dfd3c79))
- **github,logging:** Add GitHub release sync stub, tracing spans, verbosity flags, date format test ([6cff82e](https://github.com/nanodelabs/novalyn/commit/6cff82e))
- Implement missing core features: error handling, parallel processing, CLI improvements, and modern benchmarking with npm support planning ([#1](https://github.com/nanodelabs/novalyn/pull/1))
- Implement remaining tasks from tasks.md: code quality, CI/CD, and developer tooling ([#4](https://github.com/nanodelabs/novalyn/pull/4))
- Complete stages 11, 14, 16, 17, and 24 of task breakdown ([#11](https://github.com/nanodelabs/novalyn/pull/11))
- New hand rolled parser thats crazy fast ([a44c9eb](https://github.com/nanodelabs/novalyn/commit/a44c9eb))
- Async everything, comprehensive docstrings, concurrent GitHub API, and expanded test coverage ([#25](https://github.com/nanodelabs/novalyn/pull/25))
- Make everything async, concurrent and parallel ([#26](https://github.com/nanodelabs/novalyn/pull/26))
- Dont repeat yourself ([](https://github.com/nanodelabs/novalyn/commit/))

### 🩹 Fixes

- Ci ([c20fd07](https://github.com/nanodelabs/novalyn/commit/c20fd07))

### 💅 Refactors

- A lot of things ([363b909](https://github.com/nanodelabs/novalyn/commit/363b909))

### 📖 Documentation

- **tasks:** Add developer workflow (fmt, clippy, nextest) section ([4bf8270](https://github.com/nanodelabs/novalyn/commit/4bf8270))
- Implement 5 documentation stages (tasks 101-105) ([#8](https://github.com/nanodelabs/novalyn/pull/8))

### 🏡 Chore

- Stage 0 Complete ([d59b78a](https://github.com/nanodelabs/novalyn/commit/d59b78a))
- **license:** Change to my name and start year ([492e114](https://github.com/nanodelabs/novalyn/commit/492e114))
- Format ([0944a6d](https://github.com/nanodelabs/novalyn/commit/0944a6d))
- Format, lint, use aws-lc with rustls (native) ([52dea0a](https://github.com/nanodelabs/novalyn/commit/52dea0a))
- **format:** Cargo toml ([5fbc090](https://github.com/nanodelabs/novalyn/commit/5fbc090))
- Format and lint ([378918a](https://github.com/nanodelabs/novalyn/commit/378918a))
- Add some more Cargo.toml option ([a25a7e4](https://github.com/nanodelabs/novalyn/commit/a25a7e4))
- Change devcontainer image to Rust latest version ([1ea4b3a](https://github.com/nanodelabs/novalyn/commit/1ea4b3a))
- Include updated Cargo.lock and editor config ([c8a3969](https://github.com/nanodelabs/novalyn/commit/c8a3969))
- Format and lint fixes ([cee1aa9](https://github.com/nanodelabs/novalyn/commit/cee1aa9))
- Remove weird files ([0a713ba](https://github.com/nanodelabs/novalyn/commit/0a713ba))
- Format and fix tests ([f01de10](https://github.com/nanodelabs/novalyn/commit/f01de10))
- Configure cargo toml ([6042fb5](https://github.com/nanodelabs/novalyn/commit/6042fb5))
- Set up Copilot instructions for changelogen-rs repository ([#3](https://github.com/nanodelabs/novalyn/pull/3))
- Add doc tests in i ([87ddeb0](https://github.com/nanodelabs/novalyn/commit/87ddeb0))
- Reflect in tasks.md ([348313d](https://github.com/nanodelabs/novalyn/commit/348313d))
- Update tasks.md again ([0a5559f](https://github.com/nanodelabs/novalyn/commit/0a5559f))
- Remove deprecated authors field ([e9bc3bf](https://github.com/nanodelabs/novalyn/commit/e9bc3bf))
- Use 1.90.0 ([7a60da2](https://github.com/nanodelabs/novalyn/commit/7a60da2))
- Add dependabot ([405f7be](https://github.com/nanodelabs/novalyn/commit/405f7be))
- Change name ([cf99ff7](https://github.com/nanodelabs/novalyn/commit/cf99ff7))
- Add conventional commits forcing ([c65b172](https://github.com/nanodelabs/novalyn/commit/c65b172))
- Add conventional commits workflow + typos ([218f1c8](https://github.com/nanodelabs/novalyn/commit/218f1c8))
- Format and and codspeed ([76e2c5c](https://github.com/nanodelabs/novalyn/commit/76e2c5c))
- Replace make with just, add validate pr title workflow, committed config, stale workflow, coverage with llvm-cov ([00c2129](https://github.com/nanodelabs/novalyn/commit/00c2129))
- Delete CHANGELOG.md ([f3e2866](https://github.com/nanodelabs/novalyn/commit/f3e2866))
- Fix ci ([1272a1f](https://github.com/nanodelabs/novalyn/commit/1272a1f))
- Fix ci and format ([8f831de](https://github.com/nanodelabs/novalyn/commit/8f831de))
- Rename everything ([7487107](https://github.com/nanodelabs/novalyn/commit/7487107))
- Format ([fbbc362](https://github.com/nanodelabs/novalyn/commit/fbbc362))
- Use codspeed macro runners and add codeql ([#16](https://github.com/nanodelabs/novalyn/pull/16))
- Fix mimalloc and tighten security ([#17](https://github.com/nanodelabs/novalyn/pull/17))
- Better benches ([174e5a6](https://github.com/nanodelabs/novalyn/commit/174e5a6))
- Coverage token given again ([ce1ed3c](https://github.com/nanodelabs/novalyn/commit/ce1ed3c))
- Use codspeeds instrumentation mode again (finale) and perfed cargo config ([4c38d5d](https://github.com/nanodelabs/novalyn/commit/4c38d5d))
- Unpin divan version ([fa16951](https://github.com/nanodelabs/novalyn/commit/fa16951))
- Some workspace cleanup ([#20](https://github.com/nanodelabs/novalyn/pull/20))
- **ci:** Restore .github folder and workflows after accidental deletion ([12336ed](https://github.com/nanodelabs/novalyn/commit/12336ed))
- Update benches workflow to bench core only ([61075d3](https://github.com/nanodelabs/novalyn/commit/61075d3))
- Migrate to gix and others ([497c25c](https://github.com/nanodelabs/novalyn/commit/497c25c))
- Fix cargo deny ([47abd9e](https://github.com/nanodelabs/novalyn/commit/47abd9e))
- Fix benches again ([c916b21](https://github.com/nanodelabs/novalyn/commit/c916b21))
- Probable fix ([b72bbd0](https://github.com/nanodelabs/novalyn/commit/b72bbd0))
- Probable fix ([74ed32e](https://github.com/nanodelabs/novalyn/commit/74ed32e))
- Format files ([f20bc4c](https://github.com/nanodelabs/novalyn/commit/f20bc4c))
- Docstrings ([22ea008](https://github.com/nanodelabs/novalyn/commit/22ea008))
- Fix tests and doc ([b09c2ad](https://github.com/nanodelabs/novalyn/commit/b09c2ad))
- Fix doctstring and clippy ([d773d09](https://github.com/nanodelabs/novalyn/commit/d773d09))
- Add cargo-dist ([b3ec9dc](https://github.com/nanodelabs/novalyn/commit/b3ec9dc))

### ✅ Tests

- **config:** Add config tests and unknown key warnings; complete tasks 7-13 ([2eab1db](https://github.com/nanodelabs/novalyn/commit/2eab1db))
- **authors:** Address clippy field_reassign_with_default warning ([10ecac3](https://github.com/nanodelabs/novalyn/commit/10ecac3))
- **pipeline,render,github:** Deterministic ordering + pipeline exit/no-change + github fallback; adjust no-commit bump semantics ([5f97349](https://github.com/nanodelabs/novalyn/commit/5f97349))
- **cli,render,git:** Add insta snapshots, unknown subcommand, signed tag; update task statuses ([2b22862](https://github.com/nanodelabs/novalyn/commit/2b22862))

### 🤖 CI

- Add tools ([bde4a44](https://github.com/nanodelabs/novalyn/commit/bde4a44))
- Fix ([29d2b40](https://github.com/nanodelabs/novalyn/commit/29d2b40))
- Fix ([98e7e3a](https://github.com/nanodelabs/novalyn/commit/98e7e3a))
- Fix ([d40096e](https://github.com/nanodelabs/novalyn/commit/d40096e))

### ❤️ Contributors

- MuntasirSZN ([@MuntasirSZN](https://github.com/MuntasirSZN))
- Ca34d03 ([@MuntasirSZN](https://github.com/MuntasirSZN))
- Copilot ([@MicrosoftCopilot](https://github.com/MicrosoftCopilot))
- Muntasir Mahmud ([@MuntasirSZN](https://github.com/MuntasirSZN))
