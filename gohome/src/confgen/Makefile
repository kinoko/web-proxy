
GOPATH := $(realpath ${PWD}/../../)
GOOS := darwin
GOARCH := amd64

export GOPATH
export GOOS
export GOARCH

default: build


build:
	go build -v confgen

clean:
	go clean
