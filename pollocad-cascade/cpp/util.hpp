#pragma once

#include <functional>

#include <Standard_Failure.hxx>
#include <TopoDS_Shape.hxx>

#include "wrapper.h"

Error cascade_error_create(const char* s);

template <typename R>
R protect(Error *err, std::function<R()> f) {
    *err = nullptr;

    try {
        return f();
    } catch (std::logic_error &e) {
        *err = cascade_error_create(e.what());
        return R();
    } catch (Standard_Failure &ex) {
        std::string s(typeid(ex).name());
        s += ": ";
        s += ex.GetMessageString();
        *err = cascade_error_create(s.c_str());
        return R();
    } catch (...) {
        *err = cascade_error_create("Unknown C++ exception");
        return R();
    }
}

inline TopoDS_Shape* unwrap(CascadeShape obj) {
    return reinterpret_cast<TopoDS_Shape *>(obj.ptr);
}

inline CascadeShape wrap(TopoDS_Shape &&sh) {
    return CascadeShape{new TopoDS_Shape{sh}};
}
