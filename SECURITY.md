# Security Policy

## Supported versions

Security fixes are primarily applied to the latest released version.

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | Best effort |
| Development branches | No |

## Reporting a vulnerability

Please do **not** disclose sensitive security details in a public GitHub issue.

Use GitHub's **Report a vulnerability** feature when private vulnerability reporting is available for this repository.

If private reporting is unavailable, contact the maintainer through the contact information associated with the repository. Include the affected version, component, reproduction steps, impact and suggested mitigation. Do not include secrets or personal data.

## Security considerations

`semcode-search` can process source code and can optionally run a local REST API.

### Local code

The tool may index files under the path supplied by the user. Review the selected path before indexing repositories containing secrets or private material.

### REST API

The default server address is intended for local use:

```text
127.0.0.1
```

Do not expose the server to an untrusted network without reviewing authentication, authorization, input validation, resource limits and network configuration.

### Local AI models

AI embedding features may download or load model data locally. Review the model and dependency sources used by the selected version before deployment in sensitive environments.

## Dependency security

Contributors should keep dependencies reasonably up to date and investigate advisories affecting direct or transitive dependencies.

Before releases, run the normal tests, lint and build checks and review dependency changes.
