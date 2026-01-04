// Binary Search Tree: Build from array and traverse in-order
struct TreeNode {
    int value;
    struct TreeNode *left;
    struct TreeNode *right;
};

// Insert value into BST
struct TreeNode* insert(struct TreeNode *root, struct TreeNode *new_node, int value) {
    if (root == 0) {
        new_node->value = value;
        new_node->left = 0;
        new_node->right = 0;
        return new_node;
    }
    
    if (value < root->value) {
        root->left = insert(root->left, new_node, value);
    } else {
        root->right = insert(root->right, new_node, value);
    }
    
    return root;
}

// In-order traversal: stores sorted values into array
int in_order_traversal(struct TreeNode *node, int *result, int index) {
    if (node == 0) {
        return index;
    }
    
    index = in_order_traversal(node->left, result, index);
    result[index++] = node->value;
    index = in_order_traversal(node->right, result, index);
    
    return index;
}

int main() {
    // Input array (unsorted)
    int arr[] = {15, 10, 20, 8, 12, 17, 25};
    
    // Node pool for BST
    struct TreeNode nodes[7];
    struct TreeNode *root;
    int i;
    
    // Build BST
    root = 0;
    for (i = 0; i < 7; i++) {
        root = insert(root, &nodes[i], arr[i]);
    }
    
    // Get sorted array via in-order traversal
    int sorted[7];
    in_order_traversal(root, sorted, 0);
    
    // sorted array should be: [8, 10, 12, 15, 17, 20, 25]
    // Return median value: sorted[3] = 15
    return sorted[3];
}
