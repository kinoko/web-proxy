
default: all

all: confgen

get-deps:
	cd gohome/src/confgen/; make get-deps; cd ../../../

confgen:
	cd gohome/src/confgen/; make; cd ../../../

test:
	cd gohome/src/confgen/; make test; cd ../../../

clean: clean-confgen

clean-confgen:
	cd gohome/src/confgen/; make clean; cd ../../../

image:
	docker build --rm -t kinoko/web-proxy .
