// Union and struct combined usage example
// Demonstrates tagged union pattern (discriminated union)

struct IntValue {
    int value;
};

struct CharValue {
    char c1;
    char c2;
    char c3;
    char c4;
};

// Tagged union: a common pattern in C
struct Variant {
    int tag;  // 0 = int, 1 = char array
    union {
        struct IntValue as_int;
        struct CharValue as_chars;
    } data;
};

// Extract bytes from an integer using union
union IntBytes {
    int value;
    char bytes[4];
};

int get_low_byte(int n) {
    union IntBytes ib;
    ib.value = n;
    return ib.bytes[0];
}

int create_from_bytes(char b0, char b1, char b2, char b3) {
    union IntBytes ib;
    ib.bytes[0] = b0;
    ib.bytes[1] = b1;
    ib.bytes[2] = b2;
    ib.bytes[3] = b3;
    return ib.value;
}

int process_variant(struct Variant *v) {
    if (v->tag == 0) {
        return v->data.as_int.value;
    } else {
        // Sum of char values
        return v->data.as_chars.c1 + v->data.as_chars.c2 + 
               v->data.as_chars.c3 + v->data.as_chars.c4;
    }
}

int main() {
    int result;
    struct Variant v1;
    struct Variant v2;
    
    result = 0;
    
    // Test 1: Tagged union with int value
    v1.tag = 0;
    v1.data.as_int.value = 50;
    result += process_variant(&v1);  // +50
    
    // Test 2: Tagged union with char values
    v2.tag = 1;
    v2.data.as_chars.c1 = 5;
    v2.data.as_chars.c2 = 10;
    v2.data.as_chars.c3 = 15;
    v2.data.as_chars.c4 = 10;
    result += process_variant(&v2);  // +40
    
    // Test 3: Extract low byte from int
    result += get_low_byte(0x1234563C);  // +0x3C = +60
    
    // Test 4: Create int from bytes (little endian: 0x01020304 = 16909060)
    // We just check if it works by extracting low byte
    int combined;
    combined = create_from_bytes(4, 3, 2, 1);  // 0x01020304 in little endian
    result += get_low_byte(combined);  // +4
    
    return result;  // 50 + 40 + 60 + 4 = 154
}
