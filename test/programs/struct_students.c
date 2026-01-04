// Grade management system using struct array
struct Student {
    int id;
    int score;
};

// 構造体ポインタを使ってスコアで昇順ソート（バブルソート）
void sort_students(struct Student *students, int n) {
    int i;
    int j;
    struct Student temp;
    
    i = 0;
    while (i < n - 1) {
        j = 0;
        while (j < n - i - 1) {
            if (students[j].score > students[j + 1].score) {
                temp = students[j];
                students[j] = students[j + 1];
                students[j + 1] = temp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

int main() {
    struct Student students[5];
    
    students[0].id = 1;
    students[0].score = 85;
    
    students[1].id = 2;
    students[1].score = 92;
    
    students[2].id = 3;
    students[2].score = 78;
    
    students[3].id = 4;
    students[3].score = 95;
    
    students[4].id = 5;
    students[4].score = 88;
    
    sort_students(students, 5);
    
    return students[2].score; // Median after sorting: 88
}
