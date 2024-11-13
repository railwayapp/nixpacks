install-insta:
	cargo install --list | grep -q cargo-insta || cargo install cargo-insta

# when adding or modifying ./examples tests, you'll need to run this
review-snapshots: install-insta
	cargo insta test --review --test generate_plan_tests

lint:
	cargo fmt --all -- --check
	cargo check
	cargo clippy
	cd docs && yarn lint

# run locally to fix all linting errors before pushing
lint-fix:
	cargo fmt --all
	cargo check
	cargo clippy --fix --allow-dirty
	cd docs && yarn lint

# where TEST_TARGET=test_python_asdf_poetry, helpful for rerunning failed tests on CI
test-single:
	RUST_LOG=DEBUG RUST_BACKTRACE=1 cargo test --package nixpacks --test docker_run_tests -- $(TEST_TARGET) --exact

# ex: TEST_TARGET=examples/python-postgres
build-single:
	if [ ! -d "$(TEST_TARGET)" ]; then \
		echo "Error: $(TEST_TARGET) is not a valid directory."; \
		exit 1; \
	fi

	RUST_LOG=DEBUG RUST_BACKTRACE=1 cargo run -- build $(TEST_TARGET) --name node

debug-single:
	if [ ! -d "$(TEST_TARGET)" ]; then \
		echo "Error: $(TEST_TARGET) is not a valid directory."; \
		exit 1; \
	fi

	cargo run -- build $(TEST_TARGET) --out $(TEST_TARGET)
	build_debug_cmd="$(shell sed 's/docker build/BUILDX_EXPERIMENTAL=1 docker buildx debug --invoke bash build/' $(TEST_TARGET)/.nixpacks/build.sh)" && \
		eval "$$build_debug_cmd"