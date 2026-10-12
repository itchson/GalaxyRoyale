#version 300 es

precision highp float;
precision highp int;

struct Affine {
    vec4 a;
    vec4 b;
};
struct TransformTable {
    Affine values[128];
};
struct Light {
    vec4 position_radius;
    vec4 color;
};
struct Globals {
    vec4 camera;
    vec4 ambient;
    Light lights[8];
    vec4 raster;
};
struct ObjectTable {
    uvec4 pairs[64];
};
struct Caster {
    vec4 x;
    vec4 y;
    uvec4 field;
};
struct ShadowDescriptors {
    uvec4 header;
    Caster casters[8];
};
struct ShadowMasks {
    uvec4 words[1024];
};
struct VertexOut {
    vec4 position;
    vec4 color;
    vec2 cell;
    vec2 material;
    vec2 particle_uv;
    float coverage;
    vec4 surface;
    float mapped;
    vec2 object;
};
layout(std140) uniform Globals_block_0Vertex { Globals _group_0_binding_0_vs; };

layout(std140) uniform TransformTable_block_1Vertex { TransformTable _group_0_binding_1_vs; };

layout(std140) uniform ObjectTable_block_2Vertex { ObjectTable _group_0_binding_2_vs; };

layout(location = 0) in vec2 _p2vs_location0;
layout(location = 1) in uint _p2vs_location1;
layout(location = 2) in uint _p2vs_location2;
smooth out vec4 _vs2fs_location0;
flat out vec2 _vs2fs_location1;
flat out vec2 _vs2fs_location2;
smooth out vec2 _vs2fs_location3;
flat out float _vs2fs_location4;
flat out vec4 _vs2fs_location5;
flat out float _vs2fs_location6;
flat out vec2 _vs2fs_location7;

vec2 sign_not_zero(vec2 value) {
    return mix(vec2(-1.0), vec2(1.0), greaterThanEqual(value, vec2(0.0)));
}

vec3 decode_normal(uint bits) {
    vec3 normal = vec3(0.0);
    vec2 xy = (((vec2(float(((bits >> 7u) & 255u)), float(((bits >> 15u) & 255u))) / vec2(255.0)) * 2.0) - vec2(1.0));
    normal = vec3(xy, ((1.0 - abs(xy.x)) - abs(xy.y)));
    float _e30 = normal.z;
    if ((_e30 < 0.0)) {
        vec2 _e38 = sign_not_zero(xy);
        float _e41 = normal.z;
        normal = vec3(((vec2(1.0) - abs(xy.yx)) * _e38), _e41);
    }
    vec3 _e43 = normal;
    return normalize(_e43);
}

vec3 surface_normal(vec3 local_8, Affine t) {
    float magnitude = max(max(abs(t.a.z), abs(t.a.w)), max(abs(t.b.x), abs(t.b.y)));
    if ((magnitude == 0.0)) {
        return vec3(0.0, 0.0, 1.0);
    }
    vec2 x = (t.a.zw / vec2(magnitude));
    vec2 y = (t.b.xy / vec2(magnitude));
    float determinant_ = ((x.x * y.y) - (x.y * y.x));
    if ((abs(determinant_) < 1e-12)) {
        return vec3(0.0, 0.0, 1.0);
    }
    float factor = (((determinant_ >= 0.0) ? 1.0 : -1.0) / sqrt(abs(determinant_)));
    vec2 xy_1 = (vec2(((y.y * local_8.x) - (x.y * local_8.y)), ((-(y.x) * local_8.x) + (x.x * local_8.y))) * factor);
    return normalize(vec3(xy_1, local_8.z));
}

