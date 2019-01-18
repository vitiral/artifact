build-frontend: 
	cargo-web deploy -p artifact-frontend --release --target=wasm32-unknown-unknown

build: build-frontend
	cargo build -p artifact-app




