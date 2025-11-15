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
assert 8 '
int a;

int main() {
    int b;
    a = 3;
    b = 5;
    return a + b;
}'
assert 3 '
int main() {
    int a[2];
    *a = 1;
    *(a + 1) = 2;
    int *p;
    p = a;
    return *p + *(p + 1);
}'
assert 3 '
int a[2];
int main() {
    *a = 1;
    *(a + 1) = 2;
    int *p;
    p = a;
    return *p + *(p + 1);
}'
assert 3 '
int main() {
    int a[2];
    a[0] = 1;
    a[1] = 2;
    int *p;
    p = a;
    return p[0] + p[1];
}'
assert 6 '
int main() {
    char x[20];
    x[0] = -1;
    x[17] = 2;
    int y;
    y = 4;
    return x[17] + y;
}'
assert 98 '
int main() {
    char *a;
    a = "abc";
    return a[1];
}'
# GCCがlibcをリンクしてくれるおかげでprintfが使える
assert 0 '
int main() {
    printf("Hello, World! %d\n", 3);
    return 0;
}'

echo OK