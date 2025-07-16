set shell := ["bash", "-uc"]
set export

registry := "harbor.crungo.net/scout"
binary_export_dest := "/mnt/nas"

# local only, CI runners have difficulty running cross-rs due to docker-in-docker issues
build-arm64-binary:
	cross build --target=aarch64-unknown-linux-musl --release
	cp target/aarch64-unknown-linux-musl/release/treat-dispenser-api {{binary_export_dest}}/treat-dispenser-api-aarch64

build-deb-package:
	@echo Not implemented yet

test:
	cargo test -- --show-output