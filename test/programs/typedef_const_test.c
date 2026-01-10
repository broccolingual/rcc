// Test typedef with const qualifier preservation

typedef const int cint;
typedef volatile int vint;
typedef const volatile int cvint;

typedef int const int_c;
typedef int* const int_ptr_c;
typedef const int* const_int_ptr;

int main() {
    // Test basic const int
    cint a = 10;

    // Test volatile int
    vint b = 20;

    // Test const volatile int
    cvint c = 30;

    // Test int const
    int_c d = 40;

    // Test const int pointer
    int x = 50;
    const_int_ptr p1 = &x;

    // Test int* const
    int y = 60;
    int_ptr_c p2 = &y;

    return a + b + c + d + *p1 + *p2;
}
