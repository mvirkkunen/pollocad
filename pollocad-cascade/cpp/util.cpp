#include <cstdlib>
#include <cstring>

#include "wrapper.h"

Error cascade_error_create(const char* s) {
    size_t len = std::strlen(s);
    char* r = (char *)std::malloc(len + 1);
    if (r == nullptr) {
        return nullptr;
    }
    std::memcpy(r, s, len + 1);
    return r;
}

void cascade_error_free(Error err) {
    std::free(err);
}
