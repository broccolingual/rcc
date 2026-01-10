// Test 10: Comprehensive integration test
// Expected: 190

typedef struct {
    int id;
    int value;
} Data;

typedef struct GraphNode {
    int vertex;
    int weight;
    struct GraphNode *next;
} GraphNode;

typedef struct {
    int top;
    int items[10];
} Stack;

void stack_push(Stack *s, int val) {
    if (s->top < 10) {
        s->items[s->top] = val;
        s->top = s->top + 1;
    }
}

int stack_pop(Stack *s) {
    if (s->top > 0) {
        s->top = s->top - 1;
        return s->items[s->top];
    }
    return -1;
}

int stack_sum(Stack *s) {
    int sum;
    int i;
    
    sum = 0;
    for (i = 0; i < s->top; i = i + 1) {
        sum = sum + s->items[i];
    }
    return sum;
}

int graph_path_weight(GraphNode *start) {
    int total;
    GraphNode *current;
    
    total = 0;
    current = start;
    while (current != 0) {
        total = total + current->weight;
        current = current->next;
    }
    return total;
}

int main() {
    int result;
    Data d1;
    Stack s;
    int val;
    GraphNode g1, g2, g3;
    Data arr[3];
    int sum;
    int i;
    GraphNode *p;
    int hop_count;
    Stack s2;
    Data *pd;
    Stack final_stack;
    GraphNode *gp;
    int final_sum;

    result = 0;

    // Test 1: Typedef struct
    d1.id = 1;
    d1.value = 100;
    if (d1.id + d1.value != 101) return 1;
    result = result + 19;

    // Test 2: Stack operations - push
    s.top = 0;
    stack_push(&s, 10);
    stack_push(&s, 20);
    stack_push(&s, 30);
    if (s.top != 3) return 2;
    result = result + 19;

    // Test 3: Stack operations - pop
    val = stack_pop(&s);
    if (val != 30 || s.top != 2) return 3;
    result = result + 19;

    // Test 4: Stack sum
    if (stack_sum(&s) != 30) return 4;  // 10+20
    result = result + 19;

    // Test 5: Graph with typedef
    g1.vertex = 1; g1.weight = 10; g1.next = &g2;
    g2.vertex = 2; g2.weight = 20; g2.next = &g3;
    g3.vertex = 3; g3.weight = 30; g3.next = 0;
    if (graph_path_weight(&g1) != 60) return 5;
    result = result + 19;

    // Test 6: Array of typedef structs
    arr[0].id = 1; arr[0].value = 10;
    arr[1].id = 2; arr[1].value = 20;
    arr[2].id = 3; arr[2].value = 30;
    sum = 0;
    for (i = 0; i < 3; i = i + 1) {
        sum = sum + arr[i].value;
    }
    if (sum != 60) return 6;
    result = result + 19;

    // Test 7: Nested structures and pointers
    p = &g1;
    hop_count = 0;
    while (p != 0) {
        hop_count = hop_count + 1;
        p = p->next;
    }
    if (hop_count != 3) return 7;
    result = result + 19;

    // Test 8: Stack with more operations
    s2.top = 0;
    for (i = 1; i <= 5; i = i + 1) {
        stack_push(&s2, i * 10);
    }
    if (stack_sum(&s2) != 150) return 8;  // 10+20+30+40+50
    result = result + 19;

    // Test 9: Complex data manipulation
    pd = &arr[1];
    pd->value = pd->value * 2;  // 20 -> 40
    if (arr[1].value != 40) return 9;
    result = result + 19;

    // Test 10: All features combined
    // Create a graph, push vertices to stack, sum everything
    final_stack.top = 0;
    gp = &g1;
    while (gp != 0) {
        stack_push(&final_stack, gp->weight);
        gp = gp->next;
    }
    final_sum = stack_sum(&final_stack);
    if (final_sum != 60) return 10;  // 10+20+30
    result = result + 19;

    return result;  // Should be 190
}
