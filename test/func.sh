#!/bin/bash

set -e

# 共通関数を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

setup_test

# func.cをコンパイル
cc -c $SCRIPT_DIR/bin/func.c -o $SCRIPT_DIR/bin/func.o

assert_func 3 '
extern int foo();

int main() {
    return foo();
}'
assert_func 3 '
int hoge(int x) {
    return x + 1;
}
int main() {
    return hoge(2);
}'
assert_func 12 '
int add(int x, int y) {
    return x + y * 2;
}
int main() {
    int a;
    a = 5;
    return add(2, a);
}'
assert_func 3 '
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
assert_func 3 '
int main() {
    int x;
    int *y;
    y = &x;
    *y = 3;
    return x;
}'
assert_func 8 '
extern void alloc4(int **p, int a, int b, int c, int d);

int main() {
    int *p;
    alloc4(&p, 1, 2, 4, 8);
    int *q;
    q = p + 2;
    *q;
    q = p + 3;
    return *q;
}'
assert_func 6 '
int add(int a, int b, int c) {
    return a + b + c;
}
int main() {
    return add(1, 2, 3);
}'
assert_func 8 '
int a;

int main() {
    int b;
    a = 3;
    b = 5;
    return a + b;
}'
assert_func 3 '
int main() {
    int a[2];
    *a = 1;
    *(a + 1) = 2;
    int *p;
    p = a;
    return *p + *(p + 1);
}'
assert_func 3 '
int a[2];
int main() {
    *a = 1;
    *(a + 1) = 2;
    int *p;
    p = a;
    return *p + *(p + 1);
}'
assert_func 3 '
int main() {
    int a[2];
    a[0] = 1;
    a[1] = 2;
    int *p;
    p = a;
    return p[0] + p[1];
}'
assert_func 6 '
int main() {
    char x[20];
    x[0] = -1;
    x[17] = 2;
    int y;
    y = 4;
    return x[17] + y;
}'
assert_func 98 '
char main() {
    char *a;
    a = "abc";
    return a[1];
}'
assert_func 15 '
int main() {
    int a = 3;
    int b = a * 5;
    int *c = &b;
    int **d = &c;
    return b;
}'
# GCCがlibcをリンクしてくれるおかげでprintfが使える
assert_func 0 '
extern void printf(const char *fmt, ...);

int main() {
    char *a = "Hello, World! %d\n";
    printf(a, 3);
    return 0;
}'
assert_func 5 '
extern void printf(const char *fmt, ...);

int a = 5;
short b = 3;
long c = 8;
char d = 2;
int *e = &a;
int **f = &e;
char *g = "Hello\n";
int main() {
    char *h = "World\n";
    printf(g);
    printf(h);
    return **f;
}'
assert_func 8 '
int a[3] = {1, 2, 3};
int b[] = {4, 5};
int main() {
    return a[2] + b[1];
}
'
assert_func 5 '
int main() {
    int a;
    int b = 3;
    {
        int b = 5;
        a = b;
    }
    return a;
}
'
