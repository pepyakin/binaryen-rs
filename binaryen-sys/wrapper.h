#include "binaryen/src/binaryen-c.h"
#include <stdbool.h>

BinaryenImportRef BinaryenAddGlobal(BinaryenModuleRef module, const char* name, BinaryenType type, int mutable_, BinaryenExpressionRef init);
BinaryenExpressionRef BinaryenGetGlobal(BinaryenModuleRef module, const char *name, BinaryenType type);
BinaryenExpressionRef BinaryenSetGlobal(BinaryenModuleRef module, const char *name, BinaryenExpressionRef value);
