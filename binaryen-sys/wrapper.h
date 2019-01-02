#include "binaryen/src/binaryen-c.h"

BinaryenModuleRef BinaryenModuleSafeRead(const char* input, size_t inputSize);

void BinaryenShimDisposeBinaryenModuleAllocateAndWriteResult(
    BinaryenModuleAllocateAndWriteResult result
);
