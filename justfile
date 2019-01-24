build-frontend: 
	cargo-web deploy -p artifact-frontend --release --target=wasm32-unknown-unknown

check: build-frontend
	cargo check -p artifact-app -j$(( $(nproc) * 3))

build: build-frontend
	cargo build -p artifact-app -j$(( $(nproc) * 3))

build-release: build-frontend
	cargo build -p artifact-app --release -j$(( $(nproc) * 3))

test: build-frontend
	cargo test -p artifact-app -j$(( $(nproc) * 3))
