import { fail } from "./util";

export type LazyTexture = Partial<Texture>;

export interface Texture {
  // TODO Also store a baseline for all textures that for non-text is y size.
  size: [number, number];
  texture: WebGLTexture;
  usedSize: [number, number];
}

export class TexturePipeline {
  constructor(gl: WebGL2RenderingContext) {
    this.gl = gl;
    const program = shaderProgramBuild(
      gl,
      textureSourceVert,
      textureSourceFrag
    );
    this.program = program;
    this.drawInfoBuffer = gl.createBuffer() ?? fail();
    this.drawInfoIndex = gl.getUniformBlockIndex(program, "drawInfo") ?? fail();
    gl.uniformBlockBinding(program, this.drawInfoIndex, textureArrayBinding);
    this.sampler = gl.getUniformLocation(program, "sampler") ?? fail();
    const vertexArray = gl.createVertexArray() ?? fail();
    this.vertexArray = vertexArray;
    const indices = new Uint16Array([0, 1, 2, 0, 2, 3]);
    gl.bindVertexArray(vertexArray);
    // Vertex position and tex coords.
    [
      [1, 1, -1, 1, -1, -1, 1, -1],
      [1, 1, 0, 1, 0, 0, 1, 0],
    ].forEach((array, i) => {
      const buffer = gl.createBuffer() ?? fail();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(array), gl.STATIC_DRAW);
      gl.vertexAttribPointer(i, 2, gl.FLOAT, false, 0, 0);
      gl.enableVertexAttribArray(i);
    });
    // Index.
    const indexBuffer = gl.createBuffer() ?? fail();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
    // Done.
    gl.bindVertexArray(null);
  }

  draw(
    texture: WebGLTexture,
    canvasWidth: number,
    canvasHeight: number,
    x: number,
    y: number,
    size: [number, number],
    usedSize: [number, number]
  ) {
    const { drawInfoBuffer, gl, program, sampler, vertexArray } = this;
    const drawInfoArray = new Float32Array([
      canvasWidth,
      canvasHeight,
      x,
      y,
      usedSize[0],
      usedSize[1],
      size[0],
      size[1],
    ]);
    gl.useProgram(program);
    gl.bindBuffer(gl.UNIFORM_BUFFER, drawInfoBuffer);
    gl.bufferData(gl.UNIFORM_BUFFER, drawInfoArray, gl.STREAM_DRAW);
    gl.bindBufferBase(gl.UNIFORM_BUFFER, textureArrayBinding, drawInfoBuffer);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.uniform1i(sampler, 0);
    // For fixed system things, vertex array objects are probably fine.
    gl.bindVertexArray(vertexArray);
    try {
      gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);
    } finally {
      gl.bindVertexArray(null);
    }
  }

  drawInfoBuffer: WebGLBuffer;
  drawInfoIndex: number; // TODO Need to use this?
  gl: WebGL2RenderingContext;
  program: WebGLProgram;
  sampler: WebGLUniformLocation;
  vertexArray: WebGLVertexArrayObject;
}

export function fragmentMunge(glsl: string) {
  // This helps flip the y axis to match wgpu.
  // TODO Instead render to texture then flip the texture.
  const bonus = [
    "struct taca_uniform_struct { vec2 size; };",
    "uniform taca_uniform_block { taca_uniform_struct taca; };",
  ].join("\n");
  glsl = glsl.replace(/((?:^precision [^;]+;\n){2})/m, `$1${bonus}\n`);
  const inverted =
    "vec4(gl_FragCoord.x, taca.size.y - gl_FragCoord.y, gl_FragCoord.zw)";
  glsl = glsl.replace(/gl_FragCoord/g, inverted);
  return glsl;
}

export function imageDecode(
  gl: WebGL2RenderingContext,
  bytes: Uint8Array,
  fulfill: () => void,
  reject: (reason: any) => void
): Texture {
  const header = new DataView(bytes.buffer, bytes.byteOffset, 4);
  const type =
    header.getUint32(0) == 0x89504e47
      ? "png"
      : header.getInt16(0) == 0xffd8
      ? "jpeg"
      : fail();
  const blob = new Blob([bytes], { type: `image/${type}` });
  // Based on https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
  const texture = gl.createTexture() ?? fail();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    1,
    1,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    new Uint8Array([0, 0, 0, 0])
  );
  const lazyTexture: Texture = {
    size: [1, 1],
    texture,
    usedSize: [1, 1],
  };
  createImageBitmap(blob).then(
    (bitmap) => {
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        bitmap
      );
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      lazyTexture.size = lazyTexture.usedSize = [bitmap.width, bitmap.height];
      fulfill();
    },
    (reason) => reject(reason)
  );
  return lazyTexture;
}

export function shaderMunge(glsl: string) {
  // TODO Check both vertex & fragment at the same time to see if they match?
  glsl = glsl.replace(
    /^(uniform \S+\d+)(?:Vertex|Fragment)( \{ \S+ _group_\d+_binding_\d+)/m,
    "$1VertexFragment$2"
  );
  glsl = glsl.replaceAll(/\b(_group_\d+_binding_\d+_)[vf]s\b/g, "$1vfs");
  return glsl;
}

export function shaderProgramBuild(
  gl: WebGL2RenderingContext,
  vertex: string,
  fragment: string
) {
  const program = gl.createProgram() ?? fail();
  const addShader = (type: number, source: string) => {
    const shader = gl.createShader(type) ?? fail();
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    gl.getShaderParameter(shader, gl.COMPILE_STATUS) ??
      fail(gl.getShaderInfoLog(shader));
    gl.attachShader(program, shader);
  };
  addShader(gl.VERTEX_SHADER, vertex);
  addShader(gl.FRAGMENT_SHADER, fragment);
  gl.linkProgram(program);
  gl.getProgramParameter(program, gl.LINK_STATUS) ??
    fail(gl.getProgramInfoLog(program));
  return program;
}

const textureArrayBinding = 0;

const textureSourceFrag = `#version 300 es
precision mediump float;
in vec2 vTexCoord;
out vec4 outColor;
uniform drawInfo {
  vec2 canvasSize;
  vec2 drawPos;
  vec2 drawSize;
  vec2 textureSize;
};
uniform sampler2D sampler;
void main() {
  outColor = texture(sampler, vTexCoord * drawSize / textureSize);
}
`;

const textureSourceVert = `#version 300 es
precision mediump float;
layout(location = 0) in vec2 framePos;
layout(location = 1) in vec2 texCoord;
uniform drawInfo {
  vec2 canvasSize;
  vec2 drawPos;
  vec2 drawSize;
  vec2 textureSize;
};
out vec2 vTexCoord;
void main() {
  vec2 pos = framePos * drawSize * 0.5 + drawPos;
  pos.y = canvasSize.y - pos.y;
  pos = (pos / canvasSize) * 2.0 - 1.0;
  gl_Position = vec4(pos, 0.0, 1.0);
  vTexCoord = texCoord;
}
`;
