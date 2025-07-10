# Contributing to Amp Game Engine

✅ **ADR-0007 COMPLETE**: Strategic migration to Bevy 0.16.1 + strategic modularity complete. Now in **AAA Restoration Phase** - see [STRATEGIC_RESTORATION_PLAN.md](docs/STRATEGIC_RESTORATION_PLAN.md).

## Development Setup

1. **Install Rust**: Use rustup to install Rust 1.85+ (Rust 2024 edition)
2. **Clone Repository**: `git clone <repository-url>`
3. **Build**: `cargo build --workspace`
4. **Test**: `cargo test --workspace`

## Code Style

### Naming Conventions
- **Variables/Functions**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Import Organization
```rust
// External crates
use anyhow::Result;
use glam::{Vec3, Mat4};

// Standard library
use std::collections::HashMap;

// Bevy 0.16.1 prelude
use bevy::prelude::*;

// Local crate
use crate::error::MyError;
```

### Error Handling
- Prefer `Result<T, E>` over `Option<T>` for error cases
- Use `anyhow` for application errors
- Use `thiserror` for library errors
- Avoid `unwrap()` and `expect()` in production code

### Documentation
- All public APIs must have documentation
- Use `//!` for module-level documentation
- Use `///` for item documentation
- Include examples in documentation when helpful

### Safety
- Validate all input parameters
- Use appropriate bounds checking
- Prefer safe abstractions over unsafe code
- Use collision groups for physics safety

## Testing

### App-Based Testing Pattern (Oracle's Standard)
For tests that need Bevy's ECS system, use the Oracle-approved App-based pattern:

```rust
use amp_engine::test_utils::test_app;

#[test]
fn test_bevy_integration() {
    let mut app = test_app();
    app.update(); // Run one frame
    
    // Access resources
    let asset_server = app.world().resource::<AssetServer>();
    
    // Access world mutably
    let mut world = app.world_mut();
    // ... make changes
}
```

**Key Patterns:**
- Use `test_app()` for standard App with MinimalPlugins
- Use `test_app_with_assets(dir)` for custom asset directories
- Replace `World::default()` with `app.world_mut()`
- Replace custom schedule runs with `app.update()` calls
- Use `App::new()` with proper plugins for integration tests

### Unit Tests
- Place tests in the same file as the code being tested
- Use `#[cfg(test)]` module structure
- Test both success and failure cases
- Aim for 70%+ code coverage
- Keep amp_core and amp_math tests Bevy-free (no App needed)

### Integration Tests
- Place integration tests in `tests/` directory
- Test public APIs and workflows
- Include performance benchmarks where appropriate
- Use App-based pattern for ECS interactions

### Property Testing
- Use `proptest` for mathematical functions
- Test invariants and edge cases
- Focus on spatial and numerical code

## Performance Guidelines

### General
- Profile before optimizing
- Measure performance impact of changes
- Use appropriate data structures
- Avoid allocations in hot paths

### Spatial Systems
- Use Morton encoding for spatial indexing
- Implement hierarchical LOD systems
- Cache distance calculations
- Use object pooling for frequently allocated objects

### GPU Code
- Minimize state changes
- Batch draw calls
- Use appropriate buffer management
- Profile GPU usage

## Commit Guidelines

### Commit Messages
```
type(scope): brief description

Longer description if needed, explaining the why behind the change.

- List specific changes
- Reference issue numbers if applicable
```

### Types
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `perf`: Performance improvements

### Scopes
- `core`: amp_core crate
- `math`: amp_math crate
- `engine`: amp_engine crate
- `config`: config_core crate
- `factory`: gameplay_factory crate
- `ci`: CI/CD changes
- `docs`: Documentation

## Pull Request Process

1. **Create Feature Branch**: `git checkout -b feature/my-feature`
2. **Implement Changes**: Follow coding standards
3. **Add Tests**: Ensure adequate test coverage
4. **Update Documentation**: Update relevant documentation
5. **Run CI Locally**: `cargo xtask ci`
6. **Create PR**: Use the provided template
7. **Address Review**: Respond to feedback promptly

## Development Tools

### Xtask Commands
- `cargo xtask ci`: Run full CI pipeline locally
- `cargo xtask fmt`: Format code
- `cargo xtask lint`: Run clippy
- `cargo xtask test`: Run tests
- `cargo xtask doc`: Generate documentation

### IDE Setup
- **VS Code**: Install rust-analyzer extension
- **CLion**: Enable Rust plugin
- **Vim/Neovim**: Use coc-rust-analyzer or native LSP

## Architecture Guidelines

### Crate Organization
- Keep dependencies minimal and justified
- Follow the dependency graph strictly
- No circular dependencies
- Use features for optional functionality

### API Design
- Keep public APIs minimal and stable
- Use builder patterns for complex construction
- Provide sensible defaults
- Document breaking changes

### Error Handling
- Use appropriate error types for each layer
- Provide helpful error messages
- Include context in error chains
- Handle errors gracefully in examples

## Performance Targets

### Compilation
- Full workspace build: < 5 minutes
- Incremental builds: < 30 seconds
- Individual crate builds: < 1 minute

### Runtime
- Target framerate: 60+ FPS
- Memory usage: Efficient allocation patterns
- Load times: < 3 seconds for basic scenes

## Release Process

1. **Version Bump**: Update version in workspace Cargo.toml
2. **Changelog**: Update CHANGELOG.md
3. **Documentation**: Ensure docs are current
4. **Testing**: Full test suite passes
5. **Tag Release**: Create git tag
6. **Publish**: Coordinate crate publishing order

## Getting Help

- **Discord**: Join the development channel
- **Issues**: Create GitHub issues for bugs/features
- **Discussions**: Use GitHub discussions for questions
- **Email**: Contact maintainers directly if needed

## Code of Conduct

Be respectful, inclusive, and professional in all interactions. We're building something amazing together.
