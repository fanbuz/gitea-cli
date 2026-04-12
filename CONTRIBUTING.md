# Contributing

Thanks for your interest in contributing to `gitea-cli`.

## Before you start

- Open an issue for substantial changes before writing code
- Prefer small, focused pull requests
- Keep new command surfaces read-first unless there is a strong reason otherwise

## Development workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the test suite
5. Update documentation when behavior changes
6. Open a pull request with a clear summary

## Local checks

```bash
python3 -m unittest discover -s tests -v
```

If you add a new command, include:

- command help coverage
- parameter-to-tool mapping tests
- output shape updates in the README when relevant

## Style

- Keep the CLI output stable and machine-readable
- Avoid embedding personal paths, private hosts, or real tokens in docs and examples
- Prefer small functions and explicit command-to-tool mappings

## Pull requests

Please include:

- what changed
- why it changed
- how you tested it
- any follow-up work that remains
