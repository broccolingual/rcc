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
    echo -e "\033[32m( OK )\033[0m $input => $actual"
  else
    echo -e "\033[31m( NG )\033[0m $input => $expected expected, but got $actual"
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
int add(int x, int y) {
    return x + y;
}

int main() {
    int a;
    int b;
    b = 1;
    a = b;
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
    *q;
    q = p + 3;
    return *q;
}'
assert 6 '
int add(int a, int b, int c) {
    return a + b + c;
}
int main() {
    return add(1, 2, 3);
}'
assert 0 '
int x;
int a[3];
int *b;

int main() {
    int i;
    int **j;
    return 0;
}'
assert 0 '
int main() {
    int a[3];
    return 0;
}'

echo OK