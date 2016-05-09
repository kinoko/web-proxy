
default: all

all: confgen

confgen:
	pushd gohome/src/confgen/; make; popd

clean: clean-confgen

clean-confgen:
	pushd gohome/src/confgen/; make clean; popd

image:
	docker build --rm -t ci.mdl.cs.tsukuba.ac.jp/kinoko/web-proxy .
