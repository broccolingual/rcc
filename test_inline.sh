#!/bin/bash

set -e

mkdir -p ./bin

cargo build

assert() {
  expected="$1"
  input="$2"

  ./target/debug/c-compiler -i "int main() { $input }" > ./bin/tmp.s || {
    echo -e "\033[31m( ERROR )\033[0m Compilation failed: $input"
    exit 1
  }

  cc -g -o ./bin/tmp ./bin/tmp.s || {
    echo -e "\033[31m( ERROR )\033[0m Linking failed: $input"
    exit 1
  }

  set +e
  ./bin/tmp
  actual="$?"
  set -e

  if [ "$actual" = "$expected" ]; then
    echo -e "\033[32m( OK )\033[0m $input => $actual"
  else
    echo -e "\033[31m( NG )\033[0m $input => $expected expected, but got $actual"
    exit 1
  fi
}

echo + literals and basic expressions
assert 0 'return 0;'
assert 42 'return 42;'
assert 255 'return 255;'

echo + arithmetic operators
assert 21 'return 5 + 2 * 8;'
assert 47 'return 5 + 6 * 7;'
assert 15 'return 5 * (9 - 6);'
assert 4 'return (3 + 5) / 2;'
assert 2 'return 8 % 3;'
assert 0 'return (3 + 5) % 4;'
assert 35 'return 5 + 6 * 7 - 12;'
assert 8 'return 2 + 3 * 2;'

echo + arithmetic edge cases
assert 0 'return 0 * 42;'
assert 0 'return 42 * 0;'
assert 42 'return 42 / 1;'
assert 1 'return 42 / 42;'
assert 0 'return 0 % 5;'
assert 1 'return 1 % 2;'

echo + unary operators
assert 10 'return +10;'
assert 245 'return -10 + 255;'  # -10 + 255 = 245 (8-bit range)
assert 10 'return -(-10);'
assert 10 'return - - +10;'
assert 254 'return -1 + 255;'   # -1 + 255 = 254

echo + bitwise operators
assert 7 'return 3 | 5;'
assert 1 'return 3 & 5;'
assert 6 'return 3 ^ 5;'
assert 32 'return 1 << 5;'
assert 2 'return 8 >> 2;'
assert 245 'return ~10 & 255;'  # ~10 = 245 (8-bit masked)
assert 0 'return 5 & 0;'
assert 5 'return 5 | 0;'
assert 0 'return 5 ^ 5;'
assert 1 'return 1 << 0;'
assert 8 'return 8 >> 0;'

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

echo + logical operators
assert 1 'return 1 && 1;'
assert 0 'return 1 && 0;'
assert 0 'return 0 && 1;'
assert 0 'return 0 && 0;'
assert 1 'return 1 || 0;'
assert 1 'return 0 || 1;'
assert 1 'return 1 || 1;'
assert 0 'return 0 || 0;'
assert 0 'return !1;'
assert 1 'return !0;'
assert 1 'return !!1;'
assert 0 'return !!0;'

echo + short circuit evaluation
assert 1 'int x; x = 1; 0 && (x = 5); return x;'
assert 1 'int x; x = 1; 1 || (x = 5); return x;'
assert 1 'int x; x = 0; 0 || (x = 1); return x;'

echo + operator precedence
assert 8 'return 1 << 2 + 1;'
assert 5 'return (1 << 2) + 1;'
assert 14 'return 2 + 3 * 4;'
assert 20 'return (2 + 3) * 4;'
assert 0 'return 0 || 1 && 0;'
assert 1 'return (0 || 1) && 1;'

echo + associativity
assert 0 'return 1 ^ 2 ^ 3;'
assert 1 'return 1 < 2 < 3;'
assert 10 'return 100 / 5 / 2;'
assert 50 'return 100 / (5 / 2);'

echo + variables and assignment
assert 3 'int a; a = 3; return a;'
assert 13 'int a; int b; a = 3; b = 10; return a + b;'
assert 6 'int a; int b; int c; a = 1; b = 2; c = 3; return a + b + c;'

