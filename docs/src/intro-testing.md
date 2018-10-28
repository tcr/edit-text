# Testing

There are two types of tests, integration tests and unit tests.

## Unit Tests

To run unit tests, run `./tools test`.

## Integration Tests

Run `./tools test --integration` to run the test suite with integrated tests.

This runs simulated tests using headless browsers running concurrent editing operation. You should install a WebDriver implementation such as  `geckodriver`:

```
macOS:
  brew install geckodriver
Ubuntu:
  npm install -g geckodriver
Windows:
  choco install selenium-gecko-driver
```

To run just the integrated tests, run './tools test --no-unit --integration'.

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
