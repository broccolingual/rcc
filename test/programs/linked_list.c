// Self-referential struct: singly linked list
struct Node {
    int value;
    struct Node *next;
};

int main() {
    struct Node node1;
    struct Node node2;
    struct Node node3;
    
    // Set node values
    node1.value = 10;
    node2.value = 20;
    node3.value = 30;
    
    // Build links
    node1.next = &node2;
    node2.next = &node3;
    node3.next = 0;  // NULL
    
    // Traverse list and calculate sum
    struct Node *current;
    int sum;
    
    current = &node1;
    sum = 0;
    
    while (current != 0) {
        sum = sum + current->value;
        current = current->next;
    }
    
    return sum;  // 10 + 20 + 30 = 60
}
