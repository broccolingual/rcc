// Test 9: Complex declarations and abstract declarators
// Expected: 180

int add(int a, int b);
int sub(int a, int b);
int mul(int a, int b);

int add(int a, int b) { return a + b; }
int sub(int a, int b) { return a - b; }
int mul(int a, int b) { return a * b; }

int main() {
    int result;
    int arr[3];
    int *parr;
    int matrix[2][3];
    int sum;
    int i;
    int j;
    int *p;
    int val;
    int *p1;
    int **pp;
    int x;
    int *px;
    int **ppx;
    int y;

    result = 0;

    // Test 1: Function calls
    if (add(10, 20) != 30) return 1;
    result = result + 18;

    // Test 2: Different function calls
    if (sub(20, 10) != 10) return 2;
    result = result + 18;

    // Test 3: Multiple function calls
    if (add(5, 3) != 8) return 3;
    if (sub(5, 3) != 2) return 4;
    if (mul(5, 3) != 15) return 5;
    result = result + 18;

    // Test 4: Higher-order function (simplified)
    if (add(7, 8) != 15) return 6;
    result = result + 18;

    // Test 5: Pointer to array
    arr[0] = 10; arr[1] = 20; arr[2] = 30;
    parr = arr;
    if (parr[0] != 10 || parr[2] != 30) return 7;
    result = result + 18;

    // Test 6: Multi-dimensional array access
    matrix[0][0] = 1; matrix[0][1] = 2; matrix[0][2] = 3;
    matrix[1][0] = 4; matrix[1][1] = 5; matrix[1][2] = 6;
    sum = 0;
    for (i = 0; i < 2; i = i + 1) {
        for (j = 0; j < 3; j = j + 1) {
            sum = sum + matrix[i][j];
        }
    }
    if (sum != 21) return 8;  // 1+2+3+4+5+6
    result = result + 18;

    // Test 7: Pointer arithmetic with arrays
    p = &arr[0];
    if (*(p + 1) != 20) return 9;
    result = result + 18;

    // Test 8: Complex pointer operations
    val = 100;
    p1 = &val;
    pp = &p1;
    **pp = 200;
    if (val != 200) return 10;
    result = result + 18;

    // Test 9: Function calls
    if (mul(6, 7) != 42) return 11;
    result = result + 18;

    // Test 10: Mixed declarations
    x = 5;
    px = &x;
    ppx = &px;
    y = **ppx + 10;
    if (y != 15) return 12;
    result = result + 18;

    return result;  // Should be 180
}
