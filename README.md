# envow

Env schema validator and `.env.example` generator. Define your env contract once in a TOML file, validate against it and generate templates from it.

## Install

**macOS**
```sh
brew tap mb4ndeira/homebrew-tap && brew install envow
```

**Windows**
```sh
winget install mb4ndeira.envow
```

**Linux** — download `.deb` or `.rpm` from [releases](https://github.com/mb4ndeira/envow/releases).

**From source**
```sh
cargo install --git https://github.com/mb4ndeira/envow
```

## Usage

```sh
envow validate [schema]   # validate current environment against schema
envow generate [schema]   # generate .env.example from schema
```

`schema` defaults to `envow.toml` in the current directory.

| Command | Flag | Default | Effect |
|---|---|---|---|
| `validate` | `--only a,b` | all sections | validate only the listed sections |
| `generate` | `-o, --output` | `.env.example` | output file path |

Exit code `0` on success, `1` on any failure.

## Schema

```toml
[database]
DATABASE_URL = { type = "url",    required = true,  example = "postgres://user:pass@localhost:5432/db", description = "Primary database" }
DB_POOL_SIZE = { type = "int",    default = "10",   example = "10" }

[auth]
JWT_SECRET   = { type = "string", required = true,  min_length = 32, example = "a-secret-at-least-32-chars", description = "JWT signing secret" }
LOG_LEVEL    = { type = "string", default = "info", choices = ["debug", "info", "warn", "error"] }

[server]
PORT         = { type = "port",   default = "8080", example = "8080" }
```

Sections are arbitrary — name them whatever makes sense for your project. Each variable is a TOML inline table with these fields:

| Field | Description |
|---|---|
| `type` | Value type — see below. Defaults to `string` |
| `required` | Fail validation if not set and no `default` |
| `default` | Makes the var optional; used as fallback and shown in generated file |
| `example` | Shown in error output and used as placeholder in generated file |
| `description` | Shown alongside errors |
| `min_length` | Minimum character count (`string`) |
| `max_length` | Maximum character count (`string`) |
| `choices` | Restrict to a set of allowed values (`string`) |

### Types

| Type | Validates as |
|---|---|
| `string` | Any non-empty value. Supports `min_length`, `max_length`, `choices` |
| `url` | Must start with `http://` or `https://` |
| `port` | Integer 1–65535 |
| `int` | Any integer |
| `float` | Any floating-point number |
| `bool` | `true`, `false`, `1`, `0`, `yes`, `no` |
| `email` | Must contain `@` |

## Generation

`envow generate` writes a `.env.example` from your schema. Required vars are uncommented with the example as a placeholder; optional vars are commented out showing the default.

```sh
envow generate                          # → .env.example
envow generate envow.toml -o .env.template
```

Output:

```sh
# ── database ─────────────────────────────────────────────────────────────────

# Primary database [required, type:url]
DATABASE_URL=postgres://user:pass@localhost:5432/db

# Max connections [default:10]
# DB_POOL_SIZE=10

# ── auth ─────────────────────────────────────────────────────────────────────

# JWT signing secret [required, min_length:32]
JWT_SECRET=a-secret-at-least-32-chars

# Log verbosity [default:info, choices:debug|info|warn|error]
# LOG_LEVEL=info
```
