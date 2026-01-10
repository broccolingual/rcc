typedef const int cint;
typedef const int* cint_ptr;
typedef int* const int_cptr;

int main() {
    cint x = 10;
    cint y = 20;

    int z = 30;
    cint_ptr p1 = &z;

    int a = 40;
    int_cptr p2 = &a;

    return x + y;
}
