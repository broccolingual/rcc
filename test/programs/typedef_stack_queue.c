// Stack and Queue implementation using typedef

// Type definitions
typedef int Data;
typedef int Bool;
typedef int Size;

// Constants
enum {
    CAPACITY = 10,
    TRUE = 1,
    FALSE = 0,
    SUCCESS = 0,
    FAILURE = -1
};

// Stack structure using typedef
typedef struct Stack {
    Data items[10];  // CAPACITY
    Size top;
} Stack;

// Queue structure using typedef
typedef struct Queue {
    Data items[10];  // CAPACITY
    Size front;
    Size rear;
    Size count;
} Queue;

// Stack operations
void stack_init(Stack *s) {
    s->top = -1;
}

Bool stack_is_empty(Stack *s) {
    return s->top == -1;
}

Bool stack_is_full(Stack *s) {
    return s->top == CAPACITY - 1;
}

int stack_push(Stack *s, Data value) {
    if (stack_is_full(s)) {
        return FAILURE;
    }
    s->top++;
    s->items[s->top] = value;
    return SUCCESS;
}

Data stack_pop(Stack *s) {
    Data value;
    if (stack_is_empty(s)) {
        return FAILURE;
    }
    value = s->items[s->top];
    s->top--;
    return value;
}

Data stack_peek(Stack *s) {
    if (stack_is_empty(s)) {
        return FAILURE;
    }
    return s->items[s->top];
}

Size stack_size(Stack *s) {
    return s->top + 1;
}

// Queue operations
void queue_init(Queue *q) {
    q->front = 0;
    q->rear = -1;
    q->count = 0;
}

Bool queue_is_empty(Queue *q) {
    return q->count == 0;
}

Bool queue_is_full(Queue *q) {
    return q->count == CAPACITY;
}

int queue_enqueue(Queue *q, Data value) {
    if (queue_is_full(q)) {
        return FAILURE;
    }
    q->rear = (q->rear + 1) % CAPACITY;
    q->items[q->rear] = value;
    q->count++;
    return SUCCESS;
}

Data queue_dequeue(Queue *q) {
    Data value;
    if (queue_is_empty(q)) {
        return FAILURE;
    }
    value = q->items[q->front];
    q->front = (q->front + 1) % CAPACITY;
    q->count--;
    return value;
}

Data queue_front(Queue *q) {
    if (queue_is_empty(q)) {
        return FAILURE;
    }
    return q->items[q->front];
}

Size queue_size(Queue *q) {
    return q->count;
}

// Test function: stack operations
int test_stack() {
    Stack s;
    int result;

    stack_init(&s);

    // Push values: 10, 20, 30
    stack_push(&s, 10);
    stack_push(&s, 20);
    stack_push(&s, 30);

    // Size should be 3
    result = stack_size(&s);  // 3

    // Pop twice: 30, 20
    result = result + stack_pop(&s);  // 3 + 30 = 33
    result = result + stack_pop(&s);  // 33 + 20 = 53

    // Peek: 10
    result = result + stack_peek(&s);  // 53 + 10 = 63

    // Pop last: 10
    result = result + stack_pop(&s);  // 63 + 10 = 73

    // Empty check should be TRUE (1)
    result = result + stack_is_empty(&s);  // 73 + 1 = 74

    return result;
}

// Test function: queue operations
int test_queue() {
    Queue q;
    int result;

    queue_init(&q);

    // Enqueue values: 5, 10, 15, 20
    queue_enqueue(&q, 5);
    queue_enqueue(&q, 10);
    queue_enqueue(&q, 15);
    queue_enqueue(&q, 20);

    // Size should be 4
    result = queue_size(&q);  // 4

    // Front should be 5
    result = result + queue_front(&q);  // 4 + 5 = 9

    // Dequeue twice: 5, 10
    result = result + queue_dequeue(&q);  // 9 + 5 = 14
    result = result + queue_dequeue(&q);  // 14 + 10 = 24

    // Front should be 15
    result = result + queue_front(&q);  // 24 + 15 = 39

    // Size should be 2
    result = result + queue_size(&q);  // 39 + 2 = 41

    return result;
}

// Test function: combined operations
int test_combined() {
    Stack s;
    Queue q;
    int i;
    int sum;

    stack_init(&s);
    queue_init(&q);

    // Push to stack: 1, 2, 3, 4, 5
    for (i = 1; i <= 5; i++) {
        stack_push(&s, i);
    }

    // Enqueue to queue: 1, 2, 3, 4, 5
    for (i = 1; i <= 5; i++) {
        queue_enqueue(&q, i);
    }

    sum = 0;

    // Pop from stack (LIFO): 5, 4, 3
    for (i = 0; i < 3; i++) {
        sum = sum + stack_pop(&s);
    }
    // sum = 5 + 4 + 3 = 12

    // Dequeue from queue (FIFO): 1, 2, 3
    for (i = 0; i < 3; i++) {
        sum = sum + queue_dequeue(&q);
    }
    // sum = 12 + 1 + 2 + 3 = 18

    return sum;
}

int main() {
    int stack_result;
    int queue_result;
    int combined_result;
    int total;

    stack_result = test_stack();      // 74
    queue_result = test_queue();      // 41
    combined_result = test_combined(); // 18

    // Total: 74 + 41 + 18 = 133
    total = stack_result + queue_result + combined_result;

    return total;
}
