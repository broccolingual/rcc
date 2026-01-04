// Find greatest common divisor using Euclidean algorithm
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
