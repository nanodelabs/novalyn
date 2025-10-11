# novalyn Backlog

This document tracks potential features and enhancements that are not part of the MVP (Minimum Viable Product) but may be considered for future releases.

## Format

Each entry includes: ID, description, rationale, requested-by, complexity estimate.

---

## Backlog Items

### BL-001: Workspace Multi-Crate Support
- **Description**: Add support for monorepo/workspace structures with multiple Cargo crates
- **Rationale**: Enable changelog generation across multiple related crates in a workspace
- **Requested by**: Future user demand
- **Complexity**: High (requires significant changes to git handling and version inference)
- **Status**: Deferred

### BL-002: JSON Export Mode
- **Description**: Add CLI flag to export changelog data in JSON format instead of Markdown
- **Rationale**: Enable programmatic consumption of changelog data for automation
- **Requested by**: Future user demand
- **Complexity**: Low (rendering layer abstraction)
- **Status**: Deferred

### BL-003: Pre-release Channel Support
- **Description**: Support for pre-release versions (alpha, beta, rc) with separate changelog sections
- **Rationale**: Better support for staged releases and version management
- **Requested by**: Future user demand
- **Complexity**: Medium (semver inference changes)
- **Status**: Deferred

### BL-004: Template Customization Extension
- **Description**: Allow users to customize Markdown templates and output format
- **Rationale**: Different projects may have different changelog style preferences
- **Requested by**: Future user demand
- **Complexity**: Medium (template engine integration)
- **Status**: Deferred

### BL-005: Extended Git Hosting Provider Support
- **Description**: Add support for additional git hosting providers beyond GitHub/GitLab/Bitbucket
- **Rationale**: Support for self-hosted and enterprise git solutions
- **Requested by**: Future user demand
- **Complexity**: Low-Medium (provider detection and URL patterns)
- **Status**: Deferred

### BL-006: Hook/Plugin Architecture
- **Description**: Extensibility system for custom commit processing, validation, and formatting
- **Rationale**: Allow users to extend functionality without forking
- **Requested by**: Future user demand
- **Complexity**: High (API design and stability guarantees)
- **Status**: Deferred

### BL-007: JSON Log Format
- **Description**: Structured JSON logging for machine-readable output
- **Rationale**: Better integration with log aggregation and monitoring systems
- **Requested by**: Development workflow optimization
- **Complexity**: Low (tracing-subscriber configuration)
- **Status**: Optional enhancement

### BL-008: Version Suffix Logic
- **Description**: Support for version suffixes (pre-release identifiers, build metadata)
- **Rationale**: More flexible version management strategies
- **Requested by**: Semver edge cases
- **Complexity**: Medium (semver library extension)
- **Status**: Deferred

### BL-009: Enhanced Diff Summary
- **Description**: Detailed diff statistics in changelog write operation
- **Rationale**: Better visibility into what changed in the changelog file
- **Requested by**: Development workflow
- **Complexity**: Low (git2 integration)
- **Status**: Enhancement

### BL-010: Migration from git2 to gix
- **Description**: Replace git2 (libgit2) with pure Rust gix implementation
- **Rationale**: Better performance, native Rust implementation, easier cross-compilation
- **Requested by**: Technical debt reduction
- **Complexity**: High (major dependency change, testing required)
- **Status**: Future refactoring

---

## Adding New Backlog Items

When adding new items, use the next sequential ID (BL-XXX) and include:
1. Clear description of the feature
2. Why it would be valuable (rationale)
3. Who is requesting or would benefit from it
4. Estimated complexity (Low/Medium/High)
5. Current status

## Review Process

Backlog items should be reviewed periodically (e.g., before major releases) to assess:
- User demand and requests
- Implementation feasibility
- Impact on parity with JS version
- Maintenance burden

Items may be promoted from backlog to active development when there is sufficient justification and resources.
