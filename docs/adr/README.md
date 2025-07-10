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

- [ADR-0001: Multi-Crate Architecture](0001-multi-crate-architecture.md)
- [ADR-0002: Morton Encoding for Spatial Indexing](0002-morton-encoding-spatial.md)
- [ADR-0003: wgpu for Graphics Abstraction](0003-wgpu-graphics-abstraction.md)
- [ADR-0007: Strategic Shift to Bevy 0.16.1 Meta-Crate](0007-strategic-shift-bevy-meta-crate.md) ✅ **COMPLETED**

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
