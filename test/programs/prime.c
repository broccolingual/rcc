// Check if a number is prime (recursive)
int is_prime_helper(int n, int divisor) {
    if (divisor * divisor > n) {
        return 1;
    }
    if (n % divisor == 0) {
        return 0;
    }
    return is_prime_helper(n, divisor + 2);
}

int is_prime(int n) {
    if (n <= 1) {
        return 0;
    }
    if (n == 2) {
        return 1;
    }
    if (n % 2 == 0) {
        return 0;
    }
    return is_prime_helper(n, 3);
}

int main() {
    return is_prime(17); // 1 (true)
}
