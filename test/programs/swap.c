// Swap array elements using pointers
void swap(int *a, int *b) {
    int temp;
    temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x;
    int y;
    x = 10;
    y = 20;
    swap(&x, &y);
    return x; // 20
}
