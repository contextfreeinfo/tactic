#pragma once

// TODO We need to enforce layouts for all these structs.

typedef struct taca_Vec4 {
    float x, y, z, w;
} taca_Vec4;

typedef struct taca_Rgba {
    float r, g, b, a;
} taca_Rgba;

typedef const void* taca_Attribute;

typedef struct taca_ShaderInput {
    void* uniforms;
    taca_Attribute* attributes;
} taca_ShaderInput;

typedef void (*taca_FragmentShader)(
    const taca_ShaderInput* input,
    taca_Rgba* output
);

typedef void (*taca_VertexShader)(
    const taca_ShaderInput* input,
    void* output
);

typedef struct taca_AttributeLayout {
    taca_Attribute start;
    size_t stride;
} taca_AttributeLayout;

typedef struct taca_PipelineData {
    const void* uniforms;
    const taca_AttributeLayout* attributes;
    size_t attributeCount;
    size_t vertexCount;
} taca_PipelineData;

typedef struct taca_Pipeline {
    // TODO Error handling here also?
    taca_FragmentShader fragment;
    taca_VertexShader vertex;
} taca_Pipeline;

taca_EXPORT void taca_draw(
    const taca_Pipeline* pipeline,
    const taca_PipelineData* data
);
