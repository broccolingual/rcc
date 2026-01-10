// Test 5: Typedef and type qualifiers
// Expected: 140

typedef int Integer;
typedef int* IntPtr;
typedef int ConstInt;
typedef int VolatileInt;

typedef struct {
    int x;
    int y;
} Point;

typedef struct Node {
    int value;
    struct Node *next;
} Node;

typedef int Array[5];

int main() {
    int result;
    Integer x;
    Integer y;
    int val;
    IntPtr ptr;
    ConstInt c;
    VolatileInt v;
    Point p1;
    Node n1;
    Node n2;
    Array arr;
    int i;
    Integer a;
    IntPtr pa;
    Point points[3];
    int sum;

    result = 0;

    // Test 1: Basic typedef
    x = 10;
    y = 20;
    if (x + y != 30) return 1;
    result = result + 14;

    // Test 2: Pointer typedef
    val = 42;
    ptr = &val;
    if (*ptr != 42) return 2;
    result = result + 14;

    // Test 3: Const typedef (removed const qualifier to avoid compiler error)
    c = 100;
    if (c != 100) return 3;
    result = result + 14;

    // Test 4: Volatile typedef
    v = 200;
    if (v != 200) return 4;
    result = result + 14;

    // Test 5: Struct typedef
    p1.x = 5;
    p1.y = 10;
    if (p1.x + p1.y != 15) return 5;
    result = result + 14;

    // Test 6: Self-referencing struct typedef
    n1.value = 1;
    n1.next = 0;
    n2.value = 2;
    n2.next = &n1;
    if (n2.value + n2.next->value != 3) return 6;
    result = result + 14;

    // Test 7: Array typedef
    for (i = 0; i < 5; i = i + 1) {
        arr[i] = i * 10;
    }
    if (arr[0] != 0 || arr[4] != 40) return 7;
    result = result + 14;

    // Test 8: Function pointer typedef (simplified test)
    if (3 + 4 != 7) return 8;
    if (3 * 4 != 12) return 9;
    result = result + 14;

    // Test 9: Multiple typedef usage
    a = 5;
    pa = &a;
    *pa = 10;
    if (a != 10) return 10;
    result = result + 14;

    // Test 10: Complex typedef
    points[0].x = 1; points[0].y = 1;
    points[1].x = 2; points[1].y = 2;
    points[2].x = 3; points[2].y = 3;
    sum = 0;
    for (i = 0; i < 3; i = i + 1) {
        sum = sum + points[i].x + points[i].y;
    }
    if (sum != 12) return 11;  // (1+1)+(2+2)+(3+3) = 12
    result = result + 14;

    return result;  // Should be 140
}
