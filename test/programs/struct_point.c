// Coordinate manipulation using struct pointers
struct Point {
    int x;
    int y;
};

int distance_squared(struct Point *p1, struct Point *p2) {
    int dx;
    int dy;
    dx = p2->x - p1->x;
    dy = p2->y - p1->y;
    return dx * dx + dy * dy;
}

int main() {
    struct Point p1;
    struct Point p2;
    
    p1.x = 0;
    p1.y = 0;
    p2.x = 3;
    p2.y = 4;
    
    return distance_squared(&p1, &p2); // 3^2 + 4^2 = 25
}
