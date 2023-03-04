#pragma once

#include <functional>

#include <Standard_Failure.hxx>

template<typename T>
struct CppResult {
    T result;
    char *err;

    ~CppResult() {
        free(err);
    }

    T get(char **err) const {
        *err = this->err;
        return std::move(result);
    }
};

template<>
struct CppResult<void> {
    char *err;

    ~CppResult() {
        free(err);
    }

    void get(char **err) const {
        *err = this->err;
    }
};

/*template<typename T>
struct CppResult {
    struct empty{};
    std::conditional_t<std::is_same_v<T, void>, empty, T> result;
    char *err;

    ~CppResult() {
        free(err);
    }

    template <typename U = T, typename = typename std::enable_if<!std::is_void_v<U>>::type>
    T get(char **err) const {
        *err = this->err;
        return std::move(result);
    }

    template <typename U = T, typename = typename std::enable_if<std::is_void_v<U>>::type>
    void get(char **err) const {
        *err = this->err;
    }
};*/

template <typename T>
CppResult<T> protect(std::function<T()> f) {
    CppResult<T> result;

    try {
        result.result = f();
        result.err = nullptr;
    } catch (std::logic_error &e) {
        result.result = T{};
        result.err = strdup(e.what());
    } catch (Standard_Failure &ex) {
        result.result = T{};
        result.err = strdup((std::string(typeid(ex).name()) + ": " + ex.GetMessageString()).c_str());
    } catch (...) {
        result.result = T{};
        result.err = strdup("Unknown C++ exception");
    }

    return result;
}

template <>
CppResult<void> protect(std::function<void()> f) {
    CppResult<void> result;

    try {
        f();
        result.err = nullptr;
    } catch (std::logic_error &e) {
        result.err = strdup(e.what());
    } catch (Standard_Failure &ex) {
        result.err = strdup((std::string(typeid(ex).name()) + ": " + ex.GetMessageString()).c_str());
    } catch (...) {
        result.err = strdup("Unknown C++ exception");
    }

    return result;
}
