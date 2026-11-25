FROM rust:trixie

# Rust toolchain: wasm target and the components used by editors / CI
RUN rustup target add wasm32-unknown-unknown \
	&& rustup component add rustfmt clippy

# Prebuilt cargo tools via cargo-binstall (much faster than `cargo install`)
#   wasm-pack  : build the wasm package for the web page
#   cargo-sweep: prune target/ (see postStartCommand)
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
		https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
	&& cargo binstall -y --locked wasm-pack cargo-sweep
