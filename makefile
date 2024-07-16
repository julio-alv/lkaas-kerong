dev:
	cross build --target x86_64-unknown-linux-gnu
	scp ./target/x86_64-unknown-linux-gnu/debug/lkaas-kerong user@192.168.100.13:/home/user

release:
	cross build --release --target x86_64-unknown-linux-gnu
	scp ./target/x86_64-unknown-linux-gnu/release/lkaas-kerong user@192.168.100.13:/home/user
