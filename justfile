set shell := ["bash", "-uc"]
set export

registry := "harbor.crungo.net/scout"
binary_export_dest := "/mnt/nas"

# local only, CI runners have difficulty running cross-rs due to docker-in-docker issues
build-arm64-binary:
	cross build --target=aarch64-unknown-linux-musl --release
	cp target/aarch64-unknown-linux-musl/release/treat-dispenser-api {{binary_export_dest}}/treat-dispenser-api-aarch64

build-deb-package:
	just build-arm64-binary
	cargo deb --target aarch64-unknown-linux-musl --no-build --no-strip
	
test:
	cargo test -- --show-output

get-latest-deb-release:
	gh release download -p '*.deb' --dir dist/