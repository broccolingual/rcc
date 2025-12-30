#!/bin/bash

set -e

# 共通関数を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

setup_test

# 基本的なリテラル値のテスト
assert_inline 0 'return 0;'  # ゼロ
assert_inline 42 'return 42;'  # 正の整数
assert_inline 255 'return 255;'  # 8ビット最大値

# 算術演算子の優先順位と基本演算のテスト
assert_inline 21 'return 5 + 2 * 8;'  # 乗算が先に評価される
assert_inline 47 'return 5 + 6 * 7;'  # 乗算が先に評価される
assert_inline 15 'return 5 * (9 - 6);'  # 括弧による優先順位の変更
assert_inline 4 'return (3 + 5) / 2;'  # 括弧と除算
assert_inline 2 'return 8 % 3;'  # 剰余演算
assert_inline 0 'return (3 + 5) % 4;'  # 括弧と剰余演算
assert_inline 35 'return 5 + 6 * 7 - 12;'  # 複数の演算子
assert_inline 8 'return 2 + 3 * 2;'  # 乗算優先

# ゼロとの演算や特殊ケースのテスト
assert_inline 0 'return 0 * 42;'  # ゼロとの乗算
assert_inline 0 'return 42 * 0;'  # ゼロとの乗算（逆順）
assert_inline 42 'return 42 / 1;'  # 1での除算
assert_inline 1 'return 42 / 42;'  # 自分自身での除算
assert_inline 0 'return 0 % 5;'  # ゼロの剰余
assert_inline 1 'return 1 % 2;'  # 小さい値の剰余
assert_inline 10 'return 20 / 2;'  # 偶数の除算
assert_inline 3 'return 10 / 3;'  # 切り捨て除算

# 単項演算子のテスト
assert_inline 10 'return +10;'  # 単項プラス
assert_inline 245 'return -10 + 255;'  # 単項マイナス (8ビット範囲)
assert_inline 10 'return -(-10);'  # 二重否定
assert_inline 10 'return - - +10;'  # 複数の単項演算子
assert_inline 254 'return -1 + 255;'  # -1の8ビット表現

# ビット演算子のテスト
assert_inline 7 'return 3 | 5;'  # ビットOR (0b011 | 0b101 = 0b111)
assert_inline 1 'return 3 & 5;'  # ビットAND (0b011 & 0b101 = 0b001)
assert_inline 6 'return 3 ^ 5;'  # ビットXOR (0b011 ^ 0b101 = 0b110)
assert_inline 32 'return 1 << 5;'  # 左シフト (1 << 5 = 32)
assert_inline 2 'return 8 >> 2;'  # 右シフト (8 >> 2 = 2)
assert_inline 245 'return ~10 & 255;'  # ビット反転とマスク
assert_inline 0 'return 5 & 0;'  # ゼロとのAND
assert_inline 5 'return 5 | 0;'  # ゼロとのOR
assert_inline 0 'return 5 ^ 5;'  # 同じ値とのXOR
assert_inline 1 'return 1 << 0;'  # ゼロシフト
assert_inline 8 'return 8 >> 0;'  # ゼロシフト
assert_inline 12 'return 3 << 2;'  # 左シフト (3 << 2 = 12)
assert_inline 15 'return 15 & 15;'  # 自分自身とのAND

# 比較演算子のテスト
assert_inline 0 'return 0 == 1;'  # 等価（偽）
assert_inline 1 'return 42 == 42;'  # 等価（真）
assert_inline 1 'return 0 != 1;'  # 不等価（真）
assert_inline 0 'return 42 != 42;'  # 不等価（偽）
assert_inline 1 'return 0 < 1;'  # 小なり（真）
assert_inline 0 'return 1 < 1;'  # 小なり（偽）
assert_inline 0 'return 2 < 1;'  # 小なり（偽）
assert_inline 1 'return 0 <= 1;'  # 以下（真）
assert_inline 1 'return 1 <= 1;'  # 以下（真、等しい）
assert_inline 0 'return 2 <= 1;'  # 以下（偽）
assert_inline 1 'return 1 > 0;'  # 大なり（真）
assert_inline 0 'return 1 > 1;'  # 大なり（偽）
assert_inline 0 'return 1 > 2;'  # 大なり（偽）
assert_inline 1 'return 1 >= 0;'  # 以上（真）
assert_inline 1 'return 1 >= 1;'  # 以上（真、等しい）
assert_inline 0 'return 1 >= 2;'  # 以上（偽）

