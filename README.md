# gh-ghes-webhook

A `gh` CLI extension for webhook forwarding from a GHES (GitHub Enterprise Server) instance to a local process.

## Background

The standard [`gh-webhook` extension](https://github.com/cli/gh-webhook) works by connecting via websocket to github.com. This websocket is not available on GHES (as of GHES version 3.12), or security policies prevent its usage.

This extension utilizes the GitHub API to poll, retrive, and forward webhook payloads to a local port. It's interface is intended to be as close to the same as `gh-webhook` as possible, but the "polling" implementation has some drawbacks (see more below).

## Installation

```
gh extension install collinmurd/gh-ghes-webhook
```

## Usage
Forward `issue` events to `stdout`
```
gh ghes-webhook forward --github-host github.host.name --events issues --repo org/repo
```

Forward `push` and `issue` webhooks to a local service
```
gh ghes-webhook forward --github-host github.host.name --events push issues --repo org/repo --url http://localhost:3000
```

### Note about polling
Organizations may be concerned about exceeding rate limits. There are built-in protections to keep API usage at bay:
- Organization level webhooks are disabled. There is an `--org` parameter built-in to match `gh webhook`, but it is not implemented.
- The extension polls for new webhook deliveries every 5 seconds, meaning there will be a delay between an event taking place and the CLI forwarding it. As of now, this is not configurable.
- The extension will stop polling if it has not seen a new event in 10 minutes. You can simply restart the process to continue. This is also not configurable as of now.

### Compatability
There are no promises about compatability with GHES versions, because there aren't any available to test against. This CLI extension does use an [undocumented API endpoint](https://github.com/orgs/community/discussions/38262#discussioncomment-6862260), which may come or go based on GitHub's release.

:)
