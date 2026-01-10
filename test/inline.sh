#!/bin/bash

set -e

# 共通関数を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

setup_test

# =============================================================================
# リテラル・定数
# =============================================================================
assert_inline 0 'return 0;'
assert_inline 42 'return 42;'
assert_inline 255 'return 0xFF;'
assert_inline 10 'return 012;'  # 8進数
assert_inline 26 'return 0x1a;'  # 16進数
assert_inline 97 "return 'a';"  # 文字定数

# =============================================================================
# 算術演算子
# =============================================================================
assert_inline 21 'return 5 + 2 * 8;'  # 優先順位
assert_inline 15 'return 5 * (9 - 6);'  # 括弧
assert_inline 2 'return 8 % 3;'  # 剰余
assert_inline 10 'return +10;'  # 単項+
assert_inline 245 'return -10 + 255;'  # 単項-
assert_inline 10 'return -(-10);'  # 二重否定

# =============================================================================
# ビット演算子
# =============================================================================
assert_inline 7 'return 3 | 5;'  # OR
assert_inline 1 'return 3 & 5;'  # AND
assert_inline 6 'return 3 ^ 5;'  # XOR
assert_inline 32 'return 1 << 5;'  # 左シフト
assert_inline 2 'return 8 >> 2;'  # 右シフト
assert_inline 245 'return ~10 & 255;'  # NOT
assert_inline 0 'return 5 ^ 5;'  # 同値のXOR = 0

# =============================================================================
# 比較演算子
# =============================================================================
assert_inline 1 'return 42 == 42;'
assert_inline 0 'return 0 == 1;'
assert_inline 1 'return 0 != 1;'
assert_inline 1 'return 0 < 1;'
assert_inline 1 'return 1 <= 1;'
assert_inline 1 'return 1 > 0;'
assert_inline 1 'return 1 >= 1;'

# =============================================================================
# 論理演算子
# =============================================================================
assert_inline 1 'return 1 && 1;'
assert_inline 0 'return 1 && 0;'
assert_inline 1 'return 1 || 0;'
assert_inline 0 'return 0 || 0;'
assert_inline 0 'return !1;'
assert_inline 1 'return !0;'
assert_inline 1 'return !!1;'

# 短絡評価
assert_inline 1 'int x; x = 1; 0 && (x = 5); return x;'  # ANDは評価されない
assert_inline 1 'int x; x = 1; 1 || (x = 5); return x;'  # ORは評価されない
assert_inline 5 'int x; x = 1; 1 && (x = 5); return x;'  # ANDは評価される

# =============================================================================
# 変数と代入
# =============================================================================
assert_inline 3 'int a; a = 3; return a;'
assert_inline 5 'int a = 5; return a;'  # 初期化子
assert_inline 13 'int a; int b; a = 3; b = 10; return a + b;'

# 複合代入
assert_inline 7 'int a; a = 3; a += 4; return a;'
assert_inline 2 'int b; b = 5; b -= 3; return b;'
assert_inline 15 'int c; c = 3; c *= 5; return c;'
assert_inline 4 'int d; d = 20; d /= 5; return d;'
assert_inline 3 'int e; e = 3; e %= 4; return e;'
assert_inline 7 'int f; f = 3; f |= 5; return f;'
assert_inline 1 'int g; g = 3; g &= 5; return g;'
assert_inline 6 'int h; h = 3; h ^= 5; return h;'
assert_inline 16 'int i; i = 1; i <<= 4; return i;'
assert_inline 2 'int j; j = 8; j >>= 2; return j;'

# インクリメント・デクリメント
assert_inline 6 'int a; a = 5; return ++a;'
assert_inline 5 'int a; a = 5; return a++;'
assert_inline 4 'int b; b = 5; return --b;'
assert_inline 5 'int b; b = 5; return b--;'
assert_inline 11 'int a = 5; return a++ + a;'
assert_inline 12 'int a = 5; return ++a + a;'

# =============================================================================
# 制御構文
# =============================================================================

# if文
assert_inline 3 'if (1) return 3; return 4;'
assert_inline 4 'if (0) return 3; return 4;'
assert_inline 3 'if (1) return 3; else return 4;'
assert_inline 4 'if (0) return 3; else return 4;'
assert_inline 2 'if (0) return 1; else if (1) return 2; else return 3;'

# while文
assert_inline 10 'int i; i = 0; while (i < 10) i = i + 1; return i;'
assert_inline 0 'int i; i = 0; while (0) i = i + 1; return i;'
assert_inline 3 'int i; i = 0; while (1) { i = i + 1; if (i == 3) break; } return i;'
assert_inline 15 'int i = 1; int s = 0; while (i <= 5) { s = s + i; i++; } return s;'