# 論理演算子のテスト
assert_inline 1 'return 1 && 1;'  # 論理AND（真）
assert_inline 0 'return 1 && 0;'  # 論理AND（偽）
assert_inline 0 'return 0 && 1;'  # 論理AND（偽）
assert_inline 0 'return 0 && 0;'  # 論理AND（偽）
assert_inline 1 'return 1 || 0;'  # 論理OR（真）
assert_inline 1 'return 0 || 1;'  # 論理OR（真）
assert_inline 1 'return 1 || 1;'  # 論理OR（真）
assert_inline 0 'return 0 || 0;'  # 論理OR（偽）
assert_inline 0 'return !1;'  # 論理NOT（偽）
assert_inline 1 'return !0;'  # 論理NOT（真）
assert_inline 1 'return !!1;'  # 二重否定（真）
assert_inline 0 'return !!0;'  # 二重否定（偽）
assert_inline 1 'return 2 && 3;'  # 非ゼロ値の論理AND
assert_inline 1 'return 5 || 0;'  # 非ゼロ値の論理OR

# 短絡評価のテスト
assert_inline 1 'int x; x = 1; 0 && (x = 5); return x;'  # 左辺が偽なので右辺は評価されない
assert_inline 1 'int x; x = 1; 1 || (x = 5); return x;'  # 左辺が真なので右辺は評価されない
assert_inline 1 'int x; x = 0; 0 || (x = 1); return x;'  # 右辺まで評価される
assert_inline 5 'int x; x = 1; 1 && (x = 5); return x;'  # 左辺が真なので右辺も評価される

# 演算子優先順位のテスト
assert_inline 8 'return 1 << 2 + 1;'  # 加算が先（1 << 3 = 8）
assert_inline 5 'return (1 << 2) + 1;'  # シフトが先（4 + 1 = 5）
assert_inline 14 'return 2 + 3 * 4;'  # 乗算が先
assert_inline 20 'return (2 + 3) * 4;'  # 括弧で加算を先に
assert_inline 0 'return 0 || 1 && 0;'  # ANDがORより優先
assert_inline 1 'return (0 || 1) && 1;'  # 括弧でORを先に
assert_inline 1 'return 1 | 2 & 4;'  # ANDがORより優先
assert_inline 2 'return (1 | 2) & 6;'  # 括弧でORを先に

# 結合規則のテスト
assert_inline 0 'return 1 ^ 2 ^ 3;'  # 左結合（(1 ^ 2) ^ 3 = 0）
assert_inline 1 'return 1 < 2 < 3;'  # 左結合（(1 < 2) < 3 = 1 < 3 = 1）
assert_inline 10 'return 100 / 5 / 2;'  # 左結合（(100 / 5) / 2 = 10）
assert_inline 50 'return 100 / (5 / 2);'  # 右結合を括弧で強制（100 / 2 = 50）
assert_inline 6 'return 10 - 3 - 1;'  # 左結合（(10 - 3) - 1 = 6）
assert_inline 8 'return 10 - (3 - 1);'  # 括弧で右結合（10 - 2 = 8）

# 変数宣言と代入のテスト
assert_inline 3 'int a; a = 3; return a;'  # 基本的な代入
assert_inline 13 'int a; int b; a = 3; b = 10; return a + b;'  # 複数変数の代入
assert_inline 6 'int a; int b; int c; a = 1; b = 2; c = 3; return a + b + c;'  # 3つの変数
assert_inline 5 'int a = 5; return a;'  # 初期化子付き宣言

