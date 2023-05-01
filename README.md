# hashira

[![CI-badge]](ci) [![Latest Version]][crates.io] [![Docs][docs-badge]][docs-link]

[CI-badge]: https://github.com/Neo-Ciber94/hashira/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/Neo-Ciber94/hashira/actions/workflows/ci.yml

[latest version]: https://img.shields.io/crates/v/hashira.svg
[crates.io]: https://crates.io/crates/hashira

[docs-badge]: https://img.shields.io/badge/docs-hashira-blue.svg
[docs-link]: https://docs.rs/hashira/latest

A server side rendering framework built on top of yew.

## Getting started

To create a project with `hashira` you need to install the `CLI`.

```bash
cargo install --force hashira-cli
```

## Creating a new project

Using `hashira new` you can create a new project,
the CLI will prompt you with the template to use,
and the project name.

You can also use a shortcut to create a new project:

```bash
hashira new --{{template}} --name {{name}}
```

There are the templates available at the moment:

- actix-web
- axum
- rocket
- deno

More will be added in the future. Or if you want to create an adapter for your own, look at the code, most of the templates just use an adapter which starts the server, you can check the adapters in `/packages/adapters`.
