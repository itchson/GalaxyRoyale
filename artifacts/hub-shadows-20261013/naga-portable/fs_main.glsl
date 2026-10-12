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
layout(std140) uniform Globals_block_0Fragment { Globals _group_0_binding_0_fs; };

layout(std140) uniform ShadowDescriptors_block_1Fragment { ShadowDescriptors _group_0_binding_3_fs; };

layout(std140) uniform ShadowMasks_block_2Fragment { ShadowMasks _group_0_binding_4_fs; };

smooth in vec4 _vs2fs_location0;
flat in vec2 _vs2fs_location1;
flat in vec2 _vs2fs_location2;
smooth in vec2 _vs2fs_location3;
flat in float _vs2fs_location4;
flat in vec4 _vs2fs_location5;
flat in float _vs2fs_location6;
flat in vec2 _vs2fs_location7;
layout(location = 0) out vec4 _fs2p_location0;

float blocked_ray(vec2 receiver, float receiver_height, vec2 source, float source_height) {
    float blocked = 0.0;
    uint c = 0u;
    bool local_3 = false;
    bool local_4 = false;
    bool local_5 = false;
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e117 = c;
            c = (_e117 + 1u);
        }
        loop_init = false;
        uint _e8 = c;
        uint _e12 = _group_0_binding_3_fs.header.x;
        if ((_e8 < min(_e12, 8u))) {
        } else {
            break;
        }
        {
            uint _e18 = c;
            Caster caster = _group_0_binding_3_fs.casters[_e18];
            if (!((caster.field.w == 0u))) {
                local_3 = (receiver_height >= caster.x.w);
            } else {
                local_3 = true;
            }
            bool _e32 = local_3;
            if (!(_e32)) {
                local_4 = (caster.x.w >= source_height);
            } else {
                local_4 = true;
            }
            bool _e40 = local_4;
            if (_e40) {
                continue;
            }
            float fraction = ((caster.x.w - receiver_height) / max((source_height - receiver_height), 1e-6));
            vec2 point = mix(receiver, source, clamp(fraction, 0.0, 1.0));
            vec2 local_9 = vec2((dot(caster.x.xy, point) + caster.x.z), (dot(caster.y.xy, point) + caster.y.z));
            vec2 cell = floor((local_9 + (vec2(caster.field.xy) * 0.5)));
            if (!(any(lessThan(cell, vec2(0.0))))) {
                local_5 = any(greaterThanEqual(cell, vec2(caster.field.xy)));
            } else {
                local_5 = true;
            }
            bool _e85 = local_5;
            if (_e85) {
                continue;
            }
            uint bit = ((caster.field.z + (uint(cell.y) * 128u)) + uint(cell.x));
            uint word = (bit / 32u);
            if ((word >= 4096u)) {
                continue;
            }
            uint _e108 = _group_0_binding_4_fs.words[(word / 4u)][(word % 4u)];
            if (((_e108 & (1u << (bit % 32u))) != 0u)) {
                blocked = 1.0;
            }
        }
    }
    float _e120 = blocked;
    return _e120;
}

