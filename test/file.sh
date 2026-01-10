#!/bin/bash

set -e

# 共通関数を読み込む
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/common.sh"

setup_test

# テストケース
assert_file 120 "$SCRIPT_DIR/programs/factorial.c"
assert_file 6 "$SCRIPT_DIR/programs/gcd.c"
assert_file 20 "$SCRIPT_DIR/programs/swap.c"
assert_file 1 "$SCRIPT_DIR/programs/prime.c"
assert_file 55 "$SCRIPT_DIR/programs/fibonacci.c"
assert_file 64 "$SCRIPT_DIR/programs/power.c"
assert_file 5 "$SCRIPT_DIR/programs/bubble_sort.c"
assert_file 4 "$SCRIPT_DIR/programs/binary_search.c"
assert_file 5 "$SCRIPT_DIR/programs/strlen_test.c"
assert_file 25 "$SCRIPT_DIR/programs/struct_point.c"
assert_file 88 "$SCRIPT_DIR/programs/struct_students.c"
assert_file 200 "$SCRIPT_DIR/programs/struct_rectangle.c"
assert_file 60 "$SCRIPT_DIR/programs/linked_list.c"
assert_file 20 "$SCRIPT_DIR/programs/binary_tree.c"
assert_file 3 "$SCRIPT_DIR/programs/forward_declaration.c"
assert_file 90 "$SCRIPT_DIR/programs/chained_arrow.c"
assert_file 15 "$SCRIPT_DIR/programs/bst.c"
assert_file 36 "$SCRIPT_DIR/programs/shape_calculator.c"
assert_file 250 "$SCRIPT_DIR/programs/employee_system.c"
assert_file 0 "$SCRIPT_DIR/programs/abstract_declarator.c"
assert_file 154 "$SCRIPT_DIR/programs/union_struct.c"
assert_file 77 "$SCRIPT_DIR/programs/typedef_graph.c"
assert_file 133 "$SCRIPT_DIR/programs/typedef_stack_queue.c"
assert_file 198 "$SCRIPT_DIR/programs/typedef_complex_types.c"
