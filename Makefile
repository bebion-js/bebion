# Bebion JavaScript Runtime Makefile

.PHONY: all build release test clean install uninstall docs bench fmt clippy

# Default target
all: build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run tests
test:
	cargo test --all

# Run tests with output
test-verbose:
	cargo test --all -- --nocapture

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/
	rm -rf docs/book/

# Install bebion binary
install: release
	cargo install --path .

# Uninstall bebion binary
uninstall:
	cargo uninstall bebion

# Format code
fmt:
	cargo fmt --all

# Run clippy linter
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Check code without building
check:
	cargo check --all

# Run benchmarks
bench:
	cargo bench

# Generate documentation
docs:
	cargo doc --all --no-deps --open

# Build and run examples
examples:
	@echo "Building examples..."
	@mkdir -p examples
	@echo 'console.log("Hello, Bebion!");' > examples/hello.js
	@echo 'async function test() { console.log("Async works!"); } test();' > examples/async.js
	@echo 'class Test { constructor(name) { this.name = name; } greet() { return `Hello, ${this.name}!`; } } const t = new Test("World"); console.log(t.greet());' > examples/class.js
	cargo run --release -- examples/hello.js
	cargo run --release -- examples/async.js
	cargo run --release -- examples/class.js

# Development setup
dev-setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch
	cargo install mdbook

# Watch for changes and rebuild
watch:
	cargo watch -x build

# Watch and run tests
watch-test:
	cargo watch -x test

# Profile release build
profile: release
	@echo "Profiling bebion..."
	@echo 'for(let i = 0; i < 1000000; i++) { Math.sqrt(i); }' > /tmp/profile_test.js
	time ./target/release/bebion /tmp/profile_test.js
	rm /tmp/profile_test.js

# Memory usage test
memory-test: release
	@echo "Testing memory usage..."
	@echo 'let arr = []; for(let i = 0; i < 100000; i++) { arr.push({id: i, data: "test".repeat(10)}); } console.log("Created", arr.length, "objects");' > /tmp/memory_test.js
	/usr/bin/time -v ./target/release/bebion /tmp/memory_test.js 2>&1 | grep -E "(Maximum resident set size|User time|System time)"
	rm /tmp/memory_test.js

# Create release package
package: release
	@echo "Creating release package..."
	mkdir -p bebion-release
	cp target/release/bebion bebion-release/
	cp README.md bebion-release/
	cp LICENSE bebion-release/
	mkdir -p bebion-release/examples
	cp examples/*.js bebion-release/examples/ 2>/dev/null || true
	tar -czf bebion-$(shell cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "bebion") | .version').tar.gz bebion-release/
	rm -rf bebion-release/

# Docker build
docker-build:
	docker build -t bebion:latest .

# Docker run
docker-run: docker-build
	docker run -it --rm bebion:latest

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update

# Show project statistics
stats:
	@echo "=== Bebion Project Statistics ==="
	@echo "Lines of Rust code:"
	@find src crates -name "*.rs" -exec wc -l {} + | tail -1
	@echo "Number of crates:"
	@ls crates/ | wc -l
	@echo "Total files:"
	@find . -name "*.rs" -o -name "*.toml" -o -name "*.md" | wc -l
	@echo "Dependencies:"
	@cargo tree --depth 1 | grep -v "bebion" | wc -l

# Help target
help:
	@echo "Available targets:"
	@echo "  build        - Build debug version"
	@echo "  release      - Build release version"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  install      - Install bebion binary"
	@echo "  uninstall    - Uninstall bebion binary"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run linter"
	@echo "  check        - Check code without building"
	@echo "  bench        - Run benchmarks"
	@echo "  docs         - Generate documentation"
	@echo "  examples     - Build and run examples"
	@echo "  dev-setup    - Setup development environment"
	@echo "  watch        - Watch for changes and rebuild"
	@echo "  watch-test   - Watch and run tests"
	@echo "  profile      - Profile release build"
	@echo "  memory-test  - Test memory usage"
	@echo "  package      - Create release package"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run in Docker container"
	@echo "  audit        - Security audit"
	@echo "  update       - Update dependencies"
	@echo "  stats        - Show project statistics"
	@echo "  help         - Show this help"