# Update Guide

## What Foro Is

`foro` is a plugin-driven code formatter daemon written in Rust. It is the end-user formatter runtime: it accepts formatting requests, loads formatter plugins, caches them, and executes them quickly across repeated runs.

In practice, `foro` itself is not "the formatter implementation" for every language. Instead, it is the host system that runs formatter plugins. Depending on the language and plugin, those plugins may be:

- native dynamic libraries
- WASM plugins
- wrappers around existing formatter engines

The `foro` source is in https://github.com/foro-fmt/foro .

## What This Repository Is

This repository, `foro-biome`, is one specific `foro` plugin. Its job is to package the `biome` formatter so that `foro` can use it as a formatter plugin for JavaScript, TypeScript, HTML, CSS, and other Biome-supported languages.

That means this repository is not only responsible for compiling against upstream `biome`, but also for:

- matching `biome`'s required Rust toolchain
- packaging the plugin in the `.dllpack` release format expected by `foro`
- producing both platform-specific native artifacts and a WASM artifact that `foro` can download and run

Unlike `foro-rustfmt`, this plugin supports both native and WASM targets because `biome` can be compiled to WASM.

## Why Updates In This Repo Are Often Multi-Layered

When upstream `biome` changes, breakage can happen in several layers:

- the formatter API or internal crate shape changes
- the Rust toolchain requirement changes
- native or WASM dependency packaging changes
- GitHub Actions runner or release workflow behavior changes
- helper tooling such as `dll-pack-builder` needs to learn a new platform-specific rule

So an update task here is usually a release-engineering task, not just a dependency bump.

## CI / Release Model In This Repo

This repo should be operated in two stages:

1. `Release Verify` workflow on a PR or manual dispatch
2. `Release` workflow only on a pushed version tag

`Release Verify` exists to test build and packaging behavior without creating a GitHub Release. `Release` is the final publish workflow and should run only after the PR is merged and a real version tag is pushed.

This document describes how an AI agent should update this repository when upstream `biome` changes.

The goal is not just "make it build locally". The goal is:

1. update to the intended upstream version
2. keep packaging/release behavior correct on every supported platform
3. finish with a real successful GitHub Actions release run

## Scope

This repository currently packages both native and WASM `biome` formatter plugins and releases these targets:

- `x86_64-unknown-linux-gnu`
- `wasm32-wasip1`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Non-Negotiable Rules

- Do not stop at a local build if the release workflow is still broken.
- Do not "fix" upstream or packaging issues by vendoring random copies into this repo unless the user explicitly asked for that.
- If the failure is in a dependency tool such as `dll-pack-builder`, fix that tool in its own repo and pin a reviewed commit here.
- Prefer the smallest correct fix that preserves release behavior.
- When the user asks for a release/update, the task is complete only after GitHub Actions finishes successfully on the actual tagged release.

## Repositories Involved

- main repo: `foro-fmt/foro-biome`
- packaging helper: `foro-fmt/dll-pack-builder`
- upstream formatter source: `biomejs/biome`

## Required Tooling

Assume these CLIs are available and use them:

- `git`
- `cargo`
- `rustup`
- `gh`
- `uv`
- `jq`
- `rg`

## High-Level Workflow

1. inspect current pinned upstream version and toolchain
2. inspect the latest upstream version/commit/toolchain
3. update local dependencies and version numbers
4. run local builds (native and WASM)
5. push a branch and run `Release Verify` on a PR
6. if CI fails, identify whether the breakage is:
   - plugin code
   - toolchain/version pin
   - packaging helper
   - runner/workflow config
7. fix the correct layer and rerun PR verification
8. merge the PR after `Release Verify` is green
9. tag the merge commit and run the real `Release` workflow
10. repeat tag-level fixes only if publish-stage behavior still fails

## Step 1: Inspect Current State

Read at least:

- `Cargo.toml`
- `Cargo.lock`
- `.github/workflows/release-verify.yml`
- `.github/workflows/release.yaml`
- `dll-pack-build-local.sh`
- `dll-pack-build-global.sh`
- `README.md`

Confirm:

- current crate version
- current upstream `biome` dependency version (tag or rev)
- current Rust toolchain
- release target matrix
- any helper tools pinned in the workflow

## Step 2: Inspect Upstream

For `biome`, check:

- upstream HEAD commit or latest release tag
- upstream crate versions

Typical commands:

```bash
git ls-remote --tags https://github.com/biomejs/biome | grep '@biomejs/biome@' | tail -10
git clone --depth=1 --branch <tag> https://github.com/biomejs/biome /tmp/biome-upstream
rg -n 'version = ' /tmp/biome-upstream/crates/biome_service/Cargo.toml
```

Do not guess the latest version from memory.

## Step 3: Update This Repo

Update only what is required:

- formatter dependency tag/rev in `Cargo.toml` (all `biome_*` crates use the same tag)
- crate version in `Cargo.toml`
- `Cargo.lock`

Versioning rule for this repo:

- use the next patch release for follow-up fixes on the same formatter line
- if the user asked for "move to 0.x.y", bump into that series first, then use further patch bumps for release-fix iterations

## Step 4: Local Validation

Run the minimum local checks before tagging:

```bash
bash -n dll-pack-build-local.sh
cargo build
cargo build --profile super-release
```

