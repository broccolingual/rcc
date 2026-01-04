// Bubble sort
void bubble_sort(int *arr, int n) {
    int i;
    int j;
    int temp;
    
    i = 0;
    while (i < n - 1) {
        j = 0;
        while (j < n - i - 1) {
            if (arr[j] > arr[j + 1]) {
                temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

int main() {
    int arr[5] = {5, 2, 8, 1, 9};
    
    bubble_sort(arr, 5);
    
    return arr[2]; // Median after sorting: 5
}
