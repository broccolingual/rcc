// Test 3: Structs
// Expected: 120

struct Point {
    int x;
    int y;
};

struct Rectangle {
    struct Point top_left;
    struct Point bottom_right;
};

struct Student {
    int id;
    int age;
    int score;
};

struct Employee {
    int id;
    int salary;
    int years;
};

int main() {
    struct Point p1;
    struct Rectangle rect;
    struct Student s1;
    struct Student s2;
    struct Point p2;
    struct Point points[3];
    struct Employee emp;
    struct Point p3;
    struct Point p4;
    struct Rectangle r2;
    int result;
    int width;
    int height;
    int area;
    int avg;
    int sum;
    int i;
    int bonus;
    int dx;
    int dy;
    int dist_sq;
    int w;
    int h;
    
    result = 0;

    // Test 1: Basic struct (Point)
    p1.x = 10;
    p1.y = 15;
    if (p1.x + p1.y != 25) return 1;
    result = result + 12;

    // Test 2: Nested struct (Rectangle)
    rect.top_left.x = 0;
    rect.top_left.y = 10;
    rect.bottom_right.x = 10;
    rect.bottom_right.y = 0;
    width = rect.bottom_right.x - rect.top_left.x;
    height = rect.top_left.y - rect.bottom_right.y;
    area = width * height;
    if (area != 100) return 2;
    result = result + 12;

    // Test 3: Struct initialization and access
    s1.id = 1;
    s1.age = 20;
    s1.score = 85;
    s2.id = 2;
    s2.age = 22;
    s2.score = 90;
    avg = (s1.score + s2.score) / 2;
    if (avg != 87) return 3;
    result = result + 12;

    // Test 4: Struct copy
    p2.x = p1.x;
    p2.y = p1.y;
    if (p2.x != 10 || p2.y != 15) return 4;
    result = result + 12;

    // Test 5: Struct array
    points[0].x = 1; points[0].y = 2;
    points[1].x = 3; points[1].y = 4;
    points[2].x = 5; points[2].y = 6;
    sum = 0;
    for (i = 0; i < 3; i = i + 1) {
        sum = sum + points[i].x + points[i].y;
    }
    if (sum != 21) return 5;  // 1+2+3+4+5+6 = 21
    result = result + 12;

    // Test 6: Multiple struct types
    emp.id = 100;
    emp.salary = 50000;
    emp.years = 5;
    bonus = emp.salary / 10;
    if (bonus != 5000) return 6;
    result = result + 12;

    // Test 7: Forward declaration usage
    p3.x = 0;
    p3.y = 0;
    p4.x = 3;
    p4.y = 4;
    // Distance squared
    dx = p4.x - p3.x;
    dy = p4.y - p3.y;
    dist_sq = dx * dx + dy * dy;
    if (dist_sq != 25) return 7;  // 3^2 + 4^2 = 25
    result = result + 12;

    // Test 8: Struct member modification
    points[1].x = 10;
    points[1].y = 20;
    if (points[1].x + points[1].y != 30) return 8;
    result = result + 12;

    // Test 9: Complex struct operations
    r2.top_left = p1;  // x=10, y=15
    r2.bottom_right.x = 20;
    r2.bottom_right.y = 5;
    w = r2.bottom_right.x - r2.top_left.x;
    h = r2.top_left.y - r2.bottom_right.y;
    if (w != 10 || h != 10) return 9;
    result = result + 12;

    // Test 10: Struct comparison (manual)
    if (p1.x == p2.x && p1.y == p2.y) {
        result = result + 12;
    } else {
        return 10;
    }

    return result;  // Should be 120
}
