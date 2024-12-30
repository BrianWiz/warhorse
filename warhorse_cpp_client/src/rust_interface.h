// rust_interface.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

const char* my_rust_function(const char* input);
void free_string(char* ptr);

#ifdef __cplusplus
}
#endif
