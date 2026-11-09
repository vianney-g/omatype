// The C interface of omatype's Rust engine (engine/src/lib.rs). Every call
// takes and returns JSON; returned strings are released with omt_free.
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Engine OmtEngine;

OmtEngine *omt_open(const char *db_path);
char *omt_call(OmtEngine *engine, const char *method, const char *args);
void omt_free(char *s);
void omt_close(OmtEngine *engine);

#ifdef __cplusplus
}
#endif
