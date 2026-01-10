#!/bin/bash

set -e

# 共通関数を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

setup_test

# Integrated test cases (10 comprehensive tests)
assert_file 100 "$SCRIPT_DIR/programs/01_basic_ops.c"
assert_file 110 "$SCRIPT_DIR/programs/02_functions.c"
assert_file 120 "$SCRIPT_DIR/programs/03_structs.c"
assert_file 130 "$SCRIPT_DIR/programs/04_pointers.c"
assert_file 140 "$SCRIPT_DIR/programs/05_typedef.c"
assert_file 150 "$SCRIPT_DIR/programs/06_union_enum.c"
assert_file 160 "$SCRIPT_DIR/programs/07_linked_list.c"
assert_file 170 "$SCRIPT_DIR/programs/08_trees.c"
assert_file 180 "$SCRIPT_DIR/programs/09_declarations.c"
assert_file 190 "$SCRIPT_DIR/programs/10_comprehensive.c"
