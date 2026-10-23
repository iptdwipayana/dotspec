# DotSpec

> [!NOTE]
> **Status: Actively Maintained**. This project is under active development,
> and the API may change before `1.0.0`.

Config parser for `.spec` files, written in Rust as a learning exercise.

### Example

```txt
app-name: "dotspec-demo"
app-version: "0.1.0"

[server]
host: "0.0.0.0"
port: 8080
url: "http://#{server.host}:#{server.port}"
```

### Usage

```rs
let config = Config::parse(text)?;
config.get("server.host");
config.get("server.port");
```

> This API is still on plan and does not exists yet.

### Workspace

| Crate             | Purpose                                      |
| ----------------- | -------------------------------------------- |
| `dotspec`         | Entry point that re-exports everything       |
| `dotspec-core`    | Parser, checker, resolver and runtime values |
| `dotspec-codegen` | Code generation logic                        |
| `dotspec-macro`   | `#[derive(DotSpec)]` macro                   |

### License

MIT
