#include "binaryen/src/binaryen-c.h"

#ifdef __cplusplus
extern "C" {
#endif

BinaryenModuleRef BinaryenModuleSafeRead(const char* input, size_t inputSize);

void BinaryenShimDisposeBinaryenModuleAllocateAndWriteResult(
    BinaryenModuleAllocateAndWriteResult result
);

#ifdef __cplusplus
}
#endif
