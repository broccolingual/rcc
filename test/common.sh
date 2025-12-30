#!/bin/bash

# テスト用の共通関数
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ビルド用のディレクトリを作成
setup_test() {
  mkdir -p $SCRIPT_DIR/bin
  cargo build
}

# インラインテスト用のassert関数
assert_inline() {
  expected="$1"
  input="$2"

  ./target/debug/rcc -i "int main() { $input }" > $SCRIPT_DIR/bin/tmp.s || {
    echo -e "\033[31m( ERROR )\033[0m Compilation failed: $input"
    return 1
  }

  cc -g -o $SCRIPT_DIR/bin/tmp $SCRIPT_DIR/bin/tmp.s || {
    echo -e "\033[31m( ERROR )\033[0m Linking failed: $input"
    return 1
  }

  set +e
  $SCRIPT_DIR/bin/tmp
  actual="$?"
  set -e

  if [ "$actual" = "$expected" ]; then
    echo -e "\033[32m( OK )\033[0m $input => $actual"
  else
    echo -e "\033[31m( NG )\033[0m $input => $expected expected, but got $actual"
    return 1
  fi
}

# 関数テスト用のassert関数
assert_func() {
  expected="$1"
  input="$2"

  ./target/debug/rcc -i "$input" > $SCRIPT_DIR/bin/tmp.s || {
    echo -e "\033[31m( ERROR )\033[0m Compilation failed: 
$input"
    return 1
  }

  cc -g -o $SCRIPT_DIR/bin/tmp $SCRIPT_DIR/bin/tmp.s $SCRIPT_DIR/bin/func.o || {
    echo -e "\033[31m( ERROR )\033[0m Linking failed: 
$input"
    return 1
  }
  
  set +e
  $SCRIPT_DIR/bin/tmp
  actual="$?"
  set -e

  if [ "$actual" = "$expected" ]; then
    echo -e "\033[32m( OK )\033[0m 
$input => $actual"
  else
    echo -e "\033[31m( NG )\033[0m 
$input => $expected expected, but got $actual"
    return 1
  fi
}
