# Code Porting Template

**Based on successful Water and Persistence system ports to amp_gameplay**

This template provides a systematic approach for porting code modules within the Bevy-based GTA game project. Follow this checklist to ensure consistent, reliable ports that maintain code quality and architectural integrity.

## Port Overview

| Field | Value |
|-------|--------|
| **Source Location** | `_____________________` |
| **Target Crate** | `_____________________` |
| **Target Directory** | `_____________________` |
| **Port Type** | □ Module Migration □ Feature Addition □ System Refactor |
| **Estimated Effort** | □ Small (< 2 hours) □ Medium (2-8 hours) □ Large (> 8 hours) |
| **Dependencies** | `_____________________` |
| **Risk Level** | □ Low □ Medium □ High |

## Phase 1: Analysis & Planning

### 1.1 Reference File Identification
- [ ] **Map source files and their purposes**
  ```bash
  # Use the trace_deps.sh script to analyze dependencies
  ./scripts/trace_deps.sh <source_directory>
  ```
- [ ] **Document current dependencies**
  - External crates: `_____________________`
  - Internal modules: `_____________________`
  - Bevy systems: `_____________________`
- [ ] **Identify integration points**
  - Components used: `_____________________`
  - Systems that interact: `_____________________`
  - Events produced/consumed: `_____________________`

### 1.2 Target Architecture Mapping
- [ ] **Determine target crate structure**
  - Primary crate: `_____________________`
  - Secondary crates (if needed): `_____________________`
- [ ] **Plan module organization**
  ```
  target_crate/src/module_name/
  ├── mod.rs           # Public API exports
  ├── components.rs    # Bevy components
  ├── systems.rs       # Bevy systems
  ├── plugin.rs        # Bevy plugin integration
  ├── [serializable.rs] # Optional: for save/load
  └── tests.rs         # Unit and integration tests
  ```
- [ ] **Verify compatibility with existing architecture**
  - Check against AGENT.md guidelines
  - Ensure no circular dependencies
  - Validate against crate boundaries

### 1.3 Breaking Changes Assessment
- [ ] **API compatibility review**
  - Public interfaces that may change: `_____________________`
  - Systems that need updates: `_____________________`
  - Integration points requiring modification: `_____________________`
- [ ] **Dependency impact analysis**
  - Downstream modules affected: `_____________________`
  - Test files requiring updates: `_____________________`
  - Example code needing changes: `_____________________`

## Phase 2: Mechanical Port

### 2.1 File Structure Creation
- [ ] **Create target directory structure**
  ```bash
  mkdir -p crates/target_crate/src/module_name
  ```
- [ ] **Set up mod.rs with exports**
  - Follow the pattern from water/mod.rs or persistence/mod.rs
  - Export components, systems, and plugin

### 2.2 Component Migration
- [ ] **Port component definitions**
  - Ensure proper Bevy derives: `#[derive(Component, Default, Debug, Clone)]`
  - Maintain backward compatibility where possible
  - Add reflection support if needed: `#[reflect(Component)]`
- [ ] **Update component documentation**
  - Add comprehensive doc comments
  - Include usage examples
  - Document any behavioral changes

### 2.3 System Migration
- [ ] **Port system functions**
  - Update Bevy API usage to 0.16.1
  - Ensure proper query patterns
  - Maintain system scheduling requirements
- [ ] **Validate system dependencies**
  - Check resource requirements
  - Verify component access patterns
  - Ensure proper system ordering

### 2.4 Plugin Integration
- [ ] **Create plugin structure**
  - Follow WaterPlugin or PersistencePlugin pattern
  - Properly register systems in correct schedules
  - Add startup systems if needed
- [ ] **Add plugin configuration**
  - Create config resources if needed
  - Implement default configurations
  - Add plugin customization options

## Phase 3: Modernization Pass

### 3.1 Bevy 0.16.1 API Updates
- [ ] **Update to current Bevy patterns**
  - Use `app.world()` instead of `app.world`
  - Apply proper system parameter patterns
  - Update query syntax if needed