float shadow_visibility(VertexOut input_1, Light light, uint index_1) {
    bool local_6 = false;
    bool local_7 = false;
    float blocked_1 = 0.0;
    uint tap = 0u;
    if (!((input_1.object.y < 0.5))) {
        uint _e13 = _group_0_binding_3_fs.header.x;
        local_6 = (_e13 == 0u);
    } else {
        local_6 = true;
    }
    bool _e17 = local_6;
    if (!(_e17)) {
        uint _e24 = _group_0_binding_3_fs.header.y;
        local_7 = ((_e24 & (1u << index_1)) == 0u);
    } else {
        local_7 = true;
    }
    bool _e31 = local_7;
    if (_e31) {
        return 1.0;
    }
    vec2 offsets[4] = vec2[4](vec2(-1.5, -1.5), vec2(1.5, -1.5), vec2(-1.5, 1.5), vec2(1.5, 1.5));
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            uint _e66 = tap;
            tap = (_e66 + 1u);
        }
        loop_init_1 = false;
        uint _e50 = tap;
        if ((_e50 < 4u)) {
        } else {
            break;
        }
        {
            float _e53 = blocked_1;
            uint _e59 = tap;
            float _e64 = blocked_ray(input_1.cell, input_1.object.x, (light.position_radius.xy + offsets[_e59]), light.color.w);
            blocked_1 = (_e53 + _e64);
        }
    }
    float _e69 = blocked_1;
    return (1.0 - (_e69 * 0.25));
}

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
    VertexOut input_ = VertexOut(gl_FragCoord, _vs2fs_location0, _vs2fs_location1, _vs2fs_location2, _vs2fs_location3, _vs2fs_location4, _vs2fs_location5, _vs2fs_location6, _vs2fs_location7);
    vec3 color = vec3(0.0);
    bool local = false;
    float contact = 0.0;
    bool local_1 = false;
    uint i = 0u;
    uint i_1 = 0u;
    float directional = 0.0;
    bool local_2 = false;
    float alpha = 0.0;
    float response = input_.material.x;
    float object_emission = ((input_.material.y < 0.0) ? max(0.0, (-(input_.material.y) - 1.0)) : input_.material.y);
    float emission = (object_emission * input_.surface.w);
    color = (input_.color.xyz * ((1.0 - (0.12 * response)) + (0.3 * emission)));
    if ((input_.object.y > 0.5)) {
        uint _e40 = _group_0_binding_3_fs.header.x;
        local = (_e40 > 0u);
    } else {
        local = false;
    }
    bool receiver_1 = local;
    if (receiver_1) {
        uint _e52 = _group_0_binding_3_fs.header.w;
        local_1 = (_e52 != 0u);
    } else {
        local_1 = false;
    }
    bool _e56 = local_1;
    if (_e56) {
        bool loop_init_2 = true;
        while(true) {
            if (!loop_init_2) {
                uint _e83 = i;
                i = (_e83 + 1u);
            }
            loop_init_2 = false;
            uint _e59 = i;
            if ((_e59 < 8u)) {
            } else {
                break;
            }
            {
                uint _e65 = _group_0_binding_3_fs.header.y;
                uint _e67 = i;
                if (((_e65 & (1u << _e67)) != 0u)) {
                    float _e72 = contact;
                    uint _e75 = i;
                    Light _e77 = _group_0_binding_0_fs.lights[_e75];
                    uint _e78 = i;
                    float _e79 = shadow_visibility(input_, _e77, _e78);
                    contact = max(_e72, (1.0 - _e79));
                }
            }
        }
        float _e93 = contact;
        color = (((input_.color.xyz * (1.0 - (0.12 * response))) * (1.0 - (0.18 * _e93))) + (input_.color.xyz * (0.3 * emission)));
    }
    if ((response > 0.0)) {
        vec4 _e111 = _group_0_binding_0_fs.ambient;
        float _e116 = _group_0_binding_0_fs.ambient.w;
        vec3 grade = mix(vec3(1.0), _e111.xyz, clamp((_e116 * response), 0.0, 1.0));
        if (receiver_1) {
            vec3 _e122 = color;
            color = (((_e122 - (input_.color.xyz * (0.3 * emission))) * grade) + (input_.color.xyz * (0.3 * emission)));
        } else {
            vec3 _e136 = color;
            color = (_e136 * grade);
        }
        bool loop_init_3 = true;
        while(true) {
            if (!loop_init_3) {
                uint _e217 = i_1;
                i_1 = (_e217 + 1u);
            }
            loop_init_3 = false;
            uint _e140 = i_1;
            if ((_e140 < 8u)) {
            } else {
                break;
            }
            {
                uint _e145 = i_1;
                Light light_1 = _group_0_binding_0_fs.lights[_e145];
                vec2 delta = (input_.cell - light_1.position_radius.xy);
                float radius = max(1.0, light_1.position_radius.z);
                float attenuation = max(0.0, (1.0 - (dot(delta, delta) / (radius * radius))));
                directional = 1.0;
                if ((input_.mapped > 0.0)) {
                    local_2 = (light_1.color.w > 0.0);
                } else {
                    local_2 = false;
                }
                bool _e175 = local_2;
                if (_e175) {
                    vec3 toward_light = normalize(vec3((light_1.position_radius.xy - input_.cell), max(0.0001, (light_1.color.w - input_.object.x))));
                    directional = max(0.0, dot(input_.surface.xyz, toward_light));
                }
                float _e199 = directional;
                uint _e201 = i_1;
                float _e202 = shadow_visibility(input_, light_1, _e201);
                float energy = (((((attenuation * attenuation) * light_1.position_radius.w) * response) * _e199) * _e202);
                vec3 _e204 = color;
                color = (_e204 + ((light_1.color.xyz * ((input_.color.xyz * 0.8) + vec3(0.012))) * energy));
            }
        }
    }
    alpha = input_.color.w;
    if ((input_.material.y < 0.0)) {
        float radial = max(0.0, (1.0 - dot(input_.particle_uv, input_.particle_uv)));
        float _e234 = alpha;
        alpha = (((_e234 * radial) * radial) * input_.coverage);
    }
    vec3 _e239 = color;
    float _e243 = alpha;
    _fs2p_location0 = vec4(min(_e239, vec3(1.0)), _e243);
    return;
}

