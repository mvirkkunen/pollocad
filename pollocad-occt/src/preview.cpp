#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cmath>

#include <AIS_AnimationCamera.hxx>
#include <AIS_InteractiveContext.hxx>
#include <AIS_ViewController.hxx>
#include <AIS_ViewCube.hxx>
#include <AIS_Shape.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <Graphic3d_GraphicDriver.hxx>
#include <OpenGl_ArbDbg.hxx>
#include <OpenGl_Context.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <OpenGl_View.hxx>
#include <OpenGl_Window.hxx>
#include <Prs3d_DatumAspect.hxx>
#include <TopoDS_Shape.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>

#include "constants.hpp"

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

class CascadePreview : public AIS_ViewController {
private:
    Handle(OpenGl_FrameBuffer) fbo;
    Handle(OpenGl_GraphicDriver) driver;
    Handle(Aspect_NeutralWindow) window;
    Handle(V3d_View) view;
    Handle(V3d_Viewer) viewer;
    Handle(AIS_InteractiveContext) interactiveContext;
    Handle(AIS_ViewCube) viewCube;
    Handle(AIS_Shape) shape;

public:
    explicit CascadePreview() {
    }

    void init(void *display_handle, void *window_handle) {
        driver = new OpenGl_GraphicDriver{
            new Aspect_DisplayConnection{reinterpret_cast<Aspect_XDisplay *>(display_handle)},
            false};
        driver->ChangeOptions().buffersNoSwap = true;
        driver->ChangeOptions().buffersOpaqueAlpha = false;
        driver->ChangeOptions().useSystemBuffer = false;

        viewer = new V3d_Viewer{driver};
        viewer->SetDefaultBackgroundColor(Quantity_NOC_GRAY90);
        viewer->SetDefaultLights();
        viewer->SetLightOn();
        viewer->ActivateGrid(Aspect_GT_Rectangular, Aspect_GDM_Lines);

        interactiveContext = new AIS_InteractiveContext{viewer};

        view = viewer->CreateView();
        view->SetImmediateUpdate(false);

        Handle(OpenGl_Context) initGlContext = new OpenGl_Context;
        if (!initGlContext->Init(true)) {
            throw std::logic_error{"initGlContext->Init failed\n"};
        }

        //gldebug("POLLO BEGIN initialize", initGlContext);

        if (!driver->InitContext()) {
            throw std::logic_error{"driver->InitContext failed\n"};
        }

        window = new Aspect_NeutralWindow;
        window->SetVirtual(true);
        window->SetSize(1, 1);
        window->SetNativeHandle(reinterpret_cast<Aspect_Drawable>(window_handle));
        view->SetWindow(window, initGlContext->RenderingContext());

        auto glContext = driver->GetSharedContext();

        fbo = new OpenGl_FrameBuffer;

        Handle(OpenGl_Texture) colorTexture = new OpenGl_Texture;
        Handle(OpenGl_Texture) depthTexture = new OpenGl_Texture;
        initFBOTextures(glContext, colorTexture, depthTexture, 4, 2);

        NCollection_Sequence<Handle(OpenGl_Texture)> colorTextures;
        colorTextures.Append(colorTexture);

        if (!fbo->InitWrapper(glContext, colorTextures, depthTexture)) {
            throw std::logic_error{"fbo->InitWrapper failed\n"};
        }

        glContext->SetDefaultFrameBuffer(fbo);

        viewCube = new AIS_ViewCube;
        viewCube->SetSize(100.0);
        viewCube->SetViewAnimation(ViewAnimation());
        viewCube->SetDuration(0.2);
        viewCube->SetFixedAnimationLoop(false);
        viewCube->SetAutoStartAnimation(true);
        viewCube->TransformPersistence()->SetOffset2d(Graphic3d_Vec2i(150, 150));

        Handle(Prs3d_DatumAspect) aspect = new Prs3d_DatumAspect;

        struct { Prs3d_DatumParts part; Quantity_Color color; } axis[] = {
            { Prs3d_DatumParts_XAxis, Quantity_NOC_RED },
            { Prs3d_DatumParts_YAxis, Quantity_NOC_GREEN },
            { Prs3d_DatumParts_ZAxis, Quantity_NOC_BLUE },
        };
        Graphic3d_MaterialAspect mat;
        for (auto &a : axis) {
            aspect->TextAspect(a.part)->SetColor(a.color);
            aspect->ShadingAspect(a.part)->SetAspect(new Graphic3d_AspectFillArea3d{
                Aspect_IS_SOLID, a.color,
                Quantity_NOC_BLACK, Aspect_TOL_SOLID, 1.0f,
                mat, mat
            });
        }

        viewCube->Attributes()->SetDatumAspect(aspect);

        interactiveContext->Display(viewCube, false);
    }

