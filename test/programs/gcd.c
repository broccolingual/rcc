// ユークリッドの互除法で最大公約数を求める
int gcd(int a, int b) {
    while (b != 0) {
        int temp;
        temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

int main() {
    return gcd(48, 18); // 6
}
