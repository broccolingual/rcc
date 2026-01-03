// べき乗計算
int power(int base, int exp) {
    int result;
    int i;
    result = 1;
    i = 0;
    while (i < exp) {
        result = result * base;
        i = i + 1;
    }
    return result;
}

int main() {
    return power(2, 6); // 2^6 = 64
}