Also verify WASM builds:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --profile super-release
```

Local success is necessary but not sufficient.

## Step 5: Open A PR And Run Release Verify

Do not start with a tag push. First validate the change through the PR workflow.

Typical flow:

```bash
git switch -c update/<short-topic>
git add <files>
git commit -m "<message>"
git push origin HEAD
gh pr create --fill
```

Then inspect the verification workflow:

```bash
gh run list --workflow "Release Verify" --limit 5
gh run view <run_id> --json status,conclusion,jobs,url
gh run view <run_id> --job <job_id> --log
```

The PR is not ready to merge until:

- `build-local-artifacts` succeeds on every platform (including WASM)
- `build-global-artifacts` succeeds

If `Release Verify` fails, fix the right layer and push more commits to the same branch until it is green.

## Step 6: Merge The PR

Assume the AI agent is allowed to merge.

After `Release Verify` succeeds, merge the PR and sync local `main`.

Typical flow:

```bash
gh pr merge --merge --delete-branch
git switch main
git pull --ff-only origin main
```

Do not tag a pre-merge branch tip. Tag the merged commit on `main`.

## Step 7: Tag And Run The Real Release

Once the verified change is on `main`, create and push the real version tag.

Typical flow:

```bash
git tag <version>
git push origin <version>
```

Then inspect the publish workflow:

```bash
gh run list --workflow "Release" --limit 5
gh run view <run_id> --json status,conclusion,jobs,url
gh run view <run_id> --job <job_id> --log
gh release view <tag>
```

The task is not complete until:

- `build-local-artifacts` succeeds on every platform (including WASM)
- `build-global-artifacts` succeeds
- `host` succeeds
- the GitHub release exists for the new tag

## Failure Triage

When CI fails, classify the failure before editing anything.

### A. Upstream formatter/library changed

Symptoms:

- compile errors in this crate
- changed API shape in `biome_service`, `biome_configuration`, or related crates
- changed toolchain requirements

Action:

- update plugin code or toolchain pin
- keep changes minimal and specific to the upstream change

### B. Release workflow or runner changed

Symptoms:

- unsupported GitHub runner labels
- missing system tools in containers
- workflow syntax issues

Action:

- fix `.github/workflows/release-verify.yml` if the problem exists only in PR verification
- fix `.github/workflows/release.yaml`
- avoid changing product code for CI-only failures

### C. Packaging helper broke

Symptoms:

- `cargo build` succeeds
- failure happens inside `dll-pack-builder`
- missing DLL discovery, dependency graph, or artifact merge issues

Action:

- inspect `dll-pack-builder` logs and source
- fix `foro-fmt/dll-pack-builder` in its own repo
- add a regression test if possible
- push the helper fix
- pin the exact helper commit in both release workflows

Do not hide this by vendoring a patched copy into `foro-biome` unless explicitly requested.

### D. WASM-specific failure

Symptoms:

- native builds pass, WASM build fails
- linker errors or missing WASI symbols

Action:

- check if upstream `biome` added platform-specific code incompatible with WASM
- check if the `wasm32-wasip1` target needs a toolchain component update

### E. Platform-specific system library handling changed

Symptoms:

- only one OS fails
- typical on Windows with API set DLLs, or on macOS with shared cache/system dylibs

Action:

- determine whether the missing dependency is real or virtual/OS-provided
- fix dependency resolution in the packaging layer, not in the plugin crate

## Packaging-Specific Notes For This Repo

### WASM

- The WASM target (`wasm32-wasip1`) builds on the same Linux runner as the Linux native target.
- WASM artifacts are included in both `Release Verify` and `Release` workflows.
- If WASM fails and native passes, treat as a separate failure category.

### Windows

- The release helper uses `dll-pack-builder`.
- Windows imports like `api-ms-win-*` and `ext-ms-win-*` can be virtual OS-provided DLL names.
- If these fail only during packaging, treat it as a packaging/dependency-resolution problem first.

### macOS

- Intel and Apple Silicon are separate release targets.
- Keep both in the matrix.
- If one fails and the other passes, check runner labels and native dependency resolution separately.

### Linux

- The current workflow uses `buildpack-deps:focal`.
- If tools are missing in the container, fix the workflow/container setup rather than the plugin code.

## How To Fix A Dependency Helper Correctly

If the problem is in `dll-pack-builder`:

1. clone `https://github.com/foro-fmt/dll-pack-builder`
2. make the smallest correct change there
3. add a regression test if possible
4. run the helper's own tests locally
5. commit and push to that repo
6. pin the exact helper commit in:
   - `foro-biome/.github/workflows/release-verify.yml`
   - `foro-biome/.github/workflows/release.yaml`
7. rerun `Release Verify`
8. after merge, rerun the real tagged `Release`

Use an exact commit pin, not a floating branch reference.

Example install lines:

```yaml
run: uv tool install git+https://github.com/foro-fmt/dll-pack-builder@<commit>
run: python3 -m pip install git+https://github.com/foro-fmt/dll-pack-builder@<commit>
```

## Commit / Tag Strategy

Use small, descriptive commits.

Typical sequence:

1. formatter version bump
2. PR verification fix commit(s) if CI reveals packaging/workflow issues
3. merge after `Release Verify` is green
4. final tag for the actual release

Each failed tagged release may require a new patch version. Do not retag an existing released version.

## What To Report Back

When done, report:

- final version/tag
- successful GitHub Actions run ID and URL
- release URL
- what changed in this repo
- whether any dependency helper repo was changed and to which commit it was pinned
- any remaining warnings that did not block release
