#!/bin/bash

cargo build

assert() {
  expected="$1"
  input="$2"

  ./target/debug/c-compiler -i "int main() { $input }" > ./bin/tmp.s
  cc -g -o ./bin/tmp ./bin/tmp.s
  ./bin/tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo -e "\033[32m( OK )\033[0m $input => $actual"
  else
    echo -e "\033[31m( NG )\033[0m $input => $expected expected, but got $actual"
    exit 1
  fi
}

echo + arithmetic operators
assert 47 'return 5 + 6 * 7;'
assert 15 'return 5 * (9 - 6);'
assert 4 'return (3 + 5) / 2;'
assert 2 'return 8 % 3;'
assert 0 'return (3 + 5) % 4;'

echo + bitwise operators
assert 7 'return 3 | 5;' # 3 (011) | 5 (101) = 7 (111)
assert 1 'return 3 & 5;' # 3 (011) & 5 (101) = 1 (001)
assert 6 'return 3 ^ 5;' # 3 (011) ^ 5 (101) = 6 (110)
assert 32 'return 1 << 5;' # 1 (0001) << 5 = 32 (100000)
assert 2 'return 8 >> 2;' # 8 (1000) >> 2 = 2 (0010)
assert 5 'return ~-5 + 1;' # ~-5 (..11111010) + 1 = 5 (00000101)
assert 255 'return ~0 & 255;'          # ~0 is -1, masked to 255 -> 255
assert 0 'return (8 >> 2) & 1;'  # (8 >> 2) = 2, 2 & 1 = 0

echo + unary operators
assert 10 'return +10;'
assert 10 'return -10 + 20;'
assert 10 'return - -10;'
assert 10 'return - - +10;'

echo + comparison operators
assert 0 'return 0 == 1;'
assert 1 'return 42 == 42;'
assert 1 'return 0 != 1;'
assert 0 'return 42 != 42;'
assert 1 'return 0 < 1;'
assert 0 'return 1 < 1;'
assert 0 'return 2 < 1;'
assert 1 'return 0 <= 1;'
assert 1 'return 1 <= 1;'
assert 0 'return 2 <= 1;'
assert 1 'return 1 > 0;'
assert 0 'return 1 > 1;'
assert 0 'return 1 > 2;'
assert 1 'return 1 >= 0;'
assert 1 'return 1 >= 1;'
assert 0 'return 1 >= 2;'
assert 1 'int x; x = 1; 0 && (x = 5); return x;' # 右辺が評価されないことを確認
assert 1 'int x; x = 1; 1 || (x = 5); return x;' # 右辺が評価されないことを確認
assert 1 'int x; x = 0; 0 || (x = 1); return x;' # 右辺が評価されないことを確認

echo + local variables
assert 3 'int a; a = 3; return a;'
assert 13 'int a; int b; a = 3; b = 5 * 2; return a + b;'
assert 13 'int t; int e; int r; t = 3; e = 5 * 2; r = t + e; return r;'
assert 13 'int three; int ten; int result; three = 3; ten = 5 * 2; result = three + ten; return result;'

echo + assignment operators
assert 7 'int a; a = 3; a += 4; return a;' # addition
assert 2 'int b; b = 5; b -= 3; return b;' # subtraction
assert 15 'int c; c = 3; c *= 5; return c;' # multiplication
assert 4 'int d; d = 20; d /= 5; return d;' # division
assert 3 'int e; e = 3; e %= 4; return e;' # remainder
assert 7 'int f; f = 3; f |= 5; return f;' # bitwise OR
assert 1 'int g; g = 3; g &= 5; return g;' # bitwise AND
assert 6 'int h; h = 3; h ^= 5; return h;' # bitwise XOR
assert 16 'int i; i = 1; i <<= 4; return i;' # left shift
assert 2 'int j; j = 8; j >>= 2; return j;' # right shift
assert 3 'int a; int b; a = b = 3; return a;'
assert 9 'int a; int b; a = 3; b = 1; a *= b += 2; return a;'

echo + pre/post increment/decrement operators
assert 6 'int a; a = 5; return ++a;' # pre-increment
assert 5 'int a; a = 5; return a++;' # post-increment
assert 4 'int b; b = 5; return --b;' # pre-decrement
assert 5 'int b; b = 5; return b--;' # post-decrement
assert 9 'int a; int b; int c; a = 3; b = 5; c = ++a + b++; return c;' # mixed usage

echo + return statement
assert 3 'return 3;'
assert 8 'int a; a = 3; return a + 5;'
assert 8 'return 8; 5;'

