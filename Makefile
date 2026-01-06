# dirs
RUST_DIR := lambda/fetcher
# send to aws
DIST     := dist

#AWS creds
AWS_REGION ?= us-west-2
AWS_ACCOUNT_ID ?= $(shell aws sts get-caller-identity --query Account --output text)
PY_5MIN_DIR := lambda/py-five-interval
ECR_REPO_5MIN := process-5min-lambda
TAG ?= latest

ECR_URI_5MIN := $(AWS_ACCOUNT_ID).dkr.ecr.$(AWS_REGION).amazonaws.com/$(ECR_REPO_5MIN):$(TAG)

#RUST_TARGET=x86_64-unknown-linux-musl
RUST_BINS    := tiingo_iex fmp_news

.PHONY: build build-rust push-py-5min ecr-login build-py-5min init plan deploy clean tree

build: $(DIST) build-rust
	@echo "Done -> $(DIST)"
	@$(MAKE) -s tree

$(DIST):
	mkdir -p $(DIST)

# give AWS a binary for rust
# issue with cargo-lambda?
# my external cargo.toml is building target at root - fine
# TODO:
#	 port back to directory?
#	 change in architecture - no longer additional modules
build-rust:
	@echo "Building Rust..."
	
	@for bin in $(RUST_BINS); do \
		echo "Building Rust Bin for $$bin"; \
		(cd $(RUST_DIR) && cargo lambda build --release --arm64 --bin $$bin --output-format zip); \
		echo "Checking build output for $$bin"; \
		ls -la target/lambda/$$bin/; \
		mkdir -p $(DIST)/$$bin; \
		cp target/lambda/$$bin/bootstrap.zip $(DIST)/$$bin/$$bin.zip; \
	done

ecr-login:
	aws ecr get-login-password --region $(AWS_REGION) | docker login --username AWS --password-stdin $(AWS_ACCOUNT_ID).dkr.ecr.$(AWS_REGION).amazonaws.com

build-py-5min:
	DOCKER_BUILDKIT=0 docker build -t $(ECR_REPO_5MIN):$(TAG) $(PY_5MIN_DIR)

push-py-5min: ecr-login build-py-5min
	docker tag $(ECR_REPO_5MIN):$(TAG) $(ECR_URI_5MIN)
	docker push $(ECR_URI_5MIN)

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