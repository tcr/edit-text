# Testing

There are two types of tests, integration tests and unit tests.

## Unit Tests

To run unit tests, run `cargo test`.

## Integration Tests

Run `./x.rs test` to run the integration test. It requires `geckodriver` to be installed.

```
macOS:
  brew install geckodriver
Ubuntu:
  npm install -g geckodriver
Windows:
  choco install selenium-gecko-driver
```

<!--

## Oatie testing

* Transform test (oatie/in, transform_test.sh)
* Unit tests

Missing:

* Virtual monkey (on random schemas?)

## edit-text testing

* 
* `geckodriver` integrated test (./x.rs test)

Manual:

* Multi Monkey (/$/multi.html)
* Virtual Monkey (uh....)

Missing:

* Unit tests

-->