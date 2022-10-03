.PHONY test:
	cargo t
	php -dextension=./target/debug/libtape.so php_tests/vendor/bin/phpunit php_tests/tests
