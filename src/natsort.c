#include "headers.h"
#include "globals.h"

int nat_cmp(const void *name1, const void *name2)
{
    const char *n1 = *(char **)name1;
    const char *n2 = *(char **)name2;

    while (n1 != nullptr && n2 != nullptr) {
        if (*n1 < '0' || '9' < *n1 || *n2 < '0' || '9' < *n2) {
            if (*n1 == *n2) {
                n1++;
                n2++;
                continue;
            }
            if (*n1 == '.')
                return 1;
            if (*n2 == '.')
                return -1;
            return *n1 < *n2 ? -1 : 1;
        }

        int i = 0, j = 0;
        while ('0' <= *n1 && *n1 <= '9')
            i = i * 10 + *n1++ - '0';
        while ('0' <= *n2 && *n2 <= '9')
            j = j * 10 + *n2++ - '0';
        if (i == j)
            continue;
        return i < j ? -1 : 1;
    }

    if (n1 == nullptr && n2 == nullptr)
        return 0;
    else if (n1 == nullptr)
        return -1;
    else
        return 1;
}

void nat_sort(char **files, int n)
{
    qsort(files, n, sizeof(char *), nat_cmp);
}

int nat_cmp_pages(const void *page1, const void *page2)
{
    // convert (char [256]) to (char *)
    char *n1 = ((struct page *)page1)->name;
    char *n2 = ((struct page *)page2)->name;
    return nat_cmp(&n1, &n2);
}

void nat_sort_pages()
{
    qsort(pages, total_pages, sizeof(struct page), nat_cmp_pages);
}
