// Simple graph representation using typedef and adjacency matrix

// Type definitions
typedef int Vertex;
typedef int Weight;
typedef int Bool;

// Constants using enum
enum {
    NUM_VERTICES = 4,
    INF = 9999,
    TRUE = 1,
    FALSE = 0
};

// Graph structure using adjacency matrix
typedef struct Graph {
    Weight matrix[4][4];  // NUM_VERTICES x NUM_VERTICES
    int size;
} Graph;

// Initialize graph
void init_graph(Graph *g, int size) {
    int i;
    int j;

    g->size = size;
    for (i = 0; i < size; i++) {
        for (j = 0; j < size; j++) {
            if (i == j) {
                g->matrix[i][j] = 0;
            } else {
                g->matrix[i][j] = INF;
            }
        }
    }
}

// Add edge to graph
void add_edge(Graph *g, Vertex src, Vertex dest, Weight weight) {
    g->matrix[src][dest] = weight;
}

// Get edge weight
Weight get_edge(Graph *g, Vertex src, Vertex dest) {
    return g->matrix[src][dest];
}

// Count edges in graph
int count_edges(Graph *g) {
    int count;
    int i;
    int j;

    count = 0;
    for (i = 0; i < g->size; i++) {
        for (j = 0; j < g->size; j++) {
            if (i != j && g->matrix[i][j] != INF) {
                count++;
            }
        }
    }

    return count;
}

// Find minimum path using simple search
int find_min_path(Graph *g, Vertex src, Vertex dest) {
    int direct;
    int via;
    int min;
    int k;

    // Direct path
    direct = g->matrix[src][dest];
    min = direct;

    // Try all intermediate vertices
    for (k = 0; k < g->size; k++) {
        if (k != src && k != dest) {
            via = g->matrix[src][k] + g->matrix[k][dest];
            if (via < min) {
                min = via;
            }
        }
    }

    return min;
}

// Test typedef with multiple levels
typedef int Integer;
typedef Integer Number;
typedef Number Value;

Value compute_sum(Value a, Value b) {
    return a + b;
}

// Test typedef with pointer
typedef Graph* GraphPtr;

GraphPtr create_test_graph() {
    // Use static to avoid returning stack address
    static Graph g;
    init_graph(&g, 4);
    return &g;
}

int main() {
    Graph graph;
    GraphPtr gptr;
    int edge_count;
    int min_path;
    Value sum;
    int result;

    // Initialize graph with 4 vertices
    init_graph(&graph, 4);

    // Add edges:
    //   0 --5--> 1
    //   0 --3--> 2
    //   1 --2--> 3
    //   2 --4--> 3
    add_edge(&graph, 0, 1, 5);
    add_edge(&graph, 0, 2, 3);
    add_edge(&graph, 1, 3, 2);
    add_edge(&graph, 2, 3, 4);

    // Count edges (should be 4)
    edge_count = count_edges(&graph);

    // Find minimum path from 0 to 3
    // Direct: INF
    // Via 1: 5 + 2 = 7
    // Via 2: 3 + 4 = 7
    // Min: 7
    min_path = find_min_path(&graph, 0, 3);

    // Test typedef chain
    sum = compute_sum(10, 20);  // 30

    // Test typedef pointer
    gptr = create_test_graph();
    add_edge(gptr, 0, 1, 1);

    // Result: edge_count * 10 + min_path + sum
    // = 4 * 10 + 7 + 30 = 77
    result = edge_count * 10 + min_path + sum;

    return result;
}
