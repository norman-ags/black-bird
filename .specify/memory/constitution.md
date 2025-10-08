# Black Bird Constitution

## Core Principles

### I. TypeScript-First & Tauri Compatibility

All source code must be written in TypeScript and maintain compatibility with Tauri framework; No JavaScript files allowed in the codebase; All Tauri-specific APIs must be properly typed and used according to best practices

### II. Open Source Dependencies Only

Only open-source dependencies are permitted; All dependencies must be explicitly listed in package.json with proper licensing; No proprietary or closed-source libraries; Dependencies must be audited for security and maintenance status

### III. Modern React Architecture (NON-NEGOTIABLE)

Use functional React components exclusively - no class components; Modern hooks (useState, useEffect, useContext, etc.) are required; Component composition over inheritance; Proper separation of concerns between UI and business logic

### IV. Client-Side Data Storage

All user data must be stored client-side or in-memory only; No external databases for demo purposes; Use localStorage, sessionStorage, IndexedDB, or Tauri's filesystem APIs; Data persistence patterns must be clearly documented

### V. Spec Kit Workflow Compliance

Feature planning, implementation, and review must follow established Spec Kit workflow; All features require specification before implementation; Code reviews must verify constitutional compliance; Change management through proper channels only

## Technology Standards

### Code Quality & Formatting

Biome must be enforced for all code formatting and linting; No exceptions to Biome rules without constitutional amendment; Pre-commit hooks required to prevent non-compliant code; Configuration must be project-wide and consistent

### Documentation Requirements

All functions, components, and modules require JSDoc/TSDoc comments; Documentation must include purpose, parameters, return values, and usage examples; Complex business logic requires additional inline comments; API documentation must be auto-generated from code comments

### Type Safety & Validation

All public methods must include basic parameter and return type validation; Runtime type checking required for external data sources; TypeScript strict mode must be enabled; No `any` types without explicit justification and approval

## File Organization & Naming

### Naming Conventions

File naming must follow kebab-case (e.g., `user-profile.ts`, `api-client.ts`); Component files must use PascalCase for the component name (e.g., `UserProfile.tsx`); Directory structure must be logical and feature-based; Constants and types in UPPER_SNAKE_CASE

## Governance

Constitutional compliance is mandatory for all code changes; Biome configuration supersedes personal preferences; All dependencies must be approved through the specification process; Breaking changes require constitutional review and approval

**Version**: 1.0.0 | **Ratified**: 2025-10-08 | **Last Amended**: 2025-10-08
