// Self-referential struct: binary tree
struct TreeNode {
    int data;
    struct TreeNode *left;
    struct TreeNode *right;
};

int max(int a, int b) {
    if (a > b) {
        return a;
    }
    return b;
}

int tree_sum(struct TreeNode *node) {
    if (node == 0) {
        return 0;
    }
    return node->data + tree_sum(node->left) + tree_sum(node->right);
}

int tree_depth(struct TreeNode *node) {
    if (node == 0) {
        return 0;
    }
    return 1 + max(tree_depth(node->left), tree_depth(node->right));
}

int main() {
    struct TreeNode root;
    struct TreeNode left;
    struct TreeNode right;
    struct TreeNode left_left;
    
    // Build tree structure
    //        5
    //       / \
    //      3   8
    //     /
    //    1
    
    root.data = 5;
    left.data = 3;
    right.data = 8;
    left_left.data = 1;
    
    root.left = &left;
    root.right = &right;
    left.left = &left_left;
    left.right = 0;
    right.left = 0;
    right.right = 0;
    left_left.left = 0;
    left_left.right = 0;
    
    // Sum: 5 + 3 + 8 + 1 = 17
    int sum;
    sum = tree_sum(&root);
    
    // Depth: 3
    int depth;
    depth = tree_depth(&root);
    
    return sum + depth;  // 17 + 3 = 20
}
