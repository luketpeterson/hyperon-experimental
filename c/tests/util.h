#ifndef UTIL_H
#define UTIL_H

#include <hyperon/hyperon.h>

#define BUF_SIZE 4096

void str_to_buf(const char *str, void *context);

char* stratom(atom_t const* atom);
atom_t expr(atom_t atom, ...);

#endif /* UTIL_H */
