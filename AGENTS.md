# AGENTS.md

This document provides instructions for development agents operating in this repository.

## Build, Lint, and Test

### Build

To build the project, run:

```sh
cargo build
```

For a release build, use:

```sh
cargo build --release
```

### Run

To run the application, use:

```sh
cargo run
```

### Test

To run the tests, use:

```sh
cargo test
```

To run a single test, you can use a filter:

```sh
cargo test <test_name>
```

Currently, there are no tests in the project. Please add tests for any new functionality.

### Lint

To check the code for errors without building, run:

```sh
cargo check
```

For more in-depth linting, use Clippy:

```sh
cargo clippy
```

### Formatting

To format the code, run:

```sh
cargo fmt
```

## Code Style and Conventions

### Imports

- Use standard `use` statements at the top of the file.
- Group imports by standard library, external crates, and internal modules.

### Formatting

- Adhere to the standard Rust formatting by running `cargo fmt` before committing.

### Types

- Use explicit type annotations, especially for function signatures and public APIs.
- Avoid overly complex type signatures where possible.

### Naming Conventions

- **Variables and Functions:** `snake_case` (e.g., `my_variable`, `calculate_sum`)
- **Types (Structs, Enums, Traits):** `PascalCase` (e.g., `MyStruct`, `UserRole`)
- **Constants:** `UPPER_SNAKE_CASE` (e.g., `MAX_CONNECTIONS`)

### Error Handling

- Use the `Result` type for functions that can fail.
- For simple applications, returning `Box<dyn std::error::Error>` is acceptable.
- For libraries or more complex applications, define custom error types.
- Use the `?` operator to propagate errors.

### Comments

- Write clear and concise comments.
- Use `///` for documentation comments on public functions and types, explaining their purpose, parameters, and return values.
- Use `//` for inline comments to explain complex or non-obvious logic.

### Logging

- Use the `log` crate for logging.
- Use different log levels (`error`, `warn`, `info`, `debug`, `trace`) appropriately.

### Dependencies

- Use `cargo add` to add new dependencies.
- Keep dependencies up to date.

### Git

- Write clear and descriptive commit messages.
- Follow a conventional commit format if one is established in the project.
- Create pull requests for new features and bug fixes.
