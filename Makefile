.PHONY test:
	cargo t
	cargo b
	php -dextension=./target/debug/libtape.so php_tests/vendor/bin/phpunit php_tests/tests