# do-while文
assert_inline 1 'int i; i = 0; do i = i + 1; while (i < 1); return i;'
assert_inline 5 'int i; i = 0; do i = i + 1; while (i < 5); return i;'
assert_inline 1 'int i = 0; do i++; while (0); return i;'

# for文
assert_inline 55 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'
assert_inline 15 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i > 5) break; sum = sum + i; } return sum;'
assert_inline 25 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i % 2 == 0) continue; sum = sum + i; } return sum;'
assert_inline 0 'int i; for (i = 0; 0; i = i + 1) i = i + 1; return i;'

# ネストループ
assert_inline 100 'int sum = 0; int i; int j; for (i = 0; i < 10; i++) for (j = 0; j < 10; j++) sum++; return sum;'
assert_inline 50 'int sum = 0; int i; int j; for (i = 0; i < 10; i++) { for (j = 0; j < 10; j++) { if (j == 5) break; sum++; } } return sum;'
assert_inline 24 'int sum = 0; int i; int j; int k; for (i = 0; i < 2; i++) for (j = 0; j < 3; j++) for (k = 0; k < 4; k++) sum++; return sum;'

# 三項演算子
assert_inline 2 'return 1 ? 2 : 3;'
assert_inline 3 'return 0 ? 2 : 3;'
assert_inline 3 'int a; int b; a = 3; b = 5; return a < b ? a : b;'
assert_inline 10 'int x = 5; return x > 0 ? x * 2 : x;'

# ブロック
assert_inline 8 '{ int a; int b; a = 3; b = 5; return a + b; }'
assert_inline 3 '{ { { return 3; } } }'
assert_inline 2 '{ int a; a = 2; { int a; a = 3; a = a + 2; } return a;}'
assert_inline 1 'int a = 1; { int a = 2; } return a;'

# goto
assert_inline 5 'int a; a = 0; goto skip; a = 10; skip: a = a + 5; return a;'
assert_inline 3 'int i = 0; loop: i = i + 1; if (i < 3) goto loop; return i;'

# =============================================================================
# sizeof演算子
# =============================================================================
assert_inline 1 'return sizeof(char);'
assert_inline 2 'return sizeof(short);'
assert_inline 4 'return sizeof(int);'
assert_inline 8 'return sizeof(long);'
assert_inline 8 'return sizeof(char *);'
assert_inline 8 'return sizeof(int **);'
assert_inline 40 'return sizeof(int [10]);'
assert_inline 80 'return sizeof(int [4][5]);'
assert_inline 4 'int x; return sizeof(x);'
assert_inline 8 'int *p; return sizeof(p);'
assert_inline 32 'int a[8]; return sizeof(a);'
assert_inline 4 'return sizeof 1;'
assert_inline 4 'return sizeof(5 + 3);'

# =============================================================================
# ポインタ
# =============================================================================
assert_inline 3 'int a; int *b; a = 3; b = &a; return *b;'
assert_inline 7 'int a; int *p; p = &a; *p = 7; return a;'
assert_inline 5 'int a; int *p; int **pp; a = 5; p = &a; pp = &p; return **pp;'
assert_inline 10 'int a = 10; int *p = &a; return *p;'
assert_inline 15 'int a = 5; int *p = &a; *p = *p + 10; return a;'

# =============================================================================
# 配列
# =============================================================================
assert_inline 3 'int a[5]; a[0] = 3; return a[0];'
assert_inline 8 'int a[5]; a[0] = 3; a[1] = 5; return a[0] + a[1];'
assert_inline 2 'int a[3]; *(a + 1) = 2; return a[1];'
assert_inline 1 'int a[3]; int *p; p = a; a[1] = 1; return *(++p);'
assert_inline 1 'int a[2][3]; a[1][2] = 1; return a[1][2];'
assert_inline 5 'int a[2][3]; *(*(a + 1) + 2) = 5; return a[1][2];'
assert_inline 10 'int a[5]; a[2] = 10; return *(a + 2);'

# 配列初期化
assert_inline 6 'int a[3] = {1, 2, 3}; return a[0] + a[1] + a[2];'
assert_inline 3 'int a[3] = {1, 2}; return a[0] + a[1] + a[2];'
assert_inline 1 'int a[3] = {1}; return a[0] + a[1] + a[2];'
assert_inline 15 'int a[] = {1, 2, 3, 4, 5}; return a[0] + a[1] + a[2] + a[3] + a[4];'
assert_inline 0 'int a[5] = {0}; return a[0] + a[1] + a[2] + a[3] + a[4];'
assert_inline 6 'int x = 1; int y = 2; int z = 3; int *a[3] = {&x, &y, &z}; return *a[0] + *a[1] + *a[2];'

