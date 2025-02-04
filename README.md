[![Thalo — Event sourcing runtime for wasm][splash]](/)

[splash]: https://raw.githubusercontent.com/thalo-rs/thalo/main/splash.svg

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
![MIT OR Apache-2.0][license-badge]
[![Stargazers][stars-badge]][stars-url]
[![Last commit][commits-badge]][commits-url]
[![Discord][discord-badge]][discord-url]

[crates-badge]: https://img.shields.io/crates/v/thalo.svg
[crates-url]: https://crates.io/crates/thalo
[docs-badge]: https://docs.rs/thalo/badge.svg
[docs-url]: https://docs.rs/thalo
[license-badge]: https://img.shields.io/crates/l/thalo
[stars-badge]: https://img.shields.io/github/stars/thalo-rs/thalo.svg
[stars-url]: https://github.com/thalo-rs/thalo/stargazers
[commits-badge]: https://img.shields.io/github/last-commit/thalo-rs/thalo.svg
[commits-url]: https://github.com/thalo-rs/thalo/commits
[discord-badge]: https://img.shields.io/discord/913402468895965264?color=%23414EED&label=Discord&logo=Discord&logoColor=%23FFFFFF
[discord-url]: https://discord.gg/4Cq8NnPYPA

## Overview

Thalo is an [event sourcing] runtime for building distributed systems.
It is built on top of [Wasmtime] for components, and uses [Message DB] for the message store.

**Thalo is still in alpha, consider carefully before using for any production apps.**

Aggregates are compiled to wasm from any [supported programming language], and published to the registry,
where it is used by the runtime to handle commands.

[event sourcing]: https://microservices.io/patterns/data/event-sourcing.html
[wasmtime]: https://wasmtime.dev/
[message db]: https://github.com/message-db/message-db
[supported programming language]: #supported-languages

### Getting Started

**Crates are not published yet as they depend on some git dependencies.**
**Once [wit-bindgen] crates are published, then Thalo may be also published to crates.io**

**The Thalo crates currently published to crates.io are outdated and are an old incompatible version.**

To use Thalo now, you'll need to import it with the git url.

```toml
[dependencies]
thalo = { git = "https://github.com/thalo-rs/thalo" }
```

Current crates in this repository include:

- [thalo](crates/thalo) - Core library.
- [thalo_cli](crates/thalo_cli) - CLI utility for publishing modules and executing commands.
- [thalo_runtime](crates/thalo_runtime) - Runtime used to handle commands and persist events.
- [thalo_registry](crates/thalo_registry) - Module registry, for storing and retrieving modules.

The best way to get started for now is by looking at the [examples](/examples), or reaching out on our [Discord](discord-url) server.

[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
[examples]: /examples

## What is Event Sourcing

Event sourcing is programming pattern based on immutable events which act as the source of truth for your application.

Rather than the traditional state-oriented approach, your data consists of small events,
and your models can be built by replaying these events one by one to compute read models.

A common example of event sourcing is accounting, where your bank balance is the sum of all your transactions (aka events).
A more familiar technology using event sourcing is Git.

**What are the benefits?**

- **Scalability**

  Event sourced systems can operate in a very loosely coupled parallel style which provides excellent horizontal
  scalability and resilience to systems failure.

- **Time Travel**

  By storing immutable events, you have the ability to determine application state at any point in time.

- **Expressive Models**

  Events are first class objects in your system and show intent behind data changes. It makes the implicit explicit.

**What are the down sides?**

As with anything in the tech world, **everything** is about trade-offs.

There are some reasons not to use event sourcing in your system including:

- **Eventual consistency**:
  With the separation of concerns comes eventual consistency in your system, meaning your data may not be up to date immediately, but eventually will become consistent.

- **Hard to get Right**:
  It can be difficult to navigate this pattern when coming from the CRUD world, and there can be a lot of conflicting information online.

- **Idempotency**:
  Command and event handlers need to be written with idempotency in mind, meaning the same event handled should not be reprocessed.

- **High Disk Usage**:
  With all events being stored forever, disk usage will grow overtime. Though there are solutions to this such as [snapshotting].

**Resources**

Here are some useful resources on event sourcing & CQRS:

- https://microservices.io/patterns/data/event-sourcing.html
- https://moduscreate.com/blog/cqrs-event-sourcing/
- https://medium.com/@hugo.oliveira.rocha/what-they-dont-tell-you-about-event-sourcing-6afc23c69e9a
- https://www.youtube.com/watch?v=sb-WO-KcODE

[snapshotting]: https://domaincentric.net/blog/event-sourcing-snapshotting

## Why

The Event Sourcing & CQRS ecosystem seems to be dominated by C# and Java.
Thalo aims to expand this reach using WebAssembly, allowing components to be written in any supported language, not just Rust.

## [ESDL](https://github.com/thalo-rs/esdl) - Event-sourcing Schema Definition Language

Aggregates, commands and events are defined in the [ESDL schema language](https://github.com/thalo-rs/esdl).

This allows for more readable aggregate definitions and provides code generation to generate types and traits.

An example of an `.esdl` can be found in [`examples/bank_account/bank_account.esdl`](/examples/bank_account/bank_account.esdl).

## Supported Languages

For now, only Rust is supported, but in the future I hope to add support for other languages including AssemblyScript, Grain lang, Python.

## Examples

Examples include:

- [**bank_account**](examples/bank_account)
- [**counter**](examples/counter)
- [**counter_read_model**](examples/counter_read_model)
- [**todos**](examples/todos)

All examples can be seen in the [`examples`](examples) directory.

## Getting Help

As Thalo is in pre-release, the API is not stable yet.
If you'd like to get started using Thalo, you can checkout the [examples] directory, or chat on the [Discord server].

[examples]: https://github.com/thalo-rs/thalo/tree/main/examples
[discord server]: https://discord.gg/4Cq8NnPYPA

## Contributing

:balloon: Thanks for your help improving the project! As we don't currently have a contributing guide, you can ping us on the
Discord server or open an issue.

## License

This project is licensed under the [MIT] OR [Apache-2.0] license.

[mit]: /LICENSE-MIT
[apache-2.0]: /LICENSE-APACHE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Thalo by you, shall be licensed as MIT, without any additional
terms or conditions.
