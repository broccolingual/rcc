// Binary search (find value in sorted array)
int binary_search(int *arr, int n, int target) {
    int left;
    int right;
    int mid;
    
    left = 0;
    right = n - 1;
    
    while (left <= right) {
        mid = (left + right) / 2;
        if (arr[mid] == target) {
            return mid;
        }
        if (arr[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    
    return -1;
}

int main() {
    int arr[7] = {1, 3, 5, 7, 9, 11, 13};
    
    return binary_search(arr, 7, 9); // Index 4
}
