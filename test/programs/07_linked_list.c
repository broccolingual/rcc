// Test 7: Linked list
// Expected: 160

struct Node {
    int data;
    struct Node *next;
};

int list_sum(struct Node *head) {
    int sum;
    struct Node *current;
    
    sum = 0;
    current = head;
    while (current != 0) {
        sum = sum + current->data;
        current = current->next;
    }
    return sum;
}

int list_length(struct Node *head) {
    int len;
    struct Node *current;
    
    len = 0;
    current = head;
    while (current != 0) {
        len = len + 1;
        current = current->next;
    }
    return len;
}

int main() {
    int result;
    struct Node n1;
    struct Node n2;
    struct Node n3;
    struct Node *current;
    struct Node *p;
    int found;
    struct Node n4, n5, n6;
    struct Node nodes[5];
    int i;
    int total;

    result = 0;

    // Test 1: Single node
    n1.data = 10;
    n1.next = 0;
    if (n1.data != 10) return 1;
    result = result + 16;

    // Test 2: Two nodes
    n2.data = 20;
    n2.next = 0;
    n1.next = &n2;
    if (n1.next->data != 20) return 2;
    result = result + 16;

    // Test 3: Three nodes
    n3.data = 30;
    n3.next = 0;
    n2.next = &n3;
    if (n1.next->next->data != 30) return 3;
    result = result + 16;

    // Test 4: List sum
    if (list_sum(&n1) != 60) return 4;  // 10+20+30
    result = result + 16;

    // Test 5: List length
    if (list_length(&n1) != 3) return 5;
    result = result + 16;

    // Test 6: Traverse and modify
    current = &n1;
    while (current != 0) {
        current->data = current->data * 2;
        current = current->next;
    }
    if (n1.data != 20 || n2.data != 40 || n3.data != 60) return 6;
    result = result + 16;

    // Test 7: New sum after modification
    if (list_sum(&n1) != 120) return 7;  // 20+40+60
    result = result + 16;

    // Test 8: Find element
    p = &n1;
    found = 0;
    while (p != 0) {
        if (p->data == 40) {
            found = 1;
            break;
        }
        p = p->next;
    }
    if (found != 1) return 8;
    result = result + 16;

    // Test 9: Chained arrow operator
    n4.data = 1;
    n5.data = 2;
    n6.data = 3;
    n4.next = &n5;
    n5.next = &n6;
    n6.next = 0;
    if (n4.next->next->data != 3) return 9;
    result = result + 16;

    // Test 10: Complex list operations
    for (i = 0; i < 5; i = i + 1) {
        nodes[i].data = (i + 1) * 10;
        if (i < 4) {
            nodes[i].next = &nodes[i + 1];
        } else {
            nodes[i].next = 0;
        }
    }
    total = list_sum(&nodes[0]);
    if (total != 150) return 10;  // 10+20+30+40+50
    result = result + 16;

    return result;  // Should be 160
}
