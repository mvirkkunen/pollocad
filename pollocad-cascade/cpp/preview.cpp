#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cmath>

#include <AIS_InteractiveContext.hxx>
#include <AIS_ViewController.hxx>
#include <AIS_ViewCube.hxx>
#include <AIS_Shape.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <Graphic3d_GraphicDriver.hxx>
#include <OpenGl_ArbDbg.hxx>
#include <OpenGl_Context.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <OpenGl_View.hxx>
#include <OpenGl_Window.hxx>
#include <TopoDS_Shape.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>

#include "wrapper.h"
#include "util.hpp"

class CascadePreviewImpl;
static CascadePreviewImpl* unwrap(CascadePreview obj) {
    return reinterpret_cast<CascadePreviewImpl *>(obj.ptr);
}

void initFBOTextures(Handle(OpenGl_Context) &glContext, const Handle(OpenGl_Texture) &colorTexture, const Handle(OpenGl_Texture) &depthTexture, uint32_t width, uint32_t height) {
    colorTexture->Init(
        glContext,
        OpenGl_TextureFormat::FindSizedFormat(glContext, GL_RGBA8),
        Graphic3d_Vec2i{(int)width, (int)height},
        Graphic3d_TypeOfTexture::Graphic3d_TOT_2D);

    depthTexture->Init(
        glContext,
        OpenGl_TextureFormat::FindSizedFormat(glContext, GL_DEPTH24_STENCIL8),
        Graphic3d_Vec2i{(int)width, (int)height},
        Graphic3d_TypeOfTexture::Graphic3d_TOT_2D);
}

class CascadePreviewImpl : public AIS_ViewController {
private:
    Handle(Aspect_DisplayConnection) display;
    Handle(OpenGl_FrameBuffer) fbo;
    Handle(OpenGl_GraphicDriver) driver;
    Handle(OpenGl_Texture) colorTexture;
    Handle(OpenGl_Texture) depthTexture;
    Handle(V3d_Viewer) viewer;
    Handle(AIS_InteractiveContext) interactiveContext;
    Handle(AIS_ViewCube) viewCube;
    Handle(V3d_View) view;
    Handle(Aspect_NeutralWindow) window;
    Handle(AIS_Shape) shape;

public:
    explicit CascadePreviewImpl(void *display_handle, void *window_handle) {
        display = new Aspect_DisplayConnection{reinterpret_cast<Aspect_XDisplay *>(display_handle)};

        driver = new OpenGl_GraphicDriver{display, false};
        driver->ChangeOptions().buffersNoSwap = true;
        driver->ChangeOptions().buffersOpaqueAlpha = false;
        driver->ChangeOptions().useSystemBuffer = true;

        viewer = new V3d_Viewer{driver};
        viewer->SetDefaultBackgroundColor(Quantity_NOC_GRAY90);
        viewer->SetDefaultLights();
        viewer->SetLightOn();
        viewer->ActivateGrid(Aspect_GT_Rectangular, Aspect_GDM_Lines);

        interactiveContext = new AIS_InteractiveContext{viewer};

        viewCube = new AIS_ViewCube;
        //myViewCube->SetViewAnimation(myViewAnimation);
        //myViewCube->SetFixedAnimationLoop(false);
        //myViewCube->SetAutoStartAnimation(true);
        viewCube->TransformPersistence()->SetOffset2d(Graphic3d_Vec2i (0, 0));

        view = viewer->CreateView();
        view->SetImmediateUpdate(false);

        auto initGlContext = new OpenGl_Context();
        if (!initGlContext->Init(true)) {
            std::fprintf(stderr, "gl context init failed\n");
            std::exit(1);
        }

        //gldebug("POLLO BEGIN initialize", initGlContext);

        if (!driver->InitContext()) {
            std::fprintf(stderr, "driver init failed\n");
            std::exit(1);
        }

        window = new Aspect_NeutralWindow;
        window->SetVirtual(true);
        window->SetSize(1, 1);
        window->SetNativeHandle(reinterpret_cast<Aspect_Drawable>(window_handle));
        view->SetWindow(window, initGlContext->RenderingContext());

        auto glContext = driver->GetSharedContext();

        interactiveContext->Display(viewCube, false);

        //TopoDS_Shape aBox = BRepPrimAPI_MakeBox(100.0, 50.0, 90.0).Shape();
        TopoDS_Shape sh = BRepPrimAPI_MakeTorus(100.0, 20.0).Shape();
        shape = new AIS_Shape{sh};
        interactiveContext->Display(shape, AIS_Shaded, 0, false);

        fbo = new OpenGl_FrameBuffer;

        Handle(OpenGl_Texture) colorTexture = new OpenGl_Texture;
        Handle(OpenGl_Texture) depthTexture = new OpenGl_Texture;
        initFBOTextures(glContext, colorTexture, depthTexture, 1, 1);

        NCollection_Sequence<Handle(OpenGl_Texture)> colorTextures;
        colorTextures.Append(colorTexture);

        if (!fbo->InitWrapper(glContext, colorTextures, depthTexture)) {
            std::fprintf(stderr, "defaultFbo->InitWrapper failed\n");
            std::exit(1);
        }

        glContext->SetDefaultFrameBuffer(fbo);
    }

