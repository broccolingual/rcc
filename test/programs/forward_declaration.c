// Forward declaration and self-reference
struct List;  // Forward declaration

struct List {
    int value;
    struct List *next;
};

int count_nodes(struct List *head) {
    int count;
    count = 0;
    while (head != 0) {
        count = count + 1;
        head = head->next;
    }
    return count;
}

int main() {
    struct List a;
    struct List b;
    struct List c;
    
    a.value = 100;
    b.value = 200;
    c.value = 300;
    
    a.next = &b;
    b.next = &c;
    c.next = 0;
    
    int count;
    count = count_nodes(&a);
    
    return count;  // 3
}
