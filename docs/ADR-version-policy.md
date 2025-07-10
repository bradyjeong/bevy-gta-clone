# ADR-version-policy: Oracle's Version Consistency Strategy

**Status**: Active  
**Date**: 2025-01-07  
**Oracle Consultation**: [Oracle Guard Implementation](oracle-consultations.md)

## Context

After ADR-0007's strategic migration from micro-crates to Bevy 0.16.1, we needed robust version consistency enforcement to prevent the drift issues that caused the original migration. Oracle provided detailed guidance on Cargo ecosystem best practices for multi-crate workspaces.

## Oracle's Version Consistency Rules

### Single Source of Truth

1. **[workspace.dependencies]** is the SINGLE source of truth for every 3rd-party crate used by more than one member crate
2. **workspace = true** is mandatory for all shared dependencies in member crates
3. **No version pinning** in member crates for workspace dependencies

### Engine Nucleus Lock-in

- **bevy = "=0.16.1"** (patch-locked, ecosystem nucleus)
- **bevy_rapier3d = "=0.26.0"** (patch-locked, ecosystem sidekick)
- **wgpu/winit** versions enforced via `[patch.crates-io]`

### Mature Crate Flexibility

- **serde = "^1"**, **anyhow = "^1.0"** (caret-semver for mature crates)
- **Individual crate tools** (tokio, async-trait) may vary if isolated

## Version-Bump Playbook

When upgrading Bevy versions:

1. **Branch**: Create `upgrade/bevy-X.Y.Z`
2. **Single Update**: Change only `[workspace.dependencies].bevy = "=X.Y.Z"`
3. **Precision Update**: `cargo update -p bevy --precise X.Y.Z`
4. **Patch Alignment**: Update `[patch.crates-io]` with exact wgpu/winit versions
5. **CI Verification**: Full test suite + breaking change fixes
6. **Merge**: Only after all tests pass

## Two-Layer Guard System

### Layer 1: Dependency-List Hygiene

- **Workspace Inheritance**: Check all member crates use `workspace = true`
- **No Version Pinning**: Detect `version = "..."` in member crates for workspace deps
- **Shorthand Detection**: Catch `foo = "1.0"` forms that bypass workspace

### Layer 2: Actual Resolution Sanity

- **Duplicate Detection**: `cargo tree --duplicates` for critical deps
- **Patch Lock Verification**: Engine nucleus stays exactly pinned
- **Critical Dependency List**: bevy, wgpu, winit, glam, serde, ron, pollster, thiserror, anyhow

## Legitimate Local Pinning

**Allowed**:
- Crate-specific tools (tokio in amp_engine only)
- Dev-dependencies (unless macros leak)
- Bin/example-only crates outside library graph

**Forbidden**:
- Any workspace dependency with local version override
- Different major versions of critical dependencies
- Bypassing workspace inheritance

## Implementation

- **Guard Script**: `scripts/check-version-consistency.sh` (two-layer approach)
- **Pre-commit Hook**: `.githooks/pre-commit` (automatic enforcement)
- **CI Integration**: `.github/workflows/oracle-guard.yml` (merge gate)
- **Exception File**: `scripts/version_exceptions.toml` (for rare legitimate cases)

## Benefits

1. **Prevents Version Drift**: Single source of truth eliminates conflicts
2. **Faster Builds**: No duplicate major versions in dependency tree
3. **Ecosystem Alignment**: Full Bevy 0.16.1 compatibility guaranteed
4. **Upgrade Safety**: Controlled process for version bumps
5. **CI Enforcement**: Automated protection against violations

## Maintenance

- **Monitor**: `cargo tree --duplicates` for new conflicts
- **Update**: Oracle consultation for major version changes
- **Document**: All exceptions in version_exceptions.toml with rationale
- **Review**: Version policy during architectural changes

This policy ensures the stability and consistency that Oracle identified as critical for professional AAA game development with Bevy 0.16.1.
