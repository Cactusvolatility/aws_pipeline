# dirs
RUST_DIR := lambda/fetcher
PY_DIR   := lambda/api
# send to aws
DIST     := dist

#RUST_TARGET=x86_64-unknown-linux-musl
RUST_BIN    := fetcher

.PHONY: help build build-rust build-python package-python init plan deploy clean

help:
	@echo "TODO"

build: $(DIST) build-rust build-python
	@echo "Done -> $(DIST)"
	@$(MAKE) -s tree

$(DIST):
	mkdir -p $(DIST)

# give AWS a binary for rust
# issue with cargo-lambda?
build-rust:
	@echo "Building Rust..."
	
	@cd $(RUST_DIR) && cargo lambda build --release --arm64 --bin $(RUST_BIN) --output-format zip
	@echo "Checking build output…"
	@ls -la target/lambda/$(RUST_BIN)/
	@mkdir -p $(DIST)/fetcher
	@cp target/lambda/$(RUST_BIN)/bootstrap.zip $(DIST)/fetcher/$(RUST_BIN).zip


## switch to docker
#	@mkdir -p $(DIST)/fetcher
#	@docker run --rm --platform linux/amd64 \
		-v $(PWD)/$(RUST_DIR):/workspace \
		-v $(PWD)/$(DIST)/fetcher:/output \
		-w /workspace \
		rust:1.70 \
		bash -c "rustup target add $(RUST_TARGET) && cargo build --release --target $(RUST_TARGET) && cp target/$(RUST_TARGET)/release/$(RUST_BIN) /output/bootstrap"
#	@$(MAKE) -s package-rust

# now that we made it - zip it
package-rust:
	@echo "Zipping Rust -> $(DIST)/fetcher/fetcher.zip"
	@cd $(DIST)/fetcher && zip -9 -q fetcher.zip bootstrap

# AWS expects a zip
build-python:
	@echo "Packaging Python..."
	@mkdir -p $(DIST)/api && rm -rf $(DIST)/api/*
	@cp $(PY_DIR)/handler.py $(DIST)/api/
# if there's requirements then install
	@if [ -f "$(PY_DIR)/requirements.txt" ]; then \
		echo "Installing Python deps..."; \
		pip install -q -r $(PY_DIR)/requirements.txt -t $(DIST)/api; \
	fi
	@$(MAKE) -s package-python

package-python:
	@echo "Zipping Python -> $(DIST)/api/api.zip"
	@cd $(DIST)/api && zip -9 -q -r api.zip .

clean:
	@rm -rf $(DIST)
	@echo "Cleaned."

tree:
	@echo "dist contents:"
	@find $(DIST) -maxdepth 2 -type f -print | sed 's|^|  |'

init:
	cd terraform && terraform init

plan: build
	cd terraform && terraform plan

deploy: build
	cd terraform && terraform apply