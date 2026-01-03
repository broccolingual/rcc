#!/bin/bash

set -e

# テストスクリプトを実行
echo "=== Running inline tests ==="
./test/inline.sh

echo ""
echo "=== Running function tests ==="
./test/func.sh

echo ""
echo "=== Running file tests ==="
./test/file.sh

echo ""
echo "=== All tests passed! ==="
