// Rectangle area calculation using struct pointers
struct Rectangle {
    int x;
    int y;
    int width;
    int height;
};

int calc_area(struct Rectangle *rect) {
    return rect->width * rect->height;
}

// Check if two rectangles overlap
int is_overlapping(struct Rectangle *r1, struct Rectangle *r2) {
    int r1_right;
    int r1_bottom;
    int r2_right;
    int r2_bottom;
    
    r1_right = r1->x + r1->width;
    r1_bottom = r1->y + r1->height;
    r2_right = r2->x + r2->width;
    r2_bottom = r2->y + r2->height;
    
    if (r1->x >= r2_right) {
        return 0;
    }
    if (r1_right <= r2->x) {
        return 0;
    }
    if (r1->y >= r2_bottom) {
        return 0;
    }
    if (r1_bottom <= r2->y) {
        return 0;
    }
    return 1;
}

int main() {
    struct Rectangle r1;
    struct Rectangle r2;
    int area1;
    int area2;
    
    r1.x = 0;
    r1.y = 0;
    r1.width = 10;
    r1.height = 10;
    
    r2.x = 5;
    r2.y = 5;
    r2.width = 10;
    r2.height = 10;
    
    if (is_overlapping(&r1, &r2)) {
        area1 = calc_area(&r1);
        area2 = calc_area(&r2);
        return area1 + area2; // 100 + 100 = 200
    }
    
    return 0;
}
