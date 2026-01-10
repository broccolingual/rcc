// Verify const qualifier is preserved through typedef

typedef const int cint;

int main() {
    // This should work: const int can be initialized
    cint x = 10;

    // This should also work: another const int
    cint y = 20;

    return x + y;
}
