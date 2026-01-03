// 文字列の長さを求める（libc関数のextern宣言）
extern int strlen(char *s);

int main() {
    char *str = "Hello";
    
    return strlen(str); // 5
}
