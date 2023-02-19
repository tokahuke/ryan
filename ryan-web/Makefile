
build:
	wasm-pack build -t bundler

publish: build
	echo ${NODE_AUTH_TOKEN}
	wasm-pack publish -t bundler
