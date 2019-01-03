#include <cstddef>
#include <cstring>

#include "wrapper.h"
#include "asm_v_wasm.h"
#include "support/file.h"
#include "pass.h"
#include "tools/optimization-options.h"
#include "tools/fuzzing.h"
#include "binaryen-c.h"

#include "wasm.h"           // For Feature enum
#include "wasm-validator.h" // For WasmValidator

#include "wasm-binary.h"    // For SafeRead

using namespace wasm;
using namespace std;

// NOTE: this is a copy from binaryen-c.cpp
extern "C" BinaryenModuleRef BinaryenModuleSafeRead(const char* input, size_t inputSize) {
    auto* wasm = new Module;
    vector<char> buffer(false);
    buffer.resize(inputSize);
    copy_n(input, inputSize, buffer.begin());
    try {
        WasmBinaryBuilder parser(*wasm, buffer, false);
        parser.read();
    } catch (ParseException const&) {
        // FIXME: support passing back the exception text
        return NULL;
    }
    return wasm;
}

extern "C" BinaryenModuleRef translateToFuzz(const char *data, size_t len, bool emitAtomics) {
    auto module = new Module();

    vector<char> input;
    input.resize(len);
    memcpy(&input[0], data, len);

    TranslateToFuzzReader reader(*module, input);
    reader.build(emitAtomics);

    return module;
}

extern "C" void BinaryenShimDisposeBinaryenModuleAllocateAndWriteResult(
    BinaryenModuleAllocateAndWriteResult result
) {
    if (result.binary) {
        free(result.binary);
    }
    if (result.sourceMap) {
        free(result.sourceMap);
    }
}
