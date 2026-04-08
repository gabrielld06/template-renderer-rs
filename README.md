# template-renderer

`template-renderer` is a small Rust CLI for generating project files from reusable schematics. A schematic is a directory of template files plus an optional `schema.json` that defines the questions to ask before rendering.

The tool renders both file contents and file names with [Handlebars](https://handlebarsjs.com/), so placeholders like `{{projectName}}` can appear inside files and in paths such as `{{file}}.js`.

## What It Does

- Lists registered schematics from a local config file
- Adds local schematics by path
- Adds remote schematics from a Git repository
- Prompts for input using a JSON schema
- Validates answers against that schema
- Renders a schematic into a destination directory

## How It Works

When you run `generate`, the CLI:

1. Loads the schematic definition from the user config file
2. Reads `schema.json` from the schematic directory, if present
3. Prompts for each property defined in the schema
4. Validates the collected answers with `jsonschema`
5. Recursively renders every file and directory name with Handlebars
6. Writes the rendered output to the destination directory

The file `schema.json` itself is not copied to the generated output.

`.git` directories are skipped during rendering.

## Requirements

- Rust toolchain
- `git` available on `PATH` if you want to generate from remote schematics

## Build And Run

```bash
cargo build
cargo run -- --help
```

For a release build:

```bash
cargo build --release
./target/release/template-renderer --help
```

## CLI Usage

Show top-level help:

```bash
template-renderer --help
```

List registered schematics:

```bash
template-renderer list
```

Add a local schematic:

```bash
template-renderer add-schematic local my-template \
  --path /absolute/path/to/schematic \
  --description "Local starter"
```

Add a remote schematic:

```bash
template-renderer add-schematic remote my-template \
  --url https://github.com/owner/repo.git \
  --branch main \
  --description "Remote starter"
```

Remove a schematic:

```bash
template-renderer remove-schematic my-template
```

Generate files from a schematic:

```bash
template-renderer generate my-template
```

Generate into a custom directory:

```bash
template-renderer generate my-template --destination ./output
```

## Configuration

The CLI stores its config in the platform-specific application config directory provided by the `directories` crate, under:

```text
com/gabrielld06/schematics/config.toml
```

On Linux this will typically resolve to something like:

```text
~/.config/com/gabrielld06/schematics/config.toml
```

If the config file does not exist yet, the application creates it automatically with defaults.

Example repository config:

```toml
log_level = "info"

[schematics]
example = { path = "./schematics/example" }
```

Schematics can be stored in three shapes in config:

- A plain string path
- A local object with `path` and optional `description`
- A remote object with `url`, optional `branch`, and optional `description`

## Writing A Schematic

A schematic is just a directory tree. The renderer walks the tree recursively and treats every file as a Handlebars template.

Example structure from this repository:

```text
schematics/example/
├── index.js
├── package.json
├── schema.json
└── {{file}}.js
```

### `schema.json`

The schema is used to collect prompt values. This project currently supports properties with these JSON Schema types:

- `string`
- `integer`
- `number`
- `boolean`

String properties can also define `enum`, which will be shown as a selection list.

Supported custom fields:

- `x-prompt`: overrides the default prompt text
- `x-casing`: generates extra derived values for string inputs

Supported casing expansions:

- `camelCase`
- `PascalCase`
- `snake_case`
- `kebab-case`
- `UPPER_CASE`

If a property named `name` uses `x-casing: ["camelCase", "snake_case"]`, the renderer will expose:

- `{{name}}`
- `{{name_camelCase}}`
- `{{name_snake_case}}`

### Template Files

Both filenames and contents are rendered through Handlebars. Given this template:

```json
{
  "name": "{{projectName}}"
}
```

and an input of `projectName = "my-app"`, the generated file becomes:

```json
{
  "name": "my-app"
}
```

If a file is named `{{file}}.js` and the answer for `file` is `utils`, the output file will be `utils.js`.

## Example

This repository includes a sample schematic in [`schematics/example`](./schematics/example).

Its schema asks for:

- `projectName`
- `helloTo`
- `file`

One of the templates contains:

```js
export function main() {
    console.log("Hello, {{helloTo}}!");
}
```

Running:

```bash
template-renderer generate example --destination ./demo
```

will prompt for those values and write rendered files under `./demo`.

## Project Structure

- [`src/main.rs`](./src/main.rs): application entry point
- [`src/core/cli.rs`](./src/core/cli.rs): Clap CLI definitions and dispatch
- [`src/core/commands.rs`](./src/core/commands.rs): list/add/remove/generate commands
- [`src/core/config.rs`](./src/core/config.rs): config loading and saving
- [`src/core/input.rs`](./src/core/input.rs): prompt and schema handling
- [`src/core/render.rs`](./src/core/render.rs): recursive template rendering
- [`src/core/schematic.rs`](./src/core/schematic.rs): schematic types and list table formatting

## Current Notes

- Remote generation clones the repository and renders from the cloned root
- The `branch` field is accepted in config and CLI, but the current implementation clones without checking out the requested branch
- The checked-in `config.toml` is a repository example; the application itself reads from the user config directory
