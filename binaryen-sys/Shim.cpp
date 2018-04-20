
#include <cstddef>
#include <cstring>
#include "asm_v_wasm.h"
#include "support/file.h"
#include "pass.h"
#include "tools/optimization-options.h"
#include "tools/fuzzing.h"
#include "binaryen-c.h"

#include "wasm.h"           // For Feature enum
#include "wasm-validator.h" // For WasmValidator

using namespace wasm;

extern "C" BinaryenModuleRef translateToFuzz(const char *data, size_t len) {
    auto module = new Module();

    std::vector<char> input;
    input.resize(len);
    memcpy(&input[0], data, len);

    TranslateToFuzzReader reader(*module, input);
    reader.build();

    // Temporary hack to avoid generating Atomics
    if (!WasmValidator().validate(*module, (FeatureSet) Feature::MVP)) {
        std::cerr << "Invalid module (wrt. Feature::MVP)";
        delete module;
        return new Module();
    }

    return module;
}
