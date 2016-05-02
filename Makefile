
default: all

all: confgen

confgen:
	pushd gohome/src/confgen/; make; popd

clean: clean-confgen

clean-confgen:
	pushd gohome/src/confgen/; make clean; popd
