#include <stdint.h>
#include <stdlib.h>

struct PyPy;

void pypy_gc_init(intptr_t heap_size);

void *pypy_bind_mutator();

void pypy_mmtk_alloc(void *handle, intptr_t heap_size);

