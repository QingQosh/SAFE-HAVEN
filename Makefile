# ============================================================
#  SAFE-HAVEN — Developer Makefile
# ============================================================

WASM_TARGET  := wasm32-unknown-unknown
WASM_OUT     := target/wasm32-unknown-unknown/release/safe_haven.wasm
OPTIMIZED    := target/safe_haven.optimized.wasm

.PHONY: all build test fmt lint clean optimize deploy-testnet size check audit deny
.PHONY: all build test watch fmt lint clean optimize deploy-testnet size check doc smoke-test-local
.PHONY: install-tools dev dev-stop backup

## Default: lint + test
all: lint test

## Compile the contract to WASM
build:
	cargo build --target $(WASM_TARGET) --release

## Run all unit tests (native, no WASM needed)
test:
	cargo test --features testutils

## Auto-run tests on file changes (requires cargo-watch)
watch:
	cargo watch -x 'test --features testutils'

## Install all recommended dev tools (cargo-watch, soroban-cli, etc.)
install-tools:
	@echo "==> Installing cargo-watch..."
	@cargo install cargo-watch 2>/dev/null || { echo "    cargo-watch install failed (may already be installed)"; }
	@echo "==> Installing soroban-cli..."
	@cargo install --locked soroban-cli 2>/dev/null || { echo "    soroban-cli install failed (may already be installed)"; }
	@echo "==> Installing cargo-audit..."
	@cargo install cargo-audit 2>/dev/null || { echo "    cargo-audit install failed (may already be installed)"; }
	@echo "==> Installing cargo-deny..."
	@cargo install cargo-deny 2>/dev/null || { echo "    cargo-deny install failed (may already be installed)"; }
	@echo ""
	@echo "All dev tools installed."

## Full local dev setup: build, deploy to local node, start frontend
DEV_ENV_FILE := frontend/.env
LOCAL_NETWORK := local

dev: build
	@echo "==> Starting local Stellar node..."
	@stellar network start $(LOCAL_NETWORK) --background 2>/dev/null || true
	@sleep 2
	@echo "==> Setting up deployer identity..."
	@stellar keys generate dev-deployer --network $(LOCAL_NETWORK) --fund 2>/dev/null || true
	@ADMIN_ADDR=$$(stellar keys address dev-deployer); \
	echo "    Admin: $$ADMIN_ADDR"; \
	echo "==> Deploying contract..."; \
	CONTRACT_ID=$$(stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/safe_haven.wasm \
		--source dev-deployer \
		--network $(LOCAL_NETWORK)); \
	echo "    Contract ID: $$CONTRACT_ID"; \
	echo "==> Initializing contract..."; \
	stellar contract invoke \
		--id "$$CONTRACT_ID" \
		--source dev-deployer \
		--network $(LOCAL_NETWORK) \
		-- initialize \
		--admin "$$ADMIN_ADDR"; \
	echo "==> Writing $(DEV_ENV_FILE)..."; \
	echo "VITE_CONTRACT_ID=$$CONTRACT_ID" > $(DEV_ENV_FILE); \
	echo 'VITE_NETWORK_PASSPHRASE=Standalone Network ; February 2017' >> $(DEV_ENV_FILE); \
	echo 'VITE_RPC_URL=http://localhost:8000' >> $(DEV_ENV_FILE); \
	echo 'VITE_HORIZON_URL=http://localhost:8000' >> $(DEV_ENV_FILE); \
	echo 'VITE_EXPLORER_URL=' >> $(DEV_ENV_FILE); \
	echo "    Written."; \
	echo ""; \
	echo "============================================"; \
	echo "  Local dev environment ready!"; \
	echo "  Contract ID: $$CONTRACT_ID"; \
	echo "  Starting frontend at http://localhost:5173"; \
	echo "============================================"
	@echo ""
	@echo "==> Starting frontend dev server..."
	@cd frontend && npm install --silent 2>/dev/null && npm run dev

dev-stop:
	@echo "==> Stopping local Stellar node..."
	@stellar network stop $(LOCAL_NETWORK) 2>/dev/null || true
	@echo "    Local node stopped."

## Format all Rust source files
fmt:
	cargo fmt --all

## Check formatting without modifying files (used in CI)
fmt-check:
	cargo fmt --all -- --check

## Run Clippy linter (fail on warnings)
lint:
	cargo clippy --all-targets --features testutils -- -D warnings

## Run fmt-check + lint + test + audit + deny in sequence (mirrors CI)
check: fmt-check lint test audit deny

## Check dependencies for known security vulnerabilities
audit:
	cargo audit

## Check dependencies for license and ban policy compliance
deny:
	cargo deny check

## Generate and open Rust API docs
doc:
	cargo doc --no-deps --open

## Remove build artifacts
clean:
	cargo clean

## Optimize WASM binary with soroban CLI
optimize: build
	soroban contract optimize --wasm $(WASM_OUT) --wasm-out $(OPTIMIZED)
	@echo "Optimized WASM: $(OPTIMIZED)"
	@ls -lh $(OPTIMIZED)

## Deploy to Stellar Testnet (requires SOROBAN_SECRET_KEY env var)
deploy-testnet:
	bash scripts/deploy_testnet.sh

## Deploy to Stellar mainnet (requires a pre-funded deployer and explicit secret key)
deploy-mainnet:
	bash scripts/deploy_mainnet.sh

## Redeploy a previous immutable contract WASM (ARTIFACT_DIR must be provided)
rollback:
	@test -n "$(ARTIFACT_DIR)" || (echo "ARTIFACT_DIR is required"; exit 1)
	bash scripts/deploy.sh rollback $(NETWORK) --artifact-dir "$(ARTIFACT_DIR)"

## Show raw WASM size
size: build
	@ls -lh $(WASM_OUT)

## Fail if optimized WASM exceeds MAX_WASM_BYTES (default 65536 = 64 KB)
MAX_WASM_BYTES ?= 65536
check-wasm-size: optimize
	@ACTUAL=$$(wc -c < $(OPTIMIZED)); \
	echo "Optimized WASM size: $${ACTUAL} bytes (limit: $(MAX_WASM_BYTES))"; \
	if [ "$${ACTUAL}" -gt "$(MAX_WASM_BYTES)" ]; then \
		echo "ERROR: WASM too large: $${ACTUAL} bytes exceeds limit of $(MAX_WASM_BYTES) bytes"; \
		exit 1; \
	fi

## Run smoke tests against a local Soroban standalone node (requires stellar CLI)
smoke-test-local: build
	bash scripts/smoke_test_local.sh

## Export current contract state and upload a backup (requires backup env vars)
backup:
	bash scripts/backup_contract.sh
