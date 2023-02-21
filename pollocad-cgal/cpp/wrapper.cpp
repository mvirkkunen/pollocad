#include <cstdlib>
#include <functional>
#include <memory>

#include <CGAL/Cartesian.h>
#include <CGAL/Surface_mesh.h>
#include <CGAL/boost/graph/convert_nef_polyhedron_to_polygon_mesh.h>
#include <CGAL/Point_3.h>
#include <CGAL/Polyhedron_3.h>
#include <CGAL/Nef_polyhedron_3.h>
#include <CGAL/boost/graph/IO/STL.h>

#include <CGAL/Polygon_mesh_processing/triangulate_faces.h>

#include "wrapper.h"

using Kernel = CGAL::Cartesian<double>;
using Vector_3 = Kernel::Vector_3;
using Point_3 = Kernel::Point_3;
using Aff_transformation_3 = Kernel::Aff_transformation_3;
using Nef_3 = CGAL::Nef_polyhedron_3<Kernel>;
using Poly_3 = CGAL::Polyhedron_3<Kernel>;
using Mesh_3 = CGAL::Surface_mesh<Point_3>;

template <typename HDS>
class Cube : public CGAL::Modifier_base<HDS> {
public:
    double x, y, z;
    Cube(double x, double y, double z): x(x), y(y), z(z) {}
    void operator()( HDS& hds) {
        if (isnan(x) || isnan(y) || isnan(z)) {
            return;
        }

        CGAL::Polyhedron_incremental_builder_3<HDS> b{hds, true};

        b.begin_surface(8, 6, 6 * 4);

        b.add_vertex(Point_3{0, 0, 0});
        b.add_vertex(Point_3{x, 0, 0});
        b.add_vertex(Point_3{x, y, 0});
        b.add_vertex(Point_3{0, y, 0});
        b.add_vertex(Point_3{0, 0, z});
        b.add_vertex(Point_3{x, 0, z});
        b.add_vertex(Point_3{x, y, z});
        b.add_vertex(Point_3{0, y, z});

        std::array<size_t, 6 * 4> indices = {
            0, 1, 2, 3, // front
            4, 0, 3, 7, // left
            5, 4, 7, 6, // back
            1, 5, 6, 2, // right
            3, 2, 6, 7, // top
            4, 5, 1, 0, // bottom
        };

        b.begin_facet();
        for (size_t i = 0; i < indices.size(); i++) {
            if (i % 4 == 0 && i > 0) {
                b.end_facet();
                b.begin_facet();
            }
            b.add_vertex_to_facet(indices[i]);
        }
        b.end_facet();

        b.end_surface();
    }
};

char* strdup (const char* s) {
    size_t len = strlen(s);
    char* r = (char *)malloc(len + 1);
    if (r == nullptr) {
        return nullptr;
    }
    memcpy(r, s, len+1);
    return r;
}

template <typename R>
R protect(Error *err, std::function<R()> f) {
    *err = nullptr;

    try {
        return f();
    } catch (std::logic_error &e) {
        *err = strdup(e.what());
        return R();
    } catch (...) {
        *err = strdup("Unknown C++ exception");
        return R();
    }
}

extern "C" {

void error_free(Error err) {
    std::free(err);
}

Nef3Obj nef3_new_cube(double x, double y, double z, Error *err) {
    return protect<Nef3Obj>(err, [=]() {
        Poly_3 poly{};
        Cube<Poly_3::HalfedgeDS> cube{x, y, z};
        poly.delegate(cube);

        return reinterpret_cast<Nef3Obj>(new Nef_3{poly});
    });
}

Nef3Obj nef3_clone(Nef3Obj obj) {
    return reinterpret_cast<Nef3Obj>(new Nef_3{*reinterpret_cast<Nef_3 *>(obj)});
}

void nef3_union(Nef3Obj obj, Nef3Obj other, Error *err) {
    return protect<void>(err, [=]() {
        *reinterpret_cast<Nef_3 *>(obj) += *reinterpret_cast<Nef_3 *>(other);
    });
}

void nef3_difference(Nef3Obj obj, Nef3Obj other, Error *err) {
    return protect<void>(err, [=]() {
        *reinterpret_cast<Nef_3 *>(obj) -= *reinterpret_cast<Nef_3 *>(other);
    });
}

void nef3_intersection(Nef3Obj obj, Nef3Obj other, Error *err) {
    return protect<void>(err, [=]() {
        *reinterpret_cast<Nef_3 *>(obj) *= *reinterpret_cast<Nef_3 *>(other);
    });
}

void nef3_transform(Nef3Obj obj,const  double *m, Error *err) {
    return protect<void>(err, [=]() {
        Aff_transformation_3 aff{
            m[0], m[4], m[8], m[12],
            m[1], m[5], m[9], m[13],
            m[2], m[6], m[10], m[14],
        };
        reinterpret_cast<Nef_3 *>(obj)->transform(aff);
    });
}

void nef3_free(Nef3Obj obj) {
    delete reinterpret_cast<Nef_3 *>(obj);
}

MeshData *nef3_to_mesh_data(Nef3Obj obj, Error *err) {
    return protect<MeshData *>(err, [=]() {
        Nef_3 *nef = reinterpret_cast<Nef_3 *>(obj);

        Mesh_3 mesh;
        CGAL::convert_nef_polyhedron_to_polygon_mesh(*nef, mesh, true);

        //CGAL::IO::write_STL("/tmp/mesh.stl", mesh, CGAL::parameters::use_binary_mode(false));

        size_t vertex_len = mesh.num_faces() * 3 * 6;
        float *vertex_data = new float[vertex_len];
        float *vp = vertex_data;

        for (auto face : mesh.faces()) {
            auto he = mesh.halfedge(face);

            Point_3 p[3];
            p[0] = mesh.point(mesh.source(he));
            he = mesh.next(he);
            p[1] = mesh.point(mesh.source(he));
            he = mesh.next(he);
            p[2] = mesh.point(mesh.source(he));

            auto normal = CGAL::normal(p[0], p[1], p[2]);

            for (int i = 0; i < 3; i++) {
                *vp++ = p[i].x();
                *vp++ = p[i].y();
                *vp++ = p[i].z();
                *vp++ = normal.x();
                *vp++ = normal.y();
                *vp++ = normal.z();
            }
        }

        /*for (Mesh_3::size_type i = 0; i < mesh.num_vertices(); i++) {
            auto pt = mesh.point(Mesh_3::Vertex_index{i});
            *vp++ = pt.x();
            *vp++ = pt.y();
            *vp++ = pt.z();
        }*/

        /*size_t index_len = mesh.num_faces() * 3;
        uint16_t *index_data = new uint16_t[index_len];
        uint16_t *ip = index_data;

        for (auto face : mesh.faces()) {
            auto he = mesh.halfedge(face);

            for (int i = 0; i < 3; i++) {
                *ip++ = mesh.source(he);
                he = mesh.next(he);
            }
        }*/

        MeshData* data = new MeshData;
        data->vertex_len = vertex_len * sizeof(float);
        data->vertex_data = (uint8_t*)vertex_data;
        data->index_len = 0;
        data->index_data = nullptr;
        //data->index_len = index_len * sizeof(uint16_t);
        //data->index_data = (uint8_t*)index_data;
        data->stride = sizeof(float) * 6;

        return data;
    });
}

void mesh_data_free(MeshData *obj) {
    delete obj->vertex_data;
    //delete obj->index_data;
    delete obj;
}

}
