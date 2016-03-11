
default: all

all: confgen

confgen:
	pushd gohome/src/confgen/; make; popd
