#include <TopoDS_Shape.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>

#include "wrapper.h"
#include "util.hpp"

CascadeShape cascade_shape_new_box(double x, double y, double z, Error *err) {
    return protect<CascadeShape>(err, [=]() {
        return wrap(BRepPrimAPI_MakeBox{x, y, z});
    });
}

CascadeShape cascade_shape_new_cylinder(double r, double h, Error *err) {
    return protect<CascadeShape>(err, [=]() {
        return wrap(BRepPrimAPI_MakeCylinder{r, h});
    });
}

CascadeShape cascade_shape_clone(CascadeShape obj, Error *err) {
    return protect<CascadeShape>(err, [=]() {
        auto sh = unwrap(obj);
        if (sh->IsNull()) {
            return wrap(TopoDS_Shape{});
        }

        return wrap(BRepBuilderAPI_Copy{*sh});
    });
}

CascadeShape cascade_shape_boolean_op(CascadeShape obj, CascadeShape other, BooleanOp op, Error *err) {
    return protect<CascadeShape>(err, [=]() {
        auto a = unwrap(obj);
        auto b = unwrap(other);

        switch (op) {
            default:
            case BOOLEAN_OP_UNION:
                return wrap(BRepAlgoAPI_Fuse{*a, *b});
            case BOOLEAN_OP_DIFFERENCE:
                return wrap(BRepAlgoAPI_Cut{*a, *b});
            case BOOLEAN_OP_INTERSECTION:
                return wrap(BRepAlgoAPI_Common{*a, *b});
        }
    });
}

CascadeShape cascade_shape_transform(CascadeShape obj, const double *matrix, Error *err) {
    return protect<CascadeShape>(err, [=]() {
        gp_Trsf xform{};
        xform.SetValues(
            matrix[0], matrix[4], matrix[8], matrix[12],
            matrix[1], matrix[5], matrix[9], matrix[13],
            matrix[2], matrix[6], matrix[10], matrix[14]
        );

        return wrap(BRepBuilderAPI_Transform{*unwrap(obj), xform});
    });
}

void cascade_shape_free(CascadeShape obj) {
    delete unwrap(obj);
}
