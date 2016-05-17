
default: all

all: confgen

get-deps:
	pushd gohome/src/confgen/; make get-deps; popd

confgen:
	pushd gohome/src/confgen/; make; popd

test:
	pushd gohome/src/confgen/; make test; popd

clean: clean-confgen

clean-confgen:
	pushd gohome/src/confgen/; make clean; popd

image:
	docker build --rm -t kinoko/web-proxy .
