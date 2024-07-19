dev:
	cross build --target aarch64-unknown-linux-musl
	scp ./target/aarch64-unknown-linux-musl/debug/lkaas-kerong root@192.168.100.16:/etc/init.d/

release:
	cross build --release --target aarch64-unknown-linux-musl
	scp ./target/aarch64-unknown-linux-musl/release/lkaas-kerong root@192.168.100.16:/etc/init.d/
