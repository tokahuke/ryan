publish:
	-docker run --rm -v $(shell pwd):/io ghcr.io/pyo3/maturin publish \
		--compatibility manylinux2014 \
		--interpreter ${v} \
		--username ${user} \
		--password ${password} || echo "Que sera, sera! Best effort."

publish-all:
	echo '3.7 3.8 3.9 3.10' | xargs -n 1 -I % make user=${user} password=${password} v=% publish
