// Test 4: Pointers and arrays
// Expected: 130

int my_strlen(char *s) {
    int len = 0;
    while (*s != 0) {
        len = len + 1;
        s = s + 1;
    }
    return len;
}

void swap_ptr(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int result;
    int x;
    int *p;
    int arr[5];
    int *ptr;
    int sum;
    int *p2;
    int i;
    char str[6];
    int a;
    int b;
    int val;
    int *p1;
    int **pp;
    int v1;
    int v2;
    int v3;
    int *ptrs[3];
    int s;
    int nums[3];
    int *np;
    int total;
    int data[3];
    int *dp;
    int *null_ptr;

    result = 0;

    // Test 1: Basic pointer operations
    x = 42;
    p = &x;
    if (*p != 42) return 1;
    *p = 50;
    if (x != 50) return 2;
    result = result + 13;

    // Test 2: Pointer arithmetic
    arr[0] = 10; arr[1] = 20; arr[2] = 30; arr[3] = 40; arr[4] = 50;
    ptr = arr;
    if (*ptr != 10) return 3;
    ptr = ptr + 2;
    if (*ptr != 30) return 4;
    result = result + 13;

    // Test 3: Array access via pointer
    sum = 0;
    p2 = arr;
    for (i = 0; i < 5; i = i + 1) {
        sum = sum + *(p2 + i);
    }
    if (sum != 150) return 5;  // 10+20+30+40+50 = 150
    result = result + 13;

    // Test 4: String operations (strlen)
    str[0] = 'h'; str[1] = 'e'; str[2] = 'l'; str[3] = 'l'; str[4] = 'o'; str[5] = 0;
    if (my_strlen(str) != 5) return 6;
    result = result + 13;

    // Test 5: Swap via pointers
    a = 100;
    b = 200;
    swap_ptr(&a, &b);
    if (a != 200 || b != 100) return 7;
    result = result + 13;

    // Test 6: Pointer to pointer
    val = 99;
    p1 = &val;
    pp = &p1;
    if (**pp != 99) return 8;
    result = result + 13;

    // Test 7: Array of pointers
    v1 = 1;
    v2 = 2;
    v3 = 3;
    ptrs[0] = &v1;
    ptrs[1] = &v2;
    ptrs[2] = &v3;
    s = 0;
    for (i = 0; i < 3; i = i + 1) {
        s = s + *ptrs[i];
    }
    if (s != 6) return 9;
    result = result + 13;

    // Test 8: Pointer increment and dereference
    nums[0] = 5; nums[1] = 10; nums[2] = 15;
    np = nums;
    total = 0;
    total = total + *np;  // 5
    np = np + 1;
    total = total + *np;  // 10
    np = np + 1;
    total = total + *np;  // 15
    if (total != 30) return 10;
    result = result + 13;

    // Test 9: Modifying array via pointer
    data[0] = 1; data[1] = 2; data[2] = 3;
    dp = data;
    for (i = 0; i < 3; i = i + 1) {
        *(dp + i) = *(dp + i) * 2;
    }
    if (data[0] != 2 || data[1] != 4 || data[2] != 6) return 11;
    result = result + 13;

    // Test 10: Null pointer comparison
    null_ptr = 0;
    if (null_ptr == 0) {
        result = result + 13;
    } else {
        return 12;
    }

    return result;  // Should be 130
}
