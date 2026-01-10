// Test 2: Functions and recursion
// Expected: 110

int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int gcd(int a, int b) {
    if (b == 0) return a;
    return gcd(b, a % b);
}

int power(int base, int exp) {
    if (exp == 0) return 1;
    return base * power(base, exp - 1);
}

int sum_to_n(int n) {
    if (n <= 0) return 0;
    return n + sum_to_n(n - 1);
}

int is_even(int n);
int is_odd(int n);

int is_even(int n) {
    if (n == 0) return 1;
    return is_odd(n - 1);
}

int is_odd(int n) {
    if (n == 0) return 0;
    return is_even(n - 1);
}

int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
int mul(int a, int b) { return a * b; }

int main() {
    int result = 0;

    // Test 1: Factorial (5! = 120)
    if (factorial(5) != 120) return 1;
    result = result + 11;

    // Test 2: Fibonacci (fib(10) = 55)
    if (fibonacci(10) != 55) return 2;
    result = result + 11;

    // Test 3: GCD (gcd(48, 18) = 6)
    if (gcd(48, 18) != 6) return 3;
    result = result + 11;

    // Test 4: Power (2^6 = 64)
    if (power(2, 6) != 64) return 4;
    result = result + 11;

    // Test 5: Sum to N (sum(10) = 55)
    if (sum_to_n(10) != 55) return 5;
    result = result + 11;

    // Test 6: Mutual recursion (is_even(4) = 1, is_odd(3) = 1)
    if (is_even(4) != 1 || is_odd(3) != 1) return 6;
    result = result + 11;

    // Test 7: Multiple parameters
    if (add(10, 20) != 30) return 7;
    result = result + 11;

    // Test 8: Function calls in expressions
    int val = mul(add(3, 4), sub(10, 5));  // (3+4) * (10-5) = 35
    if (val != 35) return 8;
    result = result + 11;

    // Test 9: Nested function calls
    if (factorial(gcd(12, 8)) != 24) return 9;  // gcd(12,8)=4, 4!=24
    result = result + 11;

    // Test 10: Deep recursion
    if (sum_to_n(20) != 210) return 10;  // 1+2+...+20 = 210
    result = result + 11;

    return result;  // Should be 110
}
