# Contributing to Streamora

Thank you for your interest in contributing to Streamora! We welcome contributions of all kinds—bug reports, feature suggestions, documentation improvements, and code.

## Getting Started

1. **Fork the repository** on GitHub.
2. **Clone your fork** locally.
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes** and test them thoroughly.
5. **Commit with clear messages** describing what and why.
6. **Push to your fork** and open a pull request.

## Code of Conduct

Be respectful and constructive. We are committed to providing a welcoming environment for all contributors.

## Pull Request Guidelines

- Keep PRs focused on a single feature or fix.
- Include tests for new functionality.
- Update documentation if behavior changes.
- Ensure all checks pass before requesting review.
- Link related issues in the PR description.

## Local setup

From the repository root, install the locked dependencies and create the Vite workspace configuration:

```bash
npm ci
cp .env.example apps/web/.env.local
npm run dev
```

Vite runs within `apps/web`; putting only `.env.local` at the repository root will not configure the web app. See the [Quick Start](./README.md#quick-start).

## Testing

Before submitting a PR, run these from the **repository root**:

```bash
npm test          # SDK and web Vitest suites
npm run lint      # SDK and web lint checks
npm run build     # Production web build
```

To test individual workspaces:

```bash
npm -w sdk test
npm -w apps/web test
npm -w apps/web run lint
```

For Rust contract tests (requires Rust and contract dependencies):

```bash
(cd contracts/streaming && cargo test)
```

These commands match the root scripts and CI working directories.

## Reporting Issues

Please include:
- Clear description of the issue
- Steps to reproduce
- Expected vs. actual behavior
- Environment details (OS, Node version, Stellar network, etc.)

## Questions?

Open a discussion or issue on GitHub. We're happy to help!
