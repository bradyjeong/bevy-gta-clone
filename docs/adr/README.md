# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) documenting important architectural decisions made during the development of the Amp Game Engine.

## Format

Each ADR follows this template:

```markdown
# ADR-XXXX: Title

## Status
[Proposed | Accepted | Deprecated | Superseded]

## Context
What is the issue that we're seeing that is motivating this decision or change?

## Decision
What is the change that we're proposing and/or doing?

## Consequences
What becomes easier or more difficult to do because of this change?
```

## Index

- [ADR-0001: Multi-Crate Architecture](0001-multi-crate-architecture.md) - **SUPERSEDED** by ADR-0007
- [ADR-0002: Oracle-Guided Architecture](0002-oracle-guided-architecture.md) - **SUPERSEDED** by ADR-0007
- [ADR-0006: Entity Factory](0006-entity-factory.md) - **ACTIVE**
- [ADR-0007: Strategic Shift to Bevy 0.16.1 Meta-Crate](0007-strategic-shift-bevy-meta-crate.md) - ✅ **COMPLETED**
- [ADR-0008: Oracle-Guided AAA Feature Restoration Strategy](0008-oracle-guided-aaa-feature-restoration.md) - ✅ **ACCEPTED**
- [ADR-0009: GPU Culling Pipeline](0009-gpu-culling-pipeline.md) - ✅ **ACCEPTED**

**Current Architecture**: ADR-0007 defines the strategic 5-crate Bevy 0.16.1 architecture  
**Current Phase**: ADR-0008 AAA Feature Restoration - 12-week f430bc6 restoration plan  
**Current Implementation**: ADR-0009 GPU Culling Pipeline - Oracle P3a implementation

## Creating New ADRs

1. Copy the template above
2. Number sequentially (XXXX)
3. Use descriptive titles
4. Get team review before marking as "Accepted"
5. Update this index

## Guidelines

- ADRs are immutable once accepted
- If you need to change a decision, create a new ADR that supersedes the old one
- Keep ADRs focused on architectural decisions, not implementation details
- Include context about why the decision was needed
- Document both positive and negative consequences