    void gldebug(const char *msg, Handle(OpenGl_Context) ctx = Handle(OpenGl_Context){}) {
        (ctx.IsNull() ? driver->GetSharedContext() : ctx)->arbDbg->glDebugMessageInsert(
            GL_DEBUG_SOURCE_APPLICATION, GL_DEBUG_TYPE_MARKER, 0, GL_DEBUG_SEVERITY_HIGH, strlen(msg), msg);
    }

    void paint(uint32_t x, uint32_t y, uint32_t width, uint32_t height) {
        width = std::min(std::max((uint32_t)1, width), (uint32_t)4096);
        height = std::min(std::max((uint32_t)1, height), (uint32_t)4096);

        gldebug("POLLO BEGIN paint");

        auto glContext = driver->GetSharedContext();

        Standard_Integer oldWidth, oldHeight;
        window->Size(oldWidth, oldHeight);

        if ((int)width != oldWidth || (int)height != oldHeight) {
            initFBOTextures(glContext, fbo->ColorTexture(0), fbo->DepthStencilTexture(), width, height);
            fbo->ChangeViewport(width, height);
            window->SetSize(width, height);
            view->MustBeResized();
            view->Invalidate();
        }

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
        if ((int)flags & (int)MouseFlags::Left) vkeymouse |= Aspect_VKeyMouse_LeftButton;
        if ((int)flags & (int)MouseFlags::Middle) vkeymouse |= Aspect_VKeyMouse_MiddleButton;
        if ((int)flags & (int)MouseFlags::Right) vkeymouse |= Aspect_VKeyMouse_RightButton;

        Aspect_VKeyFlags vkeyflags = Aspect_VKeyFlags_NONE;
        if ((int)flags & (int)MouseFlags::Shift) vkeyflags |= Aspect_VKeyFlags_SHIFT;
        if ((int)flags & (int)MouseFlags::Ctrl) vkeyflags |= Aspect_VKeyFlags_CTRL;
        if ((int)flags & (int)MouseFlags::Alt) vkeyflags |= Aspect_VKeyFlags_ALT;

        if ((int)flags & (int)MouseFlags::ButtonChange) {
            UpdateMouseButtons(pos, vkeymouse, vkeyflags, false);
        } else {
            UpdateMousePosition(pos, vkeymouse, vkeyflags, false);
        }

        if (wheel) {
            UpdateZoom(Aspect_ScrollDelta{pos, (double)wheel / 8.0});
        }
    }

    void set_shape(TopoDS_Shape& new_shape) {
        bool center = shape.IsNull();

        if (!shape.IsNull()) {
            interactiveContext->Remove(shape, false);
        }

        shape = new AIS_Shape{new_shape};
        shape->Attributes()->SetFaceBoundaryDraw(true);
        shape->Attributes()->FaceBoundaryAspect()->SetWidth(2.0);
        shape->Attributes()->FaceBoundaryAspect()->SetTypeOfLine(Aspect_TOL_SOLID);

        interactiveContext->Display(shape, AIS_Shaded, -1, false);

        if (center) {
            view->SetProj(V3d_XnegYnegZpos, false);
            view->FitMinMax(view->Camera(), view->View()->MinMaxValues(), 0.01);
        }

        view->Invalidate();
    }

    bool has_animation() {
        return viewCube->HasAnimation();
    }
};