    void gldebug(const char *msg, Handle(OpenGl_Context) ctx = Handle(OpenGl_Context){}) {
        (ctx.IsNull() ? driver->GetSharedContext() : ctx)->arbDbg->glDebugMessageInsert(
            GL_DEBUG_SOURCE_APPLICATION, GL_DEBUG_TYPE_MARKER, 0, GL_DEBUG_SEVERITY_HIGH, strlen(msg), msg);
    }

    void paint(uint32_t x, uint32_t y, uint32_t width, uint32_t height) {
        width = std::max(width, (uint32_t)1);
        height = std::max(height, (uint32_t)1);

        gldebug("POLLO BEGIN paint");

        auto glContext = driver->GetSharedContext();

        Standard_Integer oldWidth, oldHeight;
        window->Size(oldWidth, oldHeight);

        if ((int)width != oldWidth || (int)height != oldHeight) {
            initFBOTextures(glContext, fbo->ColorTexture(0), fbo->DepthStencilTexture(), width, height);
            fbo->ChangeViewport(width, height);
            window->SetSize(width, height);
            view->MustBeResized();
        }

        view->Invalidate();

        //interactiveContext->Update(shape, true);

        view->InvalidateImmediate();
        FlushViewEvents(interactiveContext, view, true);

        gldebug("POLLO END paint");

        fbo->UnbindBuffer(glContext);
        fbo->BindReadBuffer(glContext);

        glContext->Functions()->glBlitFramebuffer(
            0, 0, width, height,
            x, y, x + width, y + height,
            GL_COLOR_BUFFER_BIT,
            GL_NEAREST);

        fbo->UnbindBuffer(glContext);
    }

    void mouse_event(int32_t x, int32_t y, int32_t wheel, MouseFlags flags) {
        Graphic3d_Vec2i pos{x, y};

        Aspect_VKeyMouse vkeymouse = Aspect_VKeyMouse_NONE;
        if (flags & MOUSE_FLAG_BUTTON_LEFT) vkeymouse |= Aspect_VKeyMouse_LeftButton;
        if (flags & MOUSE_FLAG_BUTTON_MIDDLE) vkeymouse |= Aspect_VKeyMouse_MiddleButton;
        if (flags & MOUSE_FLAG_BUTTON_RIGHT) vkeymouse |= Aspect_VKeyMouse_RightButton;

        Aspect_VKeyFlags vkeyflags = Aspect_VKeyFlags_NONE;
        if (flags & MOUSE_FLAG_MODIFIER_SHIFT) vkeyflags |= Aspect_VKeyFlags_SHIFT;
        if (flags & MOUSE_FLAG_MODIFIER_CTRL) vkeyflags |= Aspect_VKeyFlags_CTRL;
        if (flags & MOUSE_FLAG_MODIFIER_ALT) vkeyflags |= Aspect_VKeyFlags_ALT;

        if (flags & MOUSE_FLAG_BUTTON_CHANGE) {
            UpdateMouseButtons(pos, vkeymouse, vkeyflags, false);
        } else {
            UpdateMousePosition(pos, vkeymouse, vkeyflags, false);
        }

        if (wheel) {
            UpdateZoom(Aspect_ScrollDelta{pos, (double)wheel / 8.0});
        }
    }

    void set_shape(TopoDS_Shape& new_shape) {
        interactiveContext->Remove(shape, false);
        shape = new AIS_Shape{new_shape};
        interactiveContext->Display(shape, AIS_Shaded, 0, false);
    }
};

extern "C" {

CascadePreview cascade_preview_new(void *display_handle, void* window_handle, Error* err) {
    return protect<CascadePreview>(err, [=]() {
        return CascadePreview { new CascadePreviewImpl{display_handle, window_handle} };
    });
}

void cascade_preview_mouse_event(CascadePreview obj, int32_t x, int32_t y, int32_t wheel, MouseFlags flags, Error *err) {
    return protect<void>(err, [=]() {
        unwrap(obj)->mouse_event(x, y, wheel, flags);
    });
}

void cascade_preview_free(CascadePreview obj) {
    delete unwrap(obj);
}

void cascade_preview_paint(CascadePreview obj, uint32_t x, uint32_t y, uint32_t width, uint32_t height, Error* err) {
    return protect<void>(err, [=]() {
        unwrap(obj)->paint(x, y, width, height);
    });
}

void cascade_preview_set_shape(CascadePreview obj, CascadeShape shape, Error *err) {
    return protect<void>(err, [=]() {
        unwrap(obj)->set_shape(*unwrap(shape));
    });
}

}