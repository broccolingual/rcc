// Test for abstract declarators in function prototypes
int add(int, int);
int sub(int, int);
int mul(int, int);
int ptr_add(int *, int);
int ptr_deref(int *);
void modify_value(int *, int);
int sum_array(int *, int);

int add(int a, int b) {
    return a + b;
}

int sub(int x, int y) {
    return x - y;
}

int mul(int a, int b) {
    return a * b;
}

int ptr_add(int *p, int value) {
    return *p + value;
}

int ptr_deref(int *ptr) {
    return *ptr;
}

void modify_value(int *p, int val) {
    *p = val;
}

int sum_array(int *arr, int size) {
    int sum = 0;
    int i;
    for (i = 0; i < size; i++) {
        sum = sum + arr[i];
    }
    return sum;
}

int main() {
    int x = 10;
    int y = 20;
    int z = 0;
    int arr[5];
    int i;
    
    // 基本的な抽象宣言子のテスト
    if (add(5, 3) != 8) return 1;
    if (sub(10, 4) != 6) return 2;
    if (mul(7, 6) != 42) return 3;
    
    // ポインタの抽象宣言子のテスト
    if (ptr_add(&x, 5) != 15) return 4;
    if (ptr_deref(&y) != 20) return 5;
    
    modify_value(&z, 100);
    if (z != 100) return 6;
    
    // 配列ポインタの抽象宣言子のテスト
    for (i = 0; i < 5; i++) {
        arr[i] = i + 1;
    }
    if (sum_array(arr, 5) != 15) return 7;
    
    return 0;
}