void main() {
    uint index = uint(gl_VertexID);
    vec2 center = _p2vs_location0;
    uint rgba = _p2vs_location1;
    uint slot = _p2vs_location2;
    vec2 world = vec2(0.0);
    float coverage = 1.0;
    VertexOut out_ = VertexOut(vec4(0.0), vec4(0.0), vec2(0.0), vec2(0.0), vec2(0.0), 0.0, vec4(0.0), 0.0, vec2(0.0));
    vec2 corners[6] = vec2[6](vec2(-0.5, -0.5), vec2(0.5, -0.5), vec2(-0.5, 0.5), vec2(-0.5, 0.5), vec2(0.5, -0.5), vec2(0.5, 0.5));
    Affine t_1 = _group_0_binding_1_vs.values[(slot & 127u)];
    vec2 local_9 = (center + corners[index]);
    world = ((t_1.a.xy + (t_1.a.zw * local_9.x)) + (t_1.b.xy * local_9.y));
    if ((t_1.b.w < 0.0)) {
        vec4 _e52 = _group_0_binding_0_vs.raster;
        vec4 _e56 = _group_0_binding_0_vs.camera;
        vec2 raster_scale = (_e52.xy / _e56.zw);
        vec2 x_1 = (t_1.a.zw * raster_scale);
        vec2 y_1 = (t_1.b.xy * raster_scale);
        float cxx = ((x_1.x * x_1.x) + (y_1.x * y_1.x));
        float cyy = ((x_1.y * x_1.y) + (y_1.y * y_1.y));
        float cxy = ((x_1.x * x_1.y) + (y_1.x * y_1.y));
        float area = abs(((x_1.x * y_1.y) - (x_1.y * y_1.x)));
        float largest = (0.5 * ((cxx + cyy) + sqrt((((cxx - cyy) * (cxx - cyy)) + ((4.0 * cxy) * cxy)))));
        float smallest = ((area * area) / max(largest, 1e-20));
        if ((smallest < 4.0)) {
            float variance = (4.0 - smallest);
            float xx = (cxx + variance);
            float yy = (cyy + variance);
            float expanded_area = sqrt(max((((area * area) + (variance * (cxx + cyy))) + (variance * variance)), 1e-20));
            float denominator = sqrt(max(((xx + yy) + (2.0 * expanded_area)), 1e-20));
            vec2 bx = (vec2((xx + expanded_area), cxy) / vec2(denominator));
            vec2 by = (vec2(cxy, (yy + expanded_area)) / vec2(denominator));
            vec2 offset = ((bx * corners[index].x) + (by * corners[index].y));
            vec2 cell = ((t_1.a.xy + (t_1.a.zw * center.x)) + (t_1.b.xy * center.y));
            world = (cell + (offset / raster_scale));
            coverage = clamp((area / expanded_area), 0.0, 1.0);
        }
    }
    vec2 _e165 = world;
    vec4 _e168 = _group_0_binding_0_vs.camera;
    vec4 _e173 = _group_0_binding_0_vs.camera;
    vec2 screen = (((_e165 - _e168.xy) / _e173.zw) * 2.0);
    out_.position = vec4(screen.x, -(screen.y), 0.0, 1.0);
    vec4 color_1 = (vec4(float((rgba & 255u)), float(((rgba >> 8u) & 255u)), float(((rgba >> 16u) & 255u)), float(((rgba >> 24u) & 255u))) / vec4(255.0));
    out_.color = vec4(pow(color_1.xyz, vec3(2.2)), color_1.w);
    out_.cell = ((t_1.a.xy + (t_1.a.zw * center.x)) + (t_1.b.xy * center.y));
    out_.material = t_1.b.zw;
    uvec4 pair = _group_0_binding_2_vs.pairs[((slot & 127u) / 2u)];
    uint parity = ((slot & 1u) * 2u);
    out_.object = vec2(uintBitsToFloat(pair[parity]), float((pair[(parity + 1u)] & 1u)));
    out_.particle_uv = (corners[index] * 2.0);
    float _e258 = coverage;
    out_.coverage = _e258;
    out_.surface = vec4(0.0, 0.0, 1.0, 1.0);
    out_.mapped = 0.0;
    if (((slot & 2147483648u) != 0u)) {
        vec3 _e272 = decode_normal(slot);
        vec3 _e273 = surface_normal(_e272, t_1);
        out_.surface = vec4(_e273, (float(((slot >> 23u) & 255u)) / 255.0));
        out_.mapped = 1.0;
    }
    VertexOut _e284 = out_;
    gl_Position = _e284.position;
    _vs2fs_location0 = _e284.color;
    _vs2fs_location1 = _e284.cell;
    _vs2fs_location2 = _e284.material;
    _vs2fs_location3 = _e284.particle_uv;
    _vs2fs_location4 = _e284.coverage;
    _vs2fs_location5 = _e284.surface;
    _vs2fs_location6 = _e284.mapped;
    _vs2fs_location7 = _e284.object;
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
    return;
}

