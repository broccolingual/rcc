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

assert 47 '5 + 6 * 7;'
assert 15 '5 * (9 - 6);'
assert 4 '(3 + 5) / 2;'
assert 2 '8 % 3;'
assert 0 '(3 + 5) % 4;'

assert 10 '-10 + 20;'
assert 10 '- -10;'
assert 10 '- - +10;'

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

# while statement
assert 5 'i = 0; while (i < 5) i = i + 1; return i;'

# for statement
assert 55 'sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'

# do while statement
assert 5 'i = 0; do i = i + 1; while (i < 5); return i;'

# block statement
assert 8 '{ a = 3; b = 5; return a + b; }'
assert 10 '{ a = 3; b = 5; { c = 2; return a + b + c; } }'
assert 55 'sum = 0; { i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } } return sum;'

echo OK