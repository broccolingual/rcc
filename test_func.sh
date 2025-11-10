#!/bin/bash

cargo build

assert() {
  expected="$1"
  input="$2"

  ./target/debug/c-compiler "$input" > ./bin/tmp.s
  cc -o ./bin/tmp ./bin/tmp.s
  ./bin/tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "( OK ) $input => $actual"
  else
    echo "( NG ) $input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 47 'main() { return 5 + 6 * 7; }'
assert 3 'foo(x) { return x + 1; } main() { return foo(2); }'
assert 7 'add(x, y) { return x + y; } main() { a = 5; return add(2, a); }'

echo OK