# =============================================================================
# 構造体
# =============================================================================
assert_inline 8 'struct {int a; short b; int c;} s; s.a = 3; s.c = 5; return s.a + s.c;'
assert_inline 3 'struct {struct {int a; int b;} inner; int x;} outer; outer.inner.b = 3; return outer.inner.b;'
assert_inline 3 'struct Point {int x; int y;}; struct Point p; p.x = 1; p.y = 2; return p.x + p.y;'
assert_inline 30 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; return ptr->x + ptr->y;'
assert_inline 70 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; ptr->x = 30; ptr->y = 40; return ptr->x + ptr->y;'
assert_inline 15 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 5; p.y = 10; ptr = &p; return (*ptr).x + (*ptr).y;'

# =============================================================================
# 共用体
# =============================================================================
assert_inline 42 'union {int i; char c;} u; u.i = 42; return u.i;'
assert_inline 10 'union Data {int i; char c; long l;}; union Data d; d.i = 10; return d.i;'
assert_inline 5 'union {int a; int b;} u; u.a = 5; return u.b;'
assert_inline 65 'union {int i; char c;} u; u.i = 65; return u.c;'
assert_inline 1 'union {int i; char c;} u; u.i = 257; return u.c;'
assert_inline 8 'return sizeof(union {int i; long l;});'
assert_inline 4 'return sizeof(union {int i; char c; short s;});'
assert_inline 100 'union Data {int i; long l;}; union Data d; union Data *ptr; d.i = 100; ptr = &d; return ptr->i;'

# 構造体と共用体の組み合わせ
assert_inline 15 'struct S {union {int a; int b;} u; int c;}; struct S s; s.u.a = 10; s.c = 5; return s.u.a + s.c;'
assert_inline 7 'union U {struct {int x; int y;} s; long l;}; union U u; u.s.x = 3; u.s.y = 4; return u.s.x + u.s.y;'

# =============================================================================
# 列挙型
# =============================================================================
assert_inline 0 'enum {RED, GREEN, BLUE}; return RED;'
assert_inline 1 'enum {RED, GREEN, BLUE}; return GREEN;'
assert_inline 2 'enum {RED, GREEN, BLUE}; return BLUE;'
assert_inline 10 'enum {RED = 10, GREEN, BLUE}; return RED;'
assert_inline 11 'enum {RED = 10, GREEN, BLUE}; return GREEN;'
assert_inline 15 'enum {A = 5, B = 10, C = 15}; return C;'
assert_inline 0 'enum Color {RED, GREEN, BLUE}; return RED;'
assert_inline 2 'enum Color {RED, GREEN, BLUE}; enum Color c; c = BLUE; return c;'
assert_inline 1 'enum {RED, GREEN, BLUE}; return RED < GREEN;'
assert_inline 21 'enum {X = 10, Y}; return X + Y;'
assert_inline 100 'enum {MAX = 100}; return MAX;'

# =============================================================================
# キャスト
# =============================================================================
assert_inline 1 'int a = 257; return (char)a;'
assert_inline 0 'int a = 256; return (char)a;'
assert_inline 120 'int a = 0x12345678; return (char)a;'
assert_inline 42 'char c = 42; return (int)c;'
assert_inline 100 'long l = 100; return (int)l;'
assert_inline 5 'int x = 5; return (short)x;'
assert_inline 200 'short s = 200; return (long)s;'
assert_inline 1 'long l = 65537; return (short)l;'
assert_inline 0 'long l = 65536; return (short)l;'
assert_inline 0 'int *p = 0; return (long)p;'
assert_inline 4 'int a = 4; int *p = &a; return *(int *)(char *)p;'
assert_inline 1 'int a = 1; void *p = &a; return *(int *)p;'
assert_inline 2 'return (int)(5 / 2);'
assert_inline 15 'int a = 5; int b = 10; return (short)(a + b);'

# =============================================================================
# typedef
# =============================================================================
assert_inline 42 'typedef int A; A a = 42; return a;'
assert_inline 7 'typedef int A; typedef A B; B b = 7; return b;'
assert_inline 3 'typedef int A; typedef A B; typedef B C; C c = 3; return c;'
assert_inline 8 'typedef int T; T x = 3; T y = 5; return x + y;'
assert_inline 42 'typedef int* P; int x = 42; P p = &x; return *p;'
assert_inline 5 'typedef int* P; typedef P Q; int z = 5; Q q = &z; return *q;'
assert_inline 123 'struct S { int a; }; typedef struct S T; T s; s.a = 123; return s.a;'
assert_inline 77 'typedef struct { int b; } U; U u; u.b = 77; return u.b;'
assert_inline 88 'union U { int x; char c; }; typedef union U V; V v; v.x = 88; return v.x;'
assert_inline 55 'typedef int T; typedef T* TP; T x = 55; TP p = &x; return *p;'
assert_inline 66 'typedef int T; typedef T* TP; typedef TP* TPP; T x = 66; TP p = &x; TPP pp = &p; return **pp;'
assert_inline 99 'typedef struct { int v; } S; typedef S* SP; S s; s.v = 99; SP p = &s; return p->v;'
assert_inline 3 'typedef int T; T arr[3]; arr[0] = 1; arr[1] = 2; arr[2] = 3; return arr[2];'
assert_inline 42 'typedef int T; struct S { T a; }; struct S s; s.a = 42; return s.a;'

