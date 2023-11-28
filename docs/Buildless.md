# Buildless

[Buildless][0] is a suite of build caching tools and a remote build caching cloud. It can be used with nearly any build
toolchain which supports caching, including SCCache, Gradle, Maven, Bazel, and TurboRepo.

Near-caching with the [Buildless Agent][1] is free forever for every user. Then, to share caches, link your CLI to the
[Buildless Cloud][2], where it can be shared with your teammates, vendors, customers, and more.

## Using SCCache with Buildless

The Buildless adapter for SCCache supports several transports and endpoint types:

- **[Buildless Agent][1]:** Local Buildless agent and near-cache
- **[Buildless Cloud][2]:** Cloud services for caching and configuration
- **[HTTPS][3]**, **[Redis][4]**, and **[GitHub Actions][5]**

Usually, you will want to use this in tandem with the [Buildless Agent][1], which acts as a near-cache and optimized
backhaul router when cloud services are enabled.

## Installation & Setup

Obtain a [release of `sccache`][6] from the Buildless team, or clone the fork and compile it with the instructions
below.

### Obtaining a release

Releases are [published on GitHub][6], and additionally bundled with the [Buildless CLI][1] (see `buildless sccache`).
Verification of releases can be performed via [Sigstore][9] and similar tools; for more details, see the
[CLI release notes][10].

1. Download a release from one of the sources above
2. Place it somewhere on your machine, make sure it is executable
3. (**Optional**): Install the [Buildless CLI][1] and run the [Buildless Agent][8]

### Building yourself

1. Clone or add the fork: `git clone git@github.com:buildless/sccache.git`
2. Checkout your desired branch
3. Build with: `cargo build --release --features=buildless-client`
4. Check that Buildless is enabled: `./target/release/sccache --help`, which should show:

```
Usage: sccache ...

Enabled features:
    S3:        false
    Redis:     true
    Memcached: false
    GCS:       false
    GHA:       false
    Azure:     false
    Buildless: true
```

> Native zlib and TLS are built-in when using this configuration.

## Usage

If you have an API key set at `BUILDLESS_APIKEY` ([docs][7]), the module will **activate automatically.**
The [Buildless Agent][8] is detected, if running, and used (unless disabled -- see _Agent Negotation_ below). Otherwise,
**you can set `SCCACHE_BUILDLESS`** to any value to force-enable the adapter.

See other environment variables below for customizing `sccache`'s behavior as it relates to Buildless.

| Environment Variable | Description                                                                                  |
|----------------------|----------------------------------------------------------------------------------------------|
| `SCCACHE_BUILDLESS`  | Force-enables the Buildless backend, even with no API key or other credentials.              |
| `BUILDLESS_APIKEY`   | Automatically detected and set as the user's API key. Enables the module if detected.        |
| `BUILDLESS_ENDPOINT` | Sets a custom endpoint for use by `sccache`. This should only be used in advanced scenarios. |
| `BUILDLESS_NO_AGENT` | Instructs `sccache` not to ever use the [Buildless Agent][8].                                |

### Agent Negotiation

If the [Buildless Agent][8] is running on the local machine, it will be detected and used instead of the public
[Buildless Cloud service][2]. The agent can be configured to use edge services or not, and **does not require a license
or payment of any kind**. An account with Buildless Cloud is not required for local use.

Agent detection works with a "rendezvous file," defined at known path on each operating system. The file is typically
encoded in JSON and includes agent connection and protocol details.

### Buildless Cloud

If no agent is available, or if your agent is configured for Buildless Cloud services, it will automatically upload
cached objects asynchronously and pull cached objects from the cloud, via an optimized long-living connection.

### Transport Selection

Several transport types are available for use via the agent and via the Buildless Cloud, including **HTTPS**, **Redis**
(RESP), **gRPC**, and more. This adapter can use HTTPS or Redis.

Since the agent only supports HTTP at this time, it is used automatically when the agent is enabled. When using Cloud
servics directly, Redis is used, with HTTPS as a fallback. Buildless Cloud traffic is [always encrypted][11].

## Docs & Support

Find more documentation, including API reference docs, via the [Buildless Docs][12]. Support is also available for
paying users, and for free users on a best-effort basis, via the [Buildless Support][13] site.

[0]: https://less.build
[1]: https://docs.less.build/cli
[2]: https://less.build/resources/network
[3]: https://docs.less.build/reference/supported-api-interfaces
[4]: https://docs.less.build/docs/redis
[5]: https://docs.less.build/docs/github-actions
[6]: https://github.com/buildless/sccache
[7]: https://docs.less.build/docs/auth
[8]: https://docs.less.build/agent
[9]: https://sigstore.dev
[10]: https://github.com/buildless/cli/releases
[11]: https://www.ssllabs.com/ssltest/analyze.html?d=edge.less.build
[12]: https://less.build/docs
[13]: https://less.build/support
