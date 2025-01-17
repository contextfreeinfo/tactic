pub() {
    for dir in dist public; do
        pub_dir=../../../web/$dir/apps/$2
        mkdir -p $pub_dir && \
        cp $1 $pub_dir/
    done
}

shader-build() {
    naga src/$1.glsl out/$1.spv && \
    xxd -i out/$1.spv > out/$1.c
    # The fragment shader seems to bigger after opt for some reason.
    # spirv-opt -Os out/$1.spv -o out/$1.opt.spv && \
    # xxd -i out/$1.opt.spv > out/$1.c
}

rm -rf out && \
mkdir -p out/bundle && \
shader-build shader.frag && \
shader-build shader.vert && \
xxd -i src/musicbox.ogg > out/musicbox-data.c && \
"$WASI_SDK/bin/clang++" --std=c++23 -Os -s -Wall -Wextra -Werror -Isrc -Iout \
     -Wno-missing-field-initializers -Wno-unused-variable \
     -Wno-unused-parameter -fno-exceptions \
     -o out/bundle/app.wasm src/main.cpp && \
(cd out/bundle && zip -r ../music.taca .) && \
ls -l out/*.taca && \
pub out/music.taca cpp

# && \
# wasm2wat --generate-names out/music.wasm -o out/music.wat
# wasm-opt -Os out/music.wasm -o out/music.opt.wasm && \
# "$WASI_SDK/bin/clang++" --std=c++23 -fmodules -o out/music.wasm src/main.cpp -x c++-module src/taca.cpp && \
# wit-bindgen c --out-dir out --no-helpers --no-object-file --rename-world taca src/taca.wit && \

# ffmpeg -i src/musicbox.ogg -c:a libvorbis src/musicbox.webm
# ffmpeg -i src/musicbox.ogg -c:a libopus -b:a 64k -movflags +faststart src/musicbox.webm
