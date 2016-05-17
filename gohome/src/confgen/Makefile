
GOPATH := $(realpath ${PWD}/../../)
GOOS := linux
GOARCH := amd64
GO15VENDOREXPERIMENT := 1

export GOPATH
export GOOS
export GOARCH
export GO15VENDOREXPERIMENT

default: build


build: 
	go build -v confgen

clean:
	go clean

get-deps:
	glide install

test:
	go test $(glide novendor)
