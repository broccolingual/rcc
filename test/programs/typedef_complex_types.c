// Complex typedef usage: function pointers, nested structures, and type aliases

// Basic type aliases
typedef int Int;
typedef char Char;
typedef long Long;

// Pointer type aliases
typedef int* IntPtr;
typedef char* CharPtr;
typedef IntPtr* IntPtrPtr;

// Constants
enum {
    ARRAY_SIZE = 5,
    SUCCESS = 1,
    FAILURE = 0
};

// Structure type aliases
typedef struct Point {
    Int x;
    Int y;
} Point;

typedef struct Rectangle {
    Point top_left;
    Point bottom_right;
} Rectangle;

// Pointer to structure
typedef Point* PointPtr;
typedef Rectangle* RectanglePtr;

// Array type (using typedef for readability)
typedef Int IntArray[5];  // ARRAY_SIZE

// Nested typedef structures
typedef struct {
    Int id;
    Point location;
} Entity;

typedef struct {
    Entity entities[5];  // ARRAY_SIZE
    Int count;
} EntityManager;

// Union with typedef
typedef union {
    Int as_int;
    Char as_bytes[4];
} IntBytes;

// Calculate distance squared between two points
Int distance_squared(PointPtr p1, PointPtr p2) {
    Int dx;
    Int dy;

    dx = p1->x - p2->x;
    dy = p1->y - p2->y;

    return dx * dx + dy * dy;
}

// Calculate rectangle area
Int rectangle_area(RectanglePtr rect) {
    Int width;
    Int height;

    width = rect->bottom_right.x - rect->top_left.x;
    height = rect->bottom_right.y - rect->top_left.y;

    return width * height;
}

// Check if point is inside rectangle
Int point_in_rectangle(PointPtr p, RectanglePtr rect) {
    if (p->x >= rect->top_left.x && p->x <= rect->bottom_right.x) {
        if (p->y >= rect->top_left.y && p->y <= rect->bottom_right.y) {
            return SUCCESS;
        }
    }
    return FAILURE;
}

// Initialize entity manager
void init_manager(EntityManager *mgr) {
    mgr->count = 0;
}

// Add entity to manager
Int add_entity(EntityManager *mgr, Int id, Int x, Int y) {
    Int index;

    if (mgr->count >= ARRAY_SIZE) {
        return FAILURE;
    }

    index = mgr->count;
    mgr->entities[index].id = id;
    mgr->entities[index].location.x = x;
    mgr->entities[index].location.y = y;
    mgr->count++;

    return SUCCESS;
}

// Find entity by id
PointPtr find_entity_location(EntityManager *mgr, Int id) {
    Int i;

    for (i = 0; i < mgr->count; i++) {
        if (mgr->entities[i].id == id) {
            return &mgr->entities[i].location;
        }
    }

    return 0;  // NULL
}

// Test double pointer typedef
Int test_double_pointer() {
    Int value;
    IntPtr ptr;
    IntPtrPtr pptr;
    Int result;

    value = 42;
    ptr = &value;
    pptr = &ptr;

    result = **pptr;  // Should be 42

    return result;
}

// Test union typedef
Int test_union() {
    IntBytes data;
    Int sum;
    Int i;

    data.as_int = 0x01020304;

    // Sum all bytes
    sum = 0;
    for (i = 0; i < 4; i++) {
        sum = sum + (data.as_bytes[i] & 0xFF);
    }

    // Bytes: 0x04, 0x03, 0x02, 0x01 (little-endian)
    // Sum: 4 + 3 + 2 + 1 = 10
    return sum;
}

// Test array typedef
Int test_array_typedef() {
    IntArray arr;
    Int i;
    Int sum;

    // Initialize array
    for (i = 0; i < ARRAY_SIZE; i++) {
        arr[i] = i * 2;  // 0, 2, 4, 6, 8
    }

    // Calculate sum
    sum = 0;
    for (i = 0; i < ARRAY_SIZE; i++) {
        sum = sum + arr[i];
    }

    // Sum: 0 + 2 + 4 + 6 + 8 = 20
    return sum;
}

int main() {
    Point p1;
    Point p2;
    Rectangle rect;
    EntityManager mgr;
    PointPtr found_loc;
    Int result;

    // Test 1: Distance calculation
    p1.x = 0;
    p1.y = 0;
    p2.x = 3;
    p2.y = 4;

    result = distance_squared(&p1, &p2);  // 3^2 + 4^2 = 25

    // Test 2: Rectangle area
    rect.top_left.x = 0;
    rect.top_left.y = 0;
    rect.bottom_right.x = 5;
    rect.bottom_right.y = 6;

    result = result + rectangle_area(&rect);  // 25 + 30 = 55

    // Test 3: Point in rectangle
    p1.x = 2;
    p1.y = 3;
    result = result + point_in_rectangle(&p1, &rect);  // 55 + 1 = 56

    p2.x = 10;
    p2.y = 10;
    result = result + point_in_rectangle(&p2, &rect);  // 56 + 0 = 56

    // Test 4: Entity manager
    init_manager(&mgr);
    add_entity(&mgr, 101, 10, 20);
    add_entity(&mgr, 102, 30, 40);
    add_entity(&mgr, 103, 50, 60);

    found_loc = find_entity_location(&mgr, 102);
    if (found_loc != 0) {
        result = result + found_loc->x + found_loc->y;  // 56 + 30 + 40 = 126
    }

    // Test 5: Double pointer
    result = result + test_double_pointer();  // 126 + 42 = 168

    // Test 6: Union
    result = result + test_union();  // 168 + 10 = 178

    // Test 7: Array typedef
    result = result + test_array_typedef();  // 178 + 20 = 198

    return result;
}
