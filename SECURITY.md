# Gorka Security Policy

This document describes what is considered a security vulnerability in Gorka and
how vulnerabilities should be reported.

## Reporting a Vulnerability

If you discover a potential security vulnerability in Gorka, please do **not**
open a public issue. Instead, contact the maintainers privately via email:

```text
zedmfix@gmail.com
```

Provide as much information as possible, including:

- Steps to reproduce the issue
- Expected vs actual behavior
- Environment (OS, Rust version, Gorka version)
- Any relevant logs or crash dumps

## Scope

This policy covers issues that could compromise:

- Data integrity
- Confidentiality
- Availability of systems using Gorka
- Malicious code execution or exploitation of vulnerabilities

## Response

- Security reports will be acknowledged within 24–48 hours.
- We will triage, investigate, and communicate any fixes or workarounds.
- Critical vulnerabilities will be prioritized for patch releases.
- Once fixed, the issue may be disclosed publicly (with credit to the reporter).

## Supported Versions

We support the latest release of Gorka and the previous minor release for
security fixes.
