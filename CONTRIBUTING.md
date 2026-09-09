# Contributing to Helix-LSM

Thank you for your interest in contributing to **Helix-LSM**! We welcome bug reports, performance optimizations, architectural improvements, and documentation enhancements.

## Code of Conduct

Please maintain a constructive, respectful, and collaborative environment.

## Development Setup

Helix-LSM is built in Rust (edition 2021). Ensure you have a current stable toolchain installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### Running Tests

Run the full integration and unit test suite:

```bash
cargo test
```

### Running the Benchmark

Run the write-throughput and read-latency benchmark:

```bash
cargo run --release --bin helix-bench
```

## How to Submit Changes

1. **Fork the repository** on GitHub.
2. **Create a topic branch** from `main`:
   ```bash
   git checkout -b feat/your-improvement
   ```
3. **Commit your changes**:
   - Ensure all code is formatted with `cargo fmt`.
   - Ensure `cargo clippy` emits 0 warnings.
   - Add automated unit or integration tests covering your changes.
4. **Push to your fork** and submit a **Pull Request**.

## License

By contributing to Helix-LSM, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
