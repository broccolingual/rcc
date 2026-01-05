// Shape calculator using enum and struct
enum ShapeType {
    CIRCLE,
    RECTANGLE,
    TRIANGLE
};

struct Shape {
    enum ShapeType type;
    int width;
    int height;
};

int calculate_area(struct Shape *shape) {
    if (shape->type == CIRCLE) {
        // Simplified circle area: pi * r^2 -> 3 * r^2
        int r;
        r = shape->width;
        return 3 * r * r;
    }
    if (shape->type == RECTANGLE) {
        return shape->width * shape->height;
    }
    if (shape->type == TRIANGLE) {
        // Triangle area: (base * height) / 2
        return (shape->width * shape->height) / 2;
    }
    return 0;
}

int main() {
    struct Shape circle;
    struct Shape rectangle;
    struct Shape triangle;
    int total;
    
    // Circle with radius 2
    circle.type = CIRCLE;
    circle.width = 2;
    circle.height = 0;
    
    // Rectangle 3x4
    rectangle.type = RECTANGLE;
    rectangle.width = 3;
    rectangle.height = 4;
    
    // Triangle with base 6 and height 4
    triangle.type = TRIANGLE;
    triangle.width = 6;
    triangle.height = 4;
    
    // Calculate total area
    total = 0;
    total += calculate_area(&circle);     // 3 * 2 * 2 = 12
    total += calculate_area(&rectangle);  // 3 * 4 = 12
    total += calculate_area(&triangle);   // (6 * 4) / 2 = 12
    
    return total; // 12 + 12 + 12 = 36
}
