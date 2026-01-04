// Calculate string length (extern declaration for libc function)
extern int strlen(char *s);

int main() {
    char *str = "Hello";
    
    return strlen(str); // 5
}
