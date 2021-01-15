#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define BUF_SIZE 65536

void *zstd_line_read_new(const char *zstd_file_path);

const char *zstd_line_read(void *reader);
