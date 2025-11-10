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

assert 47 '
int main() {
    return 5 + 6 * 7;
}'
assert 3 '
int foo(int x) {
    return x + 1;
}
int main() {
    return foo(2);
}'
assert 12 '
int add(int x, int y) {
    return x + y * 2;
}
int main() {
    int a;
    a = 5;
    return add(2, a);
}'
assert 3 '
int main() {
    int x;
    int *y;
    y = &x;
    *y = 3;
    return x;
}
'

echo OK