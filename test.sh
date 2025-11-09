#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  cargo run -q -- "$input" > ./bin/tmp.s
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

# arithmetic operators
assert 47 '5 + 6 * 7;'
assert 15 '5 * (9 - 6);'
assert 4 '(3 + 5) / 2;'
assert 2 '8 % 3;'
assert 0 '(3 + 5) % 4;'

# bitwise operators
assert 7 '3 | 5;' # 3 (011) | 5 (101) = 7 (111)
assert 1 '3 & 5;' # 3 (011) & 5 (101) = 1 (001)
assert 6 '3 ^ 5;' # 3 (011) ^ 5 (101) = 6 (110)
assert 32 '1 << 5;' # 1 (0001) << 5 = 32 (100000)
assert 2 '8 >> 2;' # 8 (1000) >> 2 = 2 (0010)

# unary operators
assert 10 '-10 + 20;'
assert 10 '- -10;'
assert 10 '- - +10;'

# comparison operators
assert 0 '0 == 1;'
assert 1 '42 == 42;'
assert 1 '0 != 1;'
assert 0 '42 != 42;'

assert 1 '0 < 1;'
assert 0 '1 < 1;'
assert 0 '2 < 1;'
assert 1 '0 <= 1;'
assert 1 '1 <= 1;'
assert 0 '2 <= 1;'

assert 1 '1 > 0;'
assert 0 '1 > 1;'
assert 0 '1 > 2;'
assert 1 '1 >= 0;'
assert 1 '1 >= 1;'
assert 0 '1 >= 2;'

# local variables
assert 3 'a = 3;'
assert 3 'a = 3; a;'
assert 13 'a = 3; b = 5 * 2; a + b;'
assert 13 't = 3; e = 5 * 2; r = t + e; r;'
assert 13 'three = 3; ten = 5 * 2; result = three + ten; result;'

# return statement
assert 3 'return 3;'
assert 8 'a = 3; return a + 5;'
assert 8 'return 8; 5;'

# if statement
assert 3 'a = 1; if (a % 2 == 1) return 3; else return 4;'
assert 7 'a = 4; if (a % 2 == 1) return 3; else return a + 3;'
assert 4 'a = 4; if (a % 2 == 1) return 3; return 4;'
assert 2 'a = 1; if (a > 5) return 5; else if (a < 2) return 2; else return 3;'

# while statement
assert 5 'i = 0; while (i < 5) i = i + 1; return i;'

# break/continue in while statements
assert 3 'i = 0; while (1) { i = i + 1; if (i == 3) break; } return i;'
assert 25 'i = 0; sum = 0; while (i < 10) { i = i + 1; if (i % 2 == 0) continue; sum = sum + i; } return sum;'

# for statement
assert 55 'sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'

# continue/break in for statement
assert 15 'sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i > 5) break; sum = sum + i; } return sum;'
assert 25 'sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i % 2 == 0) continue; sum = sum + i; } return sum;'

# do while statement
assert 5 'i = 0; do i = i + 1; while (i < 5); return i;'

# continue/break in do while statement
assert 3 'i = 0; do { i = i + 1; if (i == 3) break; } while (1); return i;'
assert 25 'i = 0; sum = 0; do { i = i + 1; if (i % 2 == 0) continue; sum = sum + i; } while (i < 10); return sum;'

# logical operators
assert 1 '1 && 1;'
assert 0 '1 && 0;'
assert 1 'i = 3; if (i > 0 && i < 5) return 1; else return 0;'
assert 0 'i = 0; if (i > 0 && i < 5) return 1; else return 0;'
assert 0 'i = 5; if (i > 0 && i < 5) return 1; else return 0;'
assert 1 '1 || 0;'
assert 0 '0 || 0;'
assert 0 'i = 0; if (i < 0 || i > 0) return 1; else return 0;'
assert 1 'i = 3; if (i < 0 || i > 0) return 1; else return 0;'
assert 1 'i = -2; if (i < 0 || i > 0) return 1; else return 0;'

# block statement
assert 8 '{ a = 3; b = 5; return a + b; }'
assert 10 '{ a = 3; b = 5; { c = 2; return a + b + c; } }'
assert 55 'sum = 0; { i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } } return sum;'

echo OK