#include <stdio.h>
#include <stdlib.h>

int foo() { printf("OK\n"); }
int bar(int x, int y) { printf("%d\n", x + y); }
void alloc4(int **p, int a, int b, int c, int d) {
    *p = (int *)malloc(4 * sizeof(int));
    (*p)[0] = a;
    (*p)[1] = b;
    (*p)[2] = c;
    (*p)[3] = d;
}
