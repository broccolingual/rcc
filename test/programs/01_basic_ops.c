// Test 1: Basic operations and control flow
// Expected: 100

int main() {
    int result, a, b, temp, base, exp, power_result, i, j;
    int n, is_prime, x, y, tmp;
    int arr[5];
    int sarr[5];
    int target, left, right, found, mid;
    int max, min, val1, val2, bit_test, nested_sum;
    
    result = 0;

    // Test 1: Arithmetic operations (GCD)
    a = 48; b = 18;
    while (b != 0) {
        temp = b;
        b = a % b;
        a = temp;
    }
    if (a != 6) return 1;
    result = result + 10;

    // Test 2: Power calculation (2^6 = 64)
    base = 2; exp = 6; power_result = 1;
    for (i = 0; i < exp; i = i + 1) {
        power_result = power_result * base;
    }
    if (power_result != 64) return 2;
    result = result + 10;

    // Test 3: Prime check (7 is prime)
    n = 7; is_prime = 1;
    if (n <= 1) {
        is_prime = 0;
    } else {
        for (i = 2; i * i <= n; i = i + 1) {
            if (n % i == 0) {
                is_prime = 0;
            }
        }
    }
    if (is_prime != 1) return 3;
    result = result + 10;

    // Test 4: Swap
    x = 10; y = 20;
    tmp = x;
    x = y;
    y = tmp;
    if (x != 20 || y != 10) return 4;
    result = result + 10;

    // Test 5: Bubble sort
    arr[0] = 5; arr[1] = 2; arr[2] = 8; arr[3] = 1; arr[4] = 9;
    for (i = 0; i < 5; i = i + 1) {
        for (j = 0; j < 4 - i; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
        }
    }
    if (arr[0] != 1 || arr[4] != 9) return 5;
    result = result + 10;

    // Test 6: Binary search
    sarr[0] = 1; sarr[1] = 3; sarr[2] = 5; sarr[3] = 7; sarr[4] = 9;
    target = 5; left = 0; right = 4; found = -1;
    while (left <= right) {
        mid = (left + right) / 2;
        if (sarr[mid] == target) {
            found = mid;
            break;
        } else if (sarr[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    if (found != 2) return 6;
    result = result + 10;

    // Test 7: Conditional expressions
    max = (15 > 10) ? 15 : 10;
    min = (5 < 3) ? 5 : 3;
    if (max != 15 || min != 3) return 7;
    result = result + 10;

    // Test 8: Logical operations
    val1 = 5; val2 = 10;
    if ((val1 < val2) && (val2 > 0)) {
        result = result + 10;
    } else {
        return 8;
    }

    // Test 9: Bitwise operations
    bit_test = 5 & 3;
    if (bit_test == 1) {
        result = result + 10;
    } else {
        return 9;
    }

    // Test 10: Nested loops
    nested_sum = 0;
    for (i = 1; i <= 3; i = i + 1) {
        for (j = 1; j <= 3; j = j + 1) {
            nested_sum = nested_sum + 1;
        }
    }
    if (nested_sum == 9) {
        result = result + 10;
    } else {
        return 10;
    }

    return result;
}
