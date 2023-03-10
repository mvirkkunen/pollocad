#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MeshData {
    size_t vertex_len;
    uint8_t *vertex_data;
    size_t index_len;
    uint8_t *index_data;
    size_t stride;
} MeshData;

typedef enum BooleanOp {
    BOOLEAN_OP_UNION = 1,
    BOOLEAN_OP_DIFFERENCE = 2,
    BOOLEAN_OP_INTERSECTION = 3,
} BooleanOp;

typedef void *Nef3Obj;
typedef void *Mesh3Obj;
typedef char *Error;

void error_free(Error error);

Nef3Obj nef3_new_cube(double x, double y, double z, Error *err);
Nef3Obj nef3_clone(Nef3Obj obj);
void nef3_union(Nef3Obj obj, Nef3Obj other, Error *err);
void nef3_difference(Nef3Obj obj, Nef3Obj other, Error *err);
void nef3_intersection(Nef3Obj obj, Nef3Obj other, Error *err);
void nef3_transform(Nef3Obj obj, const double *matrix, Error *err);
void nef3_free(Nef3Obj obj);
MeshData *nef3_to_mesh_data(Nef3Obj obj, Error *err);

Mesh3Obj mesh3_new_from_data(
    const double *vertices,
    uint32_t num_vertices,
    const uint32_t *indices,
    uint32_t num_indices,
    Error *err);
Mesh3Obj mesh3_new_cube(double x, double y, double z, Error *err);
Mesh3Obj mesh3_clone(Mesh3Obj obj);
void mesh3_free(Mesh3Obj obj);
void mesh3_boolean_op(
    Mesh3Obj obj,
    Mesh3Obj other,
    BooleanOp op,
    uint8_t* nef_fallback,
    Error *err);
void mesh3_transform(Mesh3Obj obj, const double *matrix, Error *err);
MeshData *mesh3_to_mesh_data(Mesh3Obj obj, Error *err);

void mesh_data_free(MeshData *obj);

#ifdef __cplusplus
}
#endif
