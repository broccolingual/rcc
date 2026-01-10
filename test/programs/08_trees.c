// Test 8: Binary trees and BST
// Expected: 170

struct TreeNode {
    int value;
    struct TreeNode *left;
    struct TreeNode *right;
};

int tree_sum(struct TreeNode *root) {
    if (root == 0) return 0;
    return root->value + tree_sum(root->left) + tree_sum(root->right);
}

int tree_height(struct TreeNode *root) {
    int left_h;
    int right_h;
    
    if (root == 0) return 0;
    left_h = tree_height(root->left);
    right_h = tree_height(root->right);
    return 1 + (left_h > right_h ? left_h : right_h);
}

int bst_search(struct TreeNode *root, int target) {
    if (root == 0) return 0;
    if (root->value == target) return 1;
    if (target < root->value) return bst_search(root->left, target);
    return bst_search(root->right, target);
}

int main() {
    int result;
    struct TreeNode root;
    struct TreeNode left_child;
    struct TreeNode right_child;
    struct TreeNode n1, n2, n3, n4, n5;
    struct TreeNode *p;
    struct TreeNode t1, t2, t3, t4, t5, t6, t7;
    int total;

    result = 0;

    // Test 1: Single node tree
    root.value = 10;
    root.left = 0;
    root.right = 0;
    if (root.value != 10) return 1;
    result = result + 17;

    // Test 2: Tree with children
    left_child.value = 5;
    left_child.left = 0;
    left_child.right = 0;
    right_child.value = 15;
    right_child.left = 0;
    right_child.right = 0;
    root.left = &left_child;
    root.right = &right_child;
    if (root.left->value != 5 || root.right->value != 15) return 2;
    result = result + 17;

    // Test 3: Tree sum
    if (tree_sum(&root) != 30) return 3;  // 10+5+15
    result = result + 17;

    // Test 4: Tree height
    if (tree_height(&root) != 2) return 4;
    result = result + 17;

    // Test 5: BST construction
    n1.value = 10; n1.left = &n2; n1.right = &n3;
    n2.value = 5;  n2.left = 0;   n2.right = 0;
    n3.value = 15; n3.left = &n4; n3.right = &n5;
    n4.value = 12; n4.left = 0;   n4.right = 0;
    n5.value = 20; n5.left = 0;   n5.right = 0;
    if (tree_sum(&n1) != 62) return 5;  // 10+5+15+12+20
    result = result + 17;

    // Test 6: BST search - found
    if (bst_search(&n1, 12) != 1) return 6;
    result = result + 17;

    // Test 7: BST search - not found
    if (bst_search(&n1, 7) != 0) return 7;
    result = result + 17;

    // Test 8: Tree height with deeper tree
    if (tree_height(&n1) != 3) return 8;
    result = result + 17;

    // Test 9: Traverse and modify
    p = &n1;
    p->value = p->value * 2;  // 10 -> 20
    p->left->value = p->left->value * 2;  // 5 -> 10
    p->right->value = p->right->value * 2;  // 15 -> 30
    if (n1.value != 20 || n2.value != 10 || n3.value != 30) return 9;
    result = result + 17;

    // Test 10: Full tree operations
    t1.value = 4;
    t2.value = 2; t3.value = 6;
    t4.value = 1; t5.value = 3;
    t6.value = 5; t7.value = 7;
    t1.left = &t2; t1.right = &t3;
    t2.left = &t4; t2.right = &t5;
    t3.left = &t6; t3.right = &t7;
    t4.left = 0; t4.right = 0;
    t5.left = 0; t5.right = 0;
    t6.left = 0; t6.right = 0;
    t7.left = 0; t7.right = 0;
    total = tree_sum(&t1);
    if (total != 28) return 10;  // 1+2+3+4+5+6+7 = 28
    result = result + 17;

    return result;  // Should be 170
}
