#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <functional>
#include <memory>

#include <CGAL/Cartesian.h>
#include <CGAL/Surface_mesh.h>
#include <CGAL/boost/graph/convert_nef_polyhedron_to_polygon_mesh.h>
#include <CGAL/Point_3.h>
#include <CGAL/Polyhedron_3.h>
#include <CGAL/Nef_polyhedron_3.h>
#include <CGAL/boost/graph/IO/STL.h>
#include <CGAL/Polygon_mesh_processing/corefinement.h>
#include <CGAL/Polygon_mesh_processing/transform.h>
#include <CGAL/Polygon_mesh_processing/triangulate_faces.h>

#include "wrapper.h"

namespace PMP = CGAL::Polygon_mesh_processing;

using Kernel = CGAL::Cartesian<double>;
using Point_3 = Kernel::Point_3;
using Vector_3 = Kernel::Vector_3;
using Plane_3 = CGAL::Plane_3<Kernel>;
using Aff_transformation_3 = Kernel::Aff_transformation_3;
using Poly_3 = CGAL::Polyhedron_3<Kernel>;
using Nef_3 = CGAL::Nef_polyhedron_3<Kernel>;
using Mesh_3 = CGAL::Surface_mesh<Point_3>;

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

using VI = Mesh_3::Vertex_index;

Mesh3Obj mesh3_new_from_data(
    const double *vertices,
    uint32_t num_vertices,
    const uint32_t *indices,
    uint32_t num_indices,
    Error *err)
{
    return protect<Mesh3Obj>(err, [=]() {
        auto mesh = new Mesh_3;

        mesh->reserve(num_vertices, num_indices, num_indices / 3);

        for (uint32_t i = 0; i < num_vertices; i += 3) {
            mesh->add_vertex(Point_3{vertices[i], vertices[i + 1], vertices[i + 2]});
        }

        for (uint32_t i = 0; i < num_indices; i += 3) {
            mesh->add_face(
                Mesh_3::Vertex_index{indices[i]},
                Mesh_3::Vertex_index{indices[i + 1]},
                Mesh_3::Vertex_index{indices[i + 2]});
        }

        //PMP::triangulate_faces(*mesh);

        //PMP::orient(*mesh);

        return reinterpret_cast<Mesh3Obj>(mesh);
    });
}

Mesh3Obj mesh3_clone(Mesh3Obj obj) {
    return reinterpret_cast<Mesh3Obj>(new Mesh_3{*reinterpret_cast<Mesh_3 *>(obj)});
}

void mesh3_free(Mesh3Obj obj) {
    delete reinterpret_cast<Mesh_3 *>(obj);
}

void mesh3_boolean_op(
    Mesh3Obj obj,
    Mesh3Obj other,
    BooleanOp op,
    uint8_t* nef_fallback,
    Error *err)
{
    return protect<void>(err, [=]() {
        auto mesh = reinterpret_cast<Mesh_3 *>(obj);
        auto other_mesh = reinterpret_cast<Mesh_3 *>(other);
        *nef_fallback = 0;

        switch (op) {
            case BOOLEAN_OP_UNION:
                if (!PMP::corefine_and_compute_union(*mesh, *other_mesh, *mesh)) {
                    *nef_fallback = true;
                    auto nef = Nef_3{*mesh} + Nef_3{*other_mesh};
                    CGAL::convert_nef_polyhedron_to_polygon_mesh(nef, *mesh, true);
                }
                break;

            case BOOLEAN_OP_DIFFERENCE:
                if (!PMP::corefine_and_compute_difference(*mesh, *other_mesh, *mesh)) {
                    *nef_fallback = true;
                    auto nef = Nef_3{*mesh} - Nef_3{*other_mesh};
                    CGAL::convert_nef_polyhedron_to_polygon_mesh(nef, *mesh, true);
                }
                break;

            case BOOLEAN_OP_INTERSECTION:
                if (!PMP::corefine_and_compute_intersection(*mesh, *other_mesh, *mesh)) {
                    *nef_fallback = true;
                    auto nef = Nef_3{*mesh} * Nef_3{*other_mesh};
                    CGAL::convert_nef_polyhedron_to_polygon_mesh(nef, *mesh, true);
                }
                break;
        }
    });
}

void mesh3_transform(Mesh3Obj obj, const double *m, Error *err) {
    return protect<void>(err, [=]() {
        Aff_transformation_3 aff{
            m[0], m[4], m[8], m[12],
            m[1], m[5], m[9], m[13],
            m[2], m[6], m[10], m[14],
        };

        PMP::transform(aff, *reinterpret_cast<Mesh_3 *>(obj));
    });
}

static MeshData *mesh3_to_mesh_data_impl(Mesh_3 &mesh) {
    size_t vertex_len = mesh.number_of_faces() * 3 * 6;
    float *vertex_data = new float[vertex_len];
    float *vp = vertex_data;

    /*fprintf(
        stderr,
        "faces: %d verts: %d edge: %d halfedge: %d\n",
        mesh.number_of_faces(),
        mesh.number_of_vertices(),
        mesh.number_of_edges(),
        mesh.number_of_halfedges());*/

    for (auto face : mesh.faces()) {
        if (mesh.is_removed(face)) continue;

        auto he = mesh.halfedge(face);

        Point_3 p[3];
        p[0] = mesh.point(mesh.source(he));
        he = mesh.next(he);
        p[1] = mesh.point(mesh.source(he));
        he = mesh.next(he);
        p[2] = mesh.point(mesh.source(he));

        auto normal = CGAL::unit_normal(p[0], p[1], p[2]);

        for (int i = 0; i < 3; i++) {
            *vp++ = p[i].x();
            *vp++ = p[i].y();
            *vp++ = p[i].z();
            *vp++ = normal.x();
            *vp++ = normal.y();
            *vp++ = normal.z();

            //printf("%f %f %f ", p[i].x(), p[i].y(), p[i].z());
            //printf("(%f %f %f) ", normal.x(), normal.y(), normal.z());
        }

        //printf("\n");
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
}

MeshData *mesh3_to_mesh_data(Mesh3Obj obj, Error *err) {
    return protect<MeshData *>(err, [=]() {
        auto mesh = Mesh_3{*reinterpret_cast<Mesh_3 *>(obj)};

        //CGAL::IO::write_STL("/tmp/mesh.stl", mesh, CGAL::parameters::use_binary_mode(false));
        return mesh3_to_mesh_data_impl(mesh);
    });
}

MeshData *nef3_to_mesh_data(Nef3Obj obj, Error *err) {
    return protect<MeshData *>(err, [=]() {
        Nef_3 *nef = reinterpret_cast<Nef_3 *>(obj);

        Mesh_3 mesh;
        CGAL::convert_nef_polyhedron_to_polygon_mesh(*nef, mesh, true);

        return mesh3_to_mesh_data_impl(mesh);
    });
}

void mesh_data_free(MeshData *obj) {
    delete obj->vertex_data;
    //delete obj->index_data;
    delete obj;
}

}
