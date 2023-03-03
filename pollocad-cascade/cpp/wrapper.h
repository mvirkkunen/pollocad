#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct CascadePreview { void* ptr; } CascadePreview;

typedef char *Error;

typedef enum MouseFlags {
    MOUSE_BUTTON_LEFT         = (1 << 1),
    MOUSE_BUTTON_MIDDLE       = (1 << 2),
    MOUSE_BUTTON_RIGHT        = (1 << 3),
    MOUSE_MODIFIER_SHIFT      = (1 << 4),
    MOUSE_MODIFIER_CTRL       = (1 << 5),
    MOUSE_MODIFIER_ALT        = (1 << 6),
    MOUSE_FLAGS_BUTTON_CHANGE = (1 << 7),
} MouseFlags;

CascadePreview cascade_preview_new(void *native_display, void *native_window, Error *err);
void cascade_preview_free(CascadePreview obj);
void cascade_preview_mouse_event(CascadePreview obj, int32_t x, int32_t y, int32_t wheel, MouseFlags flags, Error *err);
void cascade_preview_paint(CascadePreview obj, uint32_t x, uint32_t y,  uint32_t width, uint32_t height, float angle, Error *err);

void error_free2(Error error);

#ifdef __cplusplus
}
#endif