# 複合代入演算子のテスト
assert_inline 7 'int a; a = 3; a += 4; return a;'  # 加算代入
assert_inline 2 'int b; b = 5; b -= 3; return b;'  # 減算代入
assert_inline 15 'int c; c = 3; c *= 5; return c;'  # 乗算代入
assert_inline 4 'int d; d = 20; d /= 5; return d;'  # 除算代入
assert_inline 3 'int e; e = 3; e %= 4; return e;'  # 剰余代入
assert_inline 7 'int f; f = 3; f |= 5; return f;'  # ビットOR代入
assert_inline 1 'int g; g = 3; g &= 5; return g;'  # ビットAND代入
assert_inline 6 'int h; h = 3; h ^= 5; return h;'  # ビットXOR代入
assert_inline 16 'int i; i = 1; i <<= 4; return i;'  # 左シフト代入
assert_inline 2 'int j; j = 8; j >>= 2; return j;'  # 右シフト代入
assert_inline 3 'int a; int b; a = b = 3; return a;'  # 連鎖代入（右結合）
assert_inline 5 'int a = 2; a += 3; return a;'  # 初期化子と複合代入

# 前置・後置インクリメント/デクリメントのテスト
assert_inline 6 'int a; a = 5; return ++a;'  # 前置インクリメント（先に加算）
assert_inline 5 'int a; a = 5; return a++;'  # 後置インクリメント（後で加算）
assert_inline 4 'int b; b = 5; return --b;'  # 前置デクリメント（先に減算）
assert_inline 5 'int b; b = 5; return b--;'  # 後置デクリメント（後で減算）
assert_inline 6 'int a; a = 5; ++a; return a;'  # 前置インクリメントの副作用
assert_inline 6 'int a; a = 5; a++; return a;'  # 後置インクリメントの副作用
assert_inline 4 'int a; a = 5; --a; return a;'  # 前置デクリメントの副作用
assert_inline 4 'int a; a = 5; a--; return a;'  # 後置デクリメントの副作用
assert_inline 11 'int a = 5; return a++ + a;'  # 後置インクリメントと式
assert_inline 12 'int a = 5; return ++a + a;'  # 前置インクリメントと式

# if文のテスト
assert_inline 3 'if (1) return 3; return 4;'  # 条件が真
assert_inline 4 'if (0) return 3; return 4;'  # 条件が偽
assert_inline 3 'if (1) return 3; else return 4;'  # else節（条件が真）
assert_inline 4 'if (0) return 3; else return 4;'  # else節（条件が偽）
assert_inline 2 'if (0) return 1; else if (1) return 2; else return 3;'  # else if
assert_inline 3 'if (0) return 1; else if (0) return 2; else return 3;'  # 全て偽
assert_inline 5 'int a = 5; if (a > 3) return a; return 0;'  # 変数を使った条件
assert_inline 10 'int a = 5; if (a < 3) a = 0; else a = 10; return a;'  # 変数の再代入

# while文のテスト
assert_inline 10 'int i; i = 0; while (i < 10) i = i + 1; return i;'  # 基本的なループ
assert_inline 0 'int i; i = 0; while (0) i = i + 1; return i;'  # 条件が偽（ループしない）
assert_inline 3 'int i; i = 0; while (1) { i = i + 1; if (i == 3) break; } return i;'  # breakでループ脱出
assert_inline 15 'int i = 1; int s = 0; while (i <= 5) { s = s + i; i++; } return s;'  # 累積計算

# do-while文のテスト
assert_inline 1 'int i; i = 0; do i = i + 1; while (i < 1); return i;'  # 1回だけ実行
assert_inline 5 'int i; i = 0; do i = i + 1; while (i < 5); return i;'  # 複数回実行
assert_inline 1 'int i = 0; do i++; while (0); return i;'  # 条件が偽でも1回は実行

# for文のテスト
assert_inline 55 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) sum = sum + i; return sum;'  # 1から10の合計
assert_inline 15 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i > 5) break; sum = sum + i; } return sum;'  # breakでループ脱出
assert_inline 25 'int sum; int i; sum = 0; for (i = 1; i <= 10; i = i + 1) { if (i % 2 == 0) continue; sum = sum + i; } return sum;'  # continueで偶数をスキップ
assert_inline 0 'int i; for (i = 0; 0; i = i + 1) i = i + 1; return i;'  # 条件が偽（ループしない）
assert_inline 10 'int i; for (i = 0; i < 10; i++); return i;'  # 空のループ本体
assert_inline 5 'int i; int j = 0; for (i = 0; i < 5; i = i + 1) j = j + 1; return j;'  # ネストなしのカウント