echo + if statement
assert 3 'int a; a = 1; if (a % 2 == 1) return 3; else return 4;'
assert 7 'int a; a = 4; if (a % 2 == 1) return 3; else return a + 3;'
assert 4 'int a; a = 4; if (a % 2 == 1) return 3; return 4;'
assert 2 'int a; a = 1; if (a > 5) return 5; else if (a < 2) return 2; else return 3;'
assert 5 'int a; a = 6; if (a > 5) { a = 4; 1 + 2; } else { a = 3; } a = 5; return a;'

echo + while statement
assert 5 'int i; i = 0; while (i < 5) i = i + 1; return i;'
assert 3 'int i; i = 0; while (1) { i = i + 1; if (i == 3) break; } return i;'
assert 25 'int i; int sum; i = 0; sum = 0; while (i < 10) { i = i + 1; if (i % 2 == 0) continue; sum = sum + i; } return sum;'

echo + for statement
assert 55 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'
assert 15 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i > 5) break; sum = sum + i; } return sum;'
assert 25 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i % 2 == 0) continue; sum = sum + i; } return sum;'

echo + do while statement
assert 5 'int i; i = 0; do i = i + 1; while (i < 5); return i;'
assert 3 'int i; i = 0; do { i = i + 1; if (i == 3) break; } while (1); return i;'
assert 25 'int i; int sum; i = 0; sum = 0; do { i = i + 1; if (i % 2 == 0) continue; sum = sum + i; } while (i < 10); return sum;'

echo + goto and labeled statements
assert 5 'int a; a = 0; goto skip; a = 10; skip: a = a + 5; return a;'

echo + logical operators
assert 1 'return 1 && 1;'
assert 0 'return 1 && 0;'
assert 1 'int i; i = 3; if (i > 0 && i < 5) return 1; else return 0;'
assert 0 'int i; i = 0; if (i > 0 && i < 5) return 1; else return 0;'
assert 0 'int i; i = 5; if (i > 0 && i < 5) return 1; else return 0;'
assert 1 'return 1 || 0;'
assert 0 'return 0 || 0;'
assert 0 'int i; i = 0; if (i < 0 || i > 0) return 1; else return 0;'
assert 1 'int i; i = 3; if (i < 0 || i > 0) return 1; else return 0;'
assert 1 'int i; i = -2; if (i < 0 || i > 0) return 1; else return 0;'
assert 0 'return !1;'
assert 1 'return !0;'
assert 1 'int i; i = 0; if (!i) return 1; else return 0;'
assert 0 'int i; i = 3; if (!i) return 1; else return 0;'
assert 0 'return 0 || 1 && 0;'

echo + ternary operator
assert 3 'int a; int b; a = 3; b = 5; return a < b ? a : b;'
assert 5 'int a; int b; a = 3; b = 5; return a > b ? a : b;'
assert 8 'int a; int b; int c; a = 3; b = 5; c = 2; return a + b > 7 ? a + b : b + c;'
assert 2 'int a; a = 2; a == 1 ? 1 : a == 2 ? 2 : 3;'

echo + block statement
assert 8 '{ int a; int b; a = 3; b = 5; return a + b; }'
assert 10 '{ int a; int b; int c; a = 3; b = 5; c = 2; return a + b + c; }'
assert 55 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'

echo + sizeof operator
assert 4 'int x; return sizeof(x);'
assert 8 'int *p; return sizeof(p);'
assert 4 'return sizeof 1;'
assert 4 'int x; return sizeof(x + 3);'
assert 8 'int *p; return sizeof(p + 3);'
assert 32 'int a[8]; return sizeof(a);'
assert 8 'int a[8]; return sizeof(&a);'
assert 128 'int a[8][4]; return sizeof(a);'
# assert 16 'int a[8][4]; return sizeof(a[0]);' # TODO: 多次元配列の対応

echo + operator precedence and associativity
assert 8 'return 1 << 2 + 1;'           # 1 << (2 + 1) = 8
assert 5 'return (1 << 2) + 1;'        # (1 << 2) + 1 = 5
assert 0 'return 1 ^ 2 ^ 3;'           # (1 ^ 2) ^ 3 = 0
assert 1 'return 1 < 2 < 3;'           # (1 < 2) = 1, then 1 < 3 -> 1

echo + pointer dereference and address-of
assert 3 'int a; int *b; a = 3; b = &a; return *b;'
assert 7 'int a; int *p; p = &a; *p = 7; return a;'

echo + octal and hexadecimal number literals
assert 10 'return 012;'
assert 26 'return 0x1a;'
assert 255 'return 0xff;' # exit codeは8ビットなので255まで
assert 35 'return 0x11 + 0x12;'
assert 19 'return 011 + 012;'

echo + struct declaration
assert 0 'struct Point { int a, b; int *p; }; return 0;'
assert 0 'struct { int a, b; int *p; } point; return 0;'
assert 0 'struct { struct {int a;} inner1, inner2; int b; } point; return 0;'

echo OK
