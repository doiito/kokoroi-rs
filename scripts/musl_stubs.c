// Stub for glibc's __cpu_features2 symbol.
// ONNX Runtime (x86_64) references this glibc-internal symbol.
// When linking statically against musl (which lacks __cpu_features2),
// we provide a minimal stub. Setting to 0 disables advanced CPU features
// (AVX etc.) — ONNX Runtime falls back to basic SSE/SSE2, which all x86_64 CPUs support.
#include <stdint.h>
uint64_t __cpu_features2 = 0;