- [ ] **Apply performance optimizations**
  - Add system sets where beneficial
  - Optimize query patterns
  - Consider parallel system execution

### 3.2 Architecture Alignment
- [ ] **Ensure crate boundary compliance**
  - No circular dependencies
  - Proper separation of concerns
  - Clean public interfaces
- [ ] **Apply project conventions**
  - Follow AGENT.md style guidelines
  - Use consistent naming patterns
  - Apply proper error handling

### 3.3 Testing Integration
- [ ] **Port existing tests**
  - Update test setup to use new structure
  - Ensure all test cases pass
  - Add integration tests if missing
- [ ] **Add new test coverage**
  - Test new functionality
  - Validate edge cases
  - Ensure proper error handling

## Phase 4: Validation

### 4.1 Compilation Verification
- [ ] **Build verification**
  ```bash
  cargo check --workspace
  cargo build --workspace
  ```
- [ ] **Linting and formatting**
  ```bash
  cargo clippy --workspace --all-targets --all-features
  cargo fmt --all
  ```
- [ ] **Documentation generation**
  ```bash
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
  ```

### 4.2 Integration Testing
- [ ] **Run test suite**
  ```bash
  cargo test --workspace
  ```
- [ ] **Performance validation**
  ```bash
  # If applicable
  cargo bench -p target_crate
  ```
- [ ] **Memory leak testing**
  ```bash
  # If applicable for long-running systems
  cargo test --workspace --all-features -- --ignored long_memory
  ```

### 4.3 Runtime Validation
- [ ] **Example execution**
  - Test with city_demo or relevant example
  - Verify system behavior in realistic scenarios
  - Check for performance regressions
- [ ] **Integration verification**
  - Ensure proper plugin registration
  - Verify system interactions
  - Test save/load if applicable

## Phase 5: Documentation & Integration

### 5.1 Code Documentation
- [ ] **Update module documentation**
  - Comprehensive module-level docs
  - Usage examples in doc comments
  - Integration guidance
- [ ] **Update public API docs**
  - Clear component documentation
  - System behavior descriptions
  - Plugin usage instructions

### 5.2 Project Integration
- [ ] **Update Cargo.toml**
  - Add new dependencies if needed
  - Update feature flags
  - Maintain version consistency
- [ ] **Update main integration points**
  - Register plugin in appropriate locations
  - Update example code
  - Modify test harnesses if needed

### 5.3 Final Verification
- [ ] **Pre-commit validation**
  ```bash
  ./scripts/pre-commit-check.sh
  ```
- [ ] **Full CI simulation**
  ```bash
  cargo xtask ci
  ```
- [ ] **Performance gate validation**
  ```bash
  cargo xtask perf
  ```

## Success Criteria

- [ ] **All compilation passes without warnings**
- [ ] **All tests pass including new integration tests**
- [ ] **Performance benchmarks meet or exceed baseline**
- [ ] **Documentation is complete and accurate**
- [ ] **No breaking changes to existing public APIs (unless planned)**
- [ ] **Code follows project conventions and style guidelines**
- [ ] **Integration with existing systems is clean and efficient**

## Common Gotchas

### Bevy API Changes
- Watch for `app.world` vs `app.world()` method calls
- Ensure proper query parameter ordering
- Check for deprecated system registration patterns

### Dependency Issues
- Verify version consistency in workspace.dependencies
- Check for circular dependencies between crates
- Ensure proper re-exports in mod.rs files

### Testing Challenges
- Update test imports for new module locations
- Ensure test resources are properly initialized
- Check for test-only visibility requirements

### Performance Considerations
- Monitor system scheduling and ordering
- Watch for query performance in large scenes
- Ensure proper entity lifecycle management

## Notes

Additional notes and considerations specific to this port:

```
[Space for port-specific notes, decisions, and considerations]
```

---

**Template Version**: 1.0  
**Based on**: Water system (amp_gameplay/src/water) and Persistence system (amp_gameplay/src/persistence)  
**Last Updated**: Sprint 9 - AAA Optimization Phase
