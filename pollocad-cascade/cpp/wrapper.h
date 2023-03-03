#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef char *Error;

typedef enum MouseFlags {
    MOUSE_FLAG_BUTTON_LEFT    = (1 << 1),
    MOUSE_FLAG_BUTTON_MIDDLE  = (1 << 2),
    MOUSE_FLAG_BUTTON_RIGHT   = (1 << 3),
    MOUSE_FLAG_BUTTON_CHANGE  = (1 << 4),
    MOUSE_FLAG_MODIFIER_SHIFT = (1 << 5),
    MOUSE_FLAG_MODIFIER_CTRL  = (1 << 6),
    MOUSE_FLAG_MODIFIER_ALT   = (1 << 7),
} MouseFlags;

typedef enum BooleanOp {
    BOOLEAN_OP_UNION = 1,
    BOOLEAN_OP_DIFFERENCE = 2,
    BOOLEAN_OP_INTERSECTION = 3,
} BooleanOp;

typedef struct CascadePreview { void* ptr; } CascadePreview;
typedef struct CascadeShape { void* ptr; } CascadeShape;

CascadePreview cascade_preview_new(void *display_handle, void *window_handle, Error *err);
void cascade_preview_free(CascadePreview obj);
void cascade_preview_mouse_event(CascadePreview obj, int32_t x, int32_t y, int32_t wheel, MouseFlags flags, Error *err);
void cascade_preview_paint(CascadePreview obj, uint32_t x, uint32_t y,  uint32_t width, uint32_t height, Error *err);
void cascade_preview_set_shape(CascadePreview obj, CascadeShape shape, Error *err);

CascadeShape cascade_shape_new_box(double x, double y, double z, Error *err);
CascadeShape cascade_shape_new_cylinder(double r, double h, Error *err);
CascadeShape cascade_shape_clone(CascadeShape obj, Error *err);
CascadeShape cascade_shape_boolean_op(CascadeShape obj, CascadeShape other, BooleanOp op, Error *err);
CascadeShape cascade_shape_transform(CascadeShape obj, const double *matrix, Error *err);
void cascade_shape_free(CascadeShape obj);

void cascade_error_free(Error error);

#ifdef __cplusplus
}
#endif
