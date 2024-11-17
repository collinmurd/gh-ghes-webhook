# gh-ghes-webhook

A `gh` CLI extension for webhook forwarding from a GHES (GitHub Enterprise Server) instance to a local process.

## Background

The standard [`gh-webhook` extension](https://github.com/cli/gh-webhook) works by connecting via websocket to github.com. This websocket is not available on GHES (as of GHES version 3.12), or security policies prevent its usage.

This extension utilizes the GitHub API to poll, retrive, and forward webhook payloads to a local port. It's interface is intended to be as close to the same as `gh-webhook` as possible, but the "polling" implementation has some drawbacks:
- There will be a delay between an event taking place on GitHub and it being forwarded.
- There will be some rate-limit concerns, and some protections will be built in to hopefully minimize those.