echo + assignment operators
assert 7 'int a; a = 3; a += 4; return a;'
assert 2 'int b; b = 5; b -= 3; return b;'
assert 15 'int c; c = 3; c *= 5; return c;'
assert 4 'int d; d = 20; d /= 5; return d;'
assert 3 'int e; e = 3; e %= 4; return e;'
assert 7 'int f; f = 3; f |= 5; return f;'
assert 1 'int g; g = 3; g &= 5; return g;'
assert 6 'int h; h = 3; h ^= 5; return h;'
assert 16 'int i; i = 1; i <<= 4; return i;'
assert 2 'int j; j = 8; j >>= 2; return j;'
assert 3 'int a; int b; a = b = 3; return a;'

echo + increment and decrement
assert 6 'int a; a = 5; return ++a;'
assert 5 'int a; a = 5; return a++;'
assert 4 'int b; b = 5; return --b;'
assert 5 'int b; b = 5; return b--;'
assert 6 'int a; a = 5; ++a; return a;'
assert 6 'int a; a = 5; a++; return a;'
assert 4 'int a; a = 5; --a; return a;'
assert 4 'int a; a = 5; a--; return a;'

echo + control flow - if statements
assert 3 'if (1) return 3; return 4;'
assert 4 'if (0) return 3; return 4;'
assert 3 'if (1) return 3; else return 4;'
assert 4 'if (0) return 3; else return 4;'
assert 2 'if (0) return 1; else if (1) return 2; else return 3;'
assert 3 'if (0) return 1; else if (0) return 2; else return 3;'

echo + control flow - while loops
assert 10 'int i; i = 0; while (i < 10) i = i + 1; return i;'
assert 0 'int i; i = 0; while (0) i = i + 1; return i;'
assert 3 'int i; i = 0; while (1) { i = i + 1; if (i == 3) break; } return i;'

echo + control flow - for loops
assert 55 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'
assert 15 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i > 5) break; sum = sum + i; } return sum;'
assert 25 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i % 2 == 0) continue; sum = sum + i; } return sum;'
assert 0 'int i; for (i = 0; 0; i = i + 1) i = i + 1; return i;'

echo + control flow - do while
assert 1 'int i; i = 0; do i = i + 1; while (i < 1); return i;'
assert 5 'int i; i = 0; do i = i + 1; while (i < 5); return i;'

echo + ternary operator
assert 2 'return 1 ? 2 : 3;'
assert 3 'return 0 ? 2 : 3;'
assert 3 'int a; int b; a = 3; b = 5; return a < b ? a : b;'
assert 5 'int a; int b; a = 3; b = 5; return a > b ? a : b;'

echo + block statements
assert 8 '{ int a; int b; a = 3; b = 5; return a + b; }'
assert 3 '{ { { return 3; } } }'
# assert 5 '{ int a; a = 2; { int a; a = 3; a = a + 2; } return a; }' # TODO: 変数のスコープ

echo + sizeof operator
assert 4 'return sizeof(int);'
assert 8 'return sizeof(int *);'
assert 4 'int x; return sizeof(x);'
assert 8 'int *p; return sizeof(p);'
assert 4 'return sizeof 1;'
assert 32 'int a[8]; return sizeof(a);'

echo + pointers
assert 3 'int a; int *b; a = 3; b = &a; return *b;'
assert 7 'int a; int *p; p = &a; *p = 7; return a;'
assert 5 'int a; int *p; int **pp; a = 5; p = &a; pp = &p; return **pp;'

echo + arrays
assert 3 'int a[5]; a[0] = 3; return a[0];'
assert 8 'int a[5]; a[0] = 3; a[1] = 5; return a[0] + a[1];'
assert 2 'int a[3]; *(a + 1) = 2; return a[1];'
assert 1 'int a[2][3]; a[1][2] = 1; return a[1][2];'
assert 5 'int a[2][3]; *(*(a + 1) + 2) = 5; return a[1][2];'
assert 2 'int i = 2; int a[2][3]; a[1][i - 1] = 2; return a[1][1];'

echo + goto and labels
assert 5 'int a; a = 0; goto skip; a = 10; skip: a = a + 5; return a;'

echo + number literals
assert 10 'return 012;'
assert 26 'return 0x1a;'
assert 255 'return 0xff;'

echo OK
