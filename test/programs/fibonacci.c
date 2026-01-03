// フィボナッチ数列のn番目を求める（再帰）
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    return fibonacci(10); // 55
}
