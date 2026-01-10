// Test 6: Union and enum
// Expected: 150

union Data {
    int i;
    char c;
    int arr[2];
};

struct Shape {
    int type;  // 0=circle, 1=rectangle
    union {
        struct { int radius; } circle;
        struct { int width; int height; } rect;
    } data;
};

int main() {
    int result;
    union Data d1;
    union Data d2;
    struct Shape s1;
    struct Shape s2;
    int area;
    union Data d3;
    int sum;
    union Data d4, d5;
    union Data d6;
    int saved;
    union Data arr[2];
    struct Shape shapes[2];
    int rect_area;

    result = 0;

    // Test 1: Basic union - int
    d1.i = 100;
    if (d1.i != 100) return 1;
    result = result + 15;

    // Test 2: Union - char (overwrites)
    d1.c = 65;  // 'A'
    if (d1.c != 65) return 2;
    result = result + 15;

    // Test 3: Union - array
    d2.arr[0] = 10;
    d2.arr[1] = 20;
    if (d2.arr[0] != 10 || d2.arr[1] != 20) return 3;
    result = result + 15;

    // Test 4: Anonymous union in struct - circle
    s1.type = 0;
    s1.data.circle.radius = 5;
    if (s1.data.circle.radius != 5) return 4;
    result = result + 15;

    // Test 5: Anonymous union in struct - rectangle
    s2.type = 1;
    s2.data.rect.width = 10;
    s2.data.rect.height = 20;
    area = s2.data.rect.width * s2.data.rect.height;
    if (area != 200) return 5;
    result = result + 15;

    // Test 6: Union size (largest member)
    d3.arr[0] = 50;
    d3.arr[1] = 100;
    sum = d3.arr[0] + d3.arr[1];
    if (sum != 150) return 6;
    result = result + 15;

    // Test 7: Multiple union variables
    d4.i = 111;
    d5.i = 222;
    if (d4.i + d5.i != 333) return 7;
    result = result + 15;

    // Test 8: Switching union types
    d6.i = 1000;
    saved = d6.i;
    d6.c = 50;
    d6.i = saved;
    if (d6.i != 1000) return 8;
    result = result + 15;

    // Test 9: Union in array
    arr[0].i = 10;
    arr[1].i = 20;
    if (arr[0].i + arr[1].i != 30) return 9;
    result = result + 15;

    // Test 10: Complex shape calculation
    shapes[0].type = 0;
    shapes[0].data.circle.radius = 3;  // Area would be ~28, but we'll just check radius
    shapes[1].type = 1;
    shapes[1].data.rect.width = 4;
    shapes[1].data.rect.height = 5;
    rect_area = shapes[1].data.rect.width * shapes[1].data.rect.height;
    if (rect_area != 20) return 10;
    result = result + 15;

    return result;  // Should be 150
}
