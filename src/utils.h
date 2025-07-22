#include "structs.h"

void load_file(char *path, struct appstate *s);
void calc_progress(struct appstate *s);
enum modes calc_mode(struct page *pages);
