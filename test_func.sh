#!/bin/bash

cargo build

cc -c ./bin/func.c -o ./bin/func.o

assert() {
  expected="$1"
  input="$2"

  ./target/debug/c-compiler "$input" > ./bin/tmp.s
  cc -o ./bin/tmp ./bin/tmp.s ./bin/func.o
  ./bin/tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "( OK ) $input => $actual"
  else
    echo "( NG ) $input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 3 '
int main() {
    return foo();
}'
assert 3 '
int hoge(int x) {
    return x + 1;
}
int main() {
    return hoge(2);
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
}'
assert 8 '
int main() {
    int *p;
    alloc4(&p, 1, 2, 4, 8);
    int *q;
    q = p + 2;
    *q;  // → 4
    q = p + 3;
    return *q;
}'

echo OK