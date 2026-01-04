// Test chained arrow operators with self-referential structs
struct Node {
    int value;
    struct Node *next;
};

int main() {
    struct Node node1;
    struct Node node2;
    struct Node node3;
    struct Node node4;
    
    // Setup linked list: node1 -> node2 -> node3 -> node4
    node1.value = 10;
    node2.value = 20;
    node3.value = 30;
    node4.value = 40;
    
    node1.next = &node2;
    node2.next = &node3;
    node3.next = &node4;
    node4.next = 0;
    
    // Test single arrow operator
    int val1;
    val1 = node1.next->value;  // Should be 20
    
    // Test double arrow operators
    int val2;
    val2 = node1.next->next->value;  // Should be 30
    
    // Test triple arrow operators
    int val3;
    val3 = node1.next->next->next->value;  // Should be 40
    
    // Calculate sum: 20 + 30 + 40 = 90
    return val1 + val2 + val3;
}
