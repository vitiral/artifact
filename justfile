build-frontend: 
	cargo-web deploy -p artifact-frontend --release --target=wasm32-unknown-unknown

check: build-frontend
	cargo check -p artifact-app

build: build-frontend
	cargo build -p artifact-app

build-release: build-frontend
	cargo build -p artifact-app --release

test: build-frontend
	cargo test -p artifact-app