# 三項演算子のテスト
assert_inline 2 'return 1 ? 2 : 3;'  # 条件が真
assert_inline 3 'return 0 ? 2 : 3;'  # 条件が偽
assert_inline 3 'int a; int b; a = 3; b = 5; return a < b ? a : b;'  # 小さい方を返す
assert_inline 5 'int a; int b; a = 3; b = 5; return a > b ? a : b;'  # 大きい方を返す
assert_inline 10 'int x = 5; return x > 0 ? x * 2 : x;'  # 式を含む三項演算子
assert_inline 5 'int a = 1; int b = 2; int c = 3; return b < c ? b + c : b - c;'  # 複雑な式

# ブロック文とスコープのテスト
assert_inline 8 '{ int a; int b; a = 3; b = 5; return a + b; }'  # 基本的なブロック
assert_inline 3 '{ { { return 3; } } }'  # ネストしたブロック
assert_inline 2 '{ int a; a = 2; { int a; a = 3; a = a + 2; } return a; }'  # 内側のスコープで変数を隠蔽
assert_inline 5 'int a = 1; { a = 5; } return a;'  # ブロック内で外側の変数を変更
assert_inline 1 'int a = 1; { int a = 2; } return a;'  # 内側で同名の変数を宣言

# sizeof演算子のテスト
assert_inline 4 'return sizeof(int);'  # int型のサイズ
assert_inline 8 'return sizeof(int *);'  # ポインタのサイズ
assert_inline 4 'int x; return sizeof(x);'  # 変数のサイズ
assert_inline 8 'int *p; return sizeof(p);'  # ポインタ変数のサイズ
assert_inline 4 'return sizeof 1;'  # リテラルのサイズ（括弧なし）
assert_inline 32 'int a[8]; return sizeof(a);'  # 配列のサイズ（8 * 4 = 32）
assert_inline 1 'return sizeof(char);'  # char型のサイズ
assert_inline 2 'return sizeof(short);'  # short型のサイズ
assert_inline 8 'return sizeof(long);'  # long型のサイズ
assert_inline 16 'int a[2][2]; return sizeof(a);'  # 2次元配列のサイズ

# ポインタの基本操作のテスト
assert_inline 3 'int a; int *b; a = 3; b = &a; return *b;'  # アドレス取得と参照外し
assert_inline 7 'int a; int *p; p = &a; *p = 7; return a;'  # ポインタ経由での代入
assert_inline 5 'int a; int *p; int **pp; a = 5; p = &a; pp = &p; return **pp;'  # 二重ポインタ
assert_inline 10 'int a = 10; int *p = &a; return *p;'  # 初期化子付きポインタ
assert_inline 15 'int a = 5; int *p = &a; *p = *p + 10; return a;'  # ポインタ経由での演算

# 配列の基本操作のテスト
assert_inline 3 'int a[5]; a[0] = 3; return a[0];'  # 配列の要素へのアクセス
assert_inline 8 'int a[5]; a[0] = 3; a[1] = 5; return a[0] + a[1];'  # 複数要素
assert_inline 2 'int a[3]; *(a + 1) = 2; return a[1];'  # ポインタ演算でのアクセス
assert_inline 1 'int a[3]; int *p; p = a; a[1] = 1; return *(++p);'  # ポインタインクリメント
assert_inline 1 'int a[2][3]; a[1][2] = 1; return a[1][2];'  # 2次元配列
assert_inline 5 'int a[2][3]; *(*(a + 1) + 2) = 5; return a[1][2];'  # 2次元配列のポインタアクセス
assert_inline 2 'int i = 2; int a[2][3]; a[1][i - 1] = 2; return a[1][1];'  # 変数によるインデックス
assert_inline 2 'int a[2][3]; int *p; p = a[1]; a[1][1] = 2; return *(++p);'  # 2次元配列の行へのポインタ
assert_inline 10 'int a[5]; a[2] = 10; return *(a + 2);'  # 配列とポインタの等価性

