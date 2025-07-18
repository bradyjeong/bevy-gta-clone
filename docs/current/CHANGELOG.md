# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0-alpha.0] - 2025-01-13

### Added
- **Sprint 9 Optimization Phase**: Complete performance optimization and polishing
- **GPU Culling Phase 3**: Real compute shader implementation with sub-0.25ms target
- **Performance Gates**: Stable 60+ FPS @1080p with comprehensive performance validation
- **Memory Optimization**: Object pools, per-frame arenas, minimal allocations
- **Large-scale Performance**: 37× improvement in entity spawning performance
- **Release Preparation**: Full v0.4.0-alpha release with Oracle's quality gates

### Changed
- **Debug Output**: Disabled noisy debug flags for production release
- **Performance Baseline**: Established comprehensive performance metrics and gates
- **Code Quality**: Comprehensive cleanup of debug artifacts and optimization
- **Documentation**: Updated all living documentation for release state

### Fixed
- **Memory Management**: Optimized allocation patterns for AAA-level performance
- **GPU Pipeline**: Complete compute shader implementation replacing simulation
- **Performance Bottlenecks**: Addressed all identified performance issues

### Performance
- **city_demo**: 60+ FPS stable @1080p (target achieved)
- **spawn_100k**: ≤3ms entity spawning (37× improvement from 111ms)
- **gpu_culling**: ≤0.25ms actual compute shader time
- **Memory**: Flat memory profile under sustained load

## [0.3.0-alpha] - 2024-12-XX

### Added
- **Sprint 8 Integration Hardening**: Professional integration with performance baseline
- **GPU Culling Phase 2**: Infrastructure with feature-flagged compute pipeline
- **Performance CI**: Automated performance testing and validation
- **AAAPlugins Architecture**: Complete plugin system rollout

### Changed
- **Architecture**: Consolidated 8-crate structure with Bevy 0.16.1 alignment
- **Testing**: 370+ tests with comprehensive coverage
- **Documentation**: Updated for production-ready state

### Fixed
- **Memory Leaks**: Complete prevention with TransientBufferPool
- **GPU Pipeline**: Professional integration with Bevy render phases
- **Performance**: Optimized CPU prepare/queue times

## [Unreleased]
