OS := $(shell uname)

ifeq ($(OS), Darwin)
	TAPE_LIB := ./target/debug/libtape.dylib 
endif

ifeq ($(OS), Linux)
	TAPE_LIB := ./target/debug/libtape.so 
endif

.PHONY test:
	cargo t
	cargo b
	php -dextension=$(TAPE_LIB) php_tests/vendor/bin/phpunit php_tests/tests --stop-on-error
