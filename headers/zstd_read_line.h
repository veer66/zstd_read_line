#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define BUF_SIZE 65536

typedef struct ZstdLineReader {

} ZstdLineReader;

struct ZstdLineReader *zstd_line_read_new(const char *zstd_file_path);

char *zstd_line_read(struct ZstdLineReader *reader);

void zstd_line_read_delete_line(char *line);

void zstd_line_read_delete(struct ZstdLineReader *reader);