# =============================================================================
# エッジケース・複合テスト
# =============================================================================

# ゼロ除算回避（コンパイラの責任範囲外だが、テストとして記録）
# assert_inline 0 'return 1 / 0;'  # 未定義動作

# オーバーフロー
assert_inline 0 'char c = 255; c = c + 1; return c;'  # 8bit overflow -> 0
assert_inline 255 'char c = 0; c = c - 1; return c & 255;'  # underflow (mask to unsigned)

# ポインタ演算の境界
assert_inline 5 'int a[5]; int *p = a; p = p + 5; p = p - 5; a[0] = 5; return *p;'

# 複雑な式
assert_inline 1 'return 1 << 2 + 1 == 8;'
assert_inline 14 'return 2 + 3 * 4;'
assert_inline 0 'return 0 || 1 && 0;'

# 結合規則
assert_inline 0 'return 1 ^ 2 ^ 3;'
assert_inline 10 'return 100 / 5 / 2;'
assert_inline 6 'return 10 - 3 - 1;'

# 多重代入
assert_inline 3 'int a; int b; a = b = 3; return a;'

# 型変換の連鎖
assert_inline 1 'return (char)(short)(int)257;'

# 複雑なポインタ演算
assert_inline 6 'int a[3]; a[0] = 1; a[1] = 2; a[2] = 3; int *p = a; return *p + *(p+1) + *(p+2);'

# void型の使用
assert_inline 42 'void *p; int x = 42; p = &x; return *(int*)p;'

# constキーワード（C11準拠）
assert_inline 10 'const int x = 10; return x;'
# assert_inline 10 'const int x = 10; x = 20; return x;'  # エラーになるべき

# 前方宣言
assert_inline 5 'struct S; struct S { int x; }; struct S s; s.x = 5; return s.x;'

# 再帰的なstruct定義
assert_inline 10 'struct Node { int val; struct Node *next; }; struct Node n; n.val = 10; n.next = 0; return n.val;'

# =============================================================================
# エスケープシーケンス（C11準拠）
# =============================================================================
# 文字定数のエスケープシーケンス
assert_inline 10 "return '\n';"  # newline
assert_inline 9 "return '\t';"  # tab
assert_inline 13 "return '\r';"  # carriage return
assert_inline 0 "return '\0';"  # null
assert_inline 92 "return '\\\\';"  # backslash
assert_inline 39 "return '\'';"  # single quote
assert_inline 34 'return '\''"'\'';'  # double quote
assert_inline 7 "return '\a';"  # alert/bell
assert_inline 8 "return '\b';"  # backspace
assert_inline 12 "return '\f';"  # form feed
assert_inline 11 "return '\v';"  # vertical tab
assert_inline 63 "return '\?';"  # question mark

# 8進エスケープシーケンス
assert_inline 0 "return '\0';"  # \0
assert_inline 7 "return '\7';"  # \7
assert_inline 65 "return '\101';"  # \101 = 'A'
assert_inline 255 "return '\377';"  # \377 = 最大値

# 16進エスケープシーケンス
assert_inline 0 "return '\x00';"  # \x00
assert_inline 10 "return '\x0a';"  # \x0a = '\n'
assert_inline 65 "return '\x41';"  # \x41 = 'A'
assert_inline 255 "return '\xff';"  # \xff = 最大値
assert_inline 255 "return '\xFF';"  # 大文字も可

# 文字列リテラルのエスケープシーケンス
assert_inline 10 'char *s = "Hello\nWorld"; return s[5];'  # \n in string
assert_inline 9 'char *s = "Hello\tWorld"; return s[5];'  # \t in string
assert_inline 0 'char *s = "test\0end"; return s[4];'  # \0 in string
assert_inline 92 'char *s = "test\\end"; return s[4];'  # \\ in string
assert_inline 34 'char *s = "say \"hi\""; return s[4];'  # \" in string

# 複数のエスケープシーケンス
assert_inline 65 'char *s = "\x41\x42\x43"; return s[0];'  # ABC
assert_inline 66 'char *s = "\x41\x42\x43"; return s[1];'
assert_inline 67 'char *s = "\x41\x42\x43"; return s[2];'

# 8進と16進の混在
assert_inline 65 'char *s = "\101\x42\103"; return s[0];'  # A
assert_inline 66 'char *s = "\101\x42\103"; return s[1];'  # B
assert_inline 67 'char *s = "\101\x42\103"; return s[2];'  # C
