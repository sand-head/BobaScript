<p align="center">
  <img src="./logo.svg" height="128" />
  <h1 align="center">BobaScript</h1>
</p>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/bobascript?style=flat-square)](https://crates.io/crates/bobascript)
  [![CI](https://img.shields.io/github/workflow/status/sand-head/BobaScript/CI?event=push&style=flat-square)](https://github.com/sand-head/BobaScript/actions/workflows/ci.yml)
  [![Open in VS Code](https://img.shields.io/badge/open-in_Visual_Studio_Code-blue?logo=visualstudiocode&style=flat-square)](https://open.vscode.dev/sand-head/BobaScript)
  [![Matrix](https://img.shields.io/matrix/bobascript:schweigert.dev?server_fqdn=matrix.schweigert.dev&style=flat-square)](https://matrix.to/#/#bobascript:schweigert.dev)

</div>

a nice, cold, refreshing scripting language built in Rust, with some fun little gimmicks!

## tell me more

ok here's the jist of my plans:

- **inspired by TypeScript and Rust!** has very Rust-like syntax and is expression-based, but has modern JavaScript-like elements such as tuples and records. a type system like TypeScript's is planned!
- **built with several use-cases in mind!** from embedding it in your own app to building web apps in it, BobaScript has you covered! build server side rendered apps PHP-style using BSX, or compile it to either WASM or modern JavaScript!
- **we don't know the concept of null values!** I've never even heard of them, personally! in this language anything that doesn't return a value just returns an empty tuple by default! I also plan on introducing `Option` types when the type system gets thrown in.

these are all subject-to-additions as I work on this, but never removals or changes! I, the person who hasn't worked in lang dev before, will be stubbornly sticking to my guns on at least those three points

## this is ridiculous and pointless

that's just, like, your opinion, man!