# gotoとラベルのテスト
assert_inline 5 'int a; a = 0; goto skip; a = 10; skip: a = a + 5; return a;'  # 基本的なgoto
assert_inline 3 'int i = 0; loop: i = i + 1; if (i < 3) goto loop; return i;'  # ループをgotoで実装
assert_inline 10 'int a = 5; goto end; a = 0; end: a = a + 5; return a;'  # スキップしてラベルへ

# 数値リテラルのテスト
assert_inline 10 'return 012;'  # 8進数リテラル
assert_inline 26 'return 0x1a;'  # 16進数リテラル（小文字）
assert_inline 255 'return 0xff;'  # 16進数リテラル（最大値）
assert_inline 255 'return 0xFF;'  # 16進数リテラル（大文字）
assert_inline 0 'return 0x0;'  # 16進数のゼロ
assert_inline 0 'return 00;'  # 8進数のゼロ

# 配列の初期化子のテスト
assert_inline 6 'int a[3] = {1, 2, 3}; return a[0] + a[1] + a[2];'  # 完全な初期化
assert_inline 3 'int a[3] = {1, 2}; return a[0] + a[1] + a[2];'  # 部分的な初期化（残りは0）
assert_inline 1 'int a[3] = {1}; return a[0] + a[1] + a[2];'  # 最初の要素のみ初期化
assert_inline 3 'int a[3] = {0, 1, 2, 3}; return a[0] + a[1] + a[2];'  # 初期化子が多すぎる場合
assert_inline 6 'int x = 1; int y = 2; int z = 3; int *a[3] = {&x, &y, &z}; return *a[0] + *a[1] + *a[2];'  # ポインタ配列の初期化
assert_inline 15 'int a[] = {1, 2, 3, 4, 5}; return a[0] + a[1] + a[2] + a[3] + a[4];'  # サイズを省略した初期化
assert_inline 0 'int a[5] = {0}; return a[0] + a[1] + a[2] + a[3] + a[4];'  # 全て0で初期化
# assert_inline 21 'int a[2][3] = {{1,2,3}, {4,5,6}}; return a[0][0] + a[0][1] + a[0][2] + a[1][0] + a[1][1] + a[1][2];'  # 2次元配列の初期化（未実装）

# 構造体の基本操作のテスト
assert_inline 8 'struct {int a; short b; int c;} s; s.a = 3; s.c = 5; return s.a + s.c;'  # 匿名構造体
assert_inline 3 'struct {struct {int a; int b;} inner; int x;} outer; outer.inner.b = 3; return outer.inner.b;'  # ネストした構造体
assert_inline 3 'struct Point {int x; int y;}; struct Point p; p.x = 1; p.y = 2; return p.x + p.y;'  # 名前付き構造体
assert_inline 10 'struct {int x; int y; int z;} s; s.x = 1; s.y = 4; s.z = 5; return s.x + s.y + s.z;'  # 3つのフィールド

# 構造体ポインタとアロー演算子のテスト
assert_inline 30 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; return ptr->x + ptr->y;'  # アロー演算子で両フィールドにアクセス
assert_inline 10 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; return ptr->x;'  # アロー演算子でxにアクセス
assert_inline 20 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; return ptr->y;'  # アロー演算子でyにアクセス
assert_inline 70 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; ptr->x = 30; ptr->y = 40; return ptr->x + ptr->y;'  # アロー演算子で書き込み
assert_inline 30 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 10; p.y = 20; ptr = &p; ptr->x = 30; return p.x;'  # ポインタ経由での書き込みが元の変数に反映
assert_inline 15 'struct Point {int x; int y;}; struct Point p; struct Point *ptr; p.x = 5; p.y = 10; ptr = &p; return (*ptr).x + (*ptr).y;'  # ドット演算子での参照外し

echo OK
