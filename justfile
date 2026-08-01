android_project := "./artemis-android"

jni_dir := android_project / "app/src/main/jniLibs"
bindings_out := android_project / "app/src/main/java"

arm64_lib := "target/aarch64-linux-android/release/libartemis.so"
x86_64_lib := "target/x86_64-linux-android/release/libartemis.so"

default:
    @just --list

build-arm64:
    cargo ndk -t arm64-v8a build --release

build-x86_64:
    cargo ndk -t x86_64 build --release

build: build-arm64 build-x86_64

copy-arm64: build-arm64
    mkdir -p {{jni_dir}}/arm64-v8a
    cp {{arm64_lib}} {{jni_dir}}/arm64-v8a/

copy-x86_64: build-x86_64
    mkdir -p {{jni_dir}}/x86_64
    cp {{x86_64_lib}} {{jni_dir}}/x86_64/

copy: copy-arm64 copy-x86_64

bindings: build-arm64
    cargo run --release --bin uniffi-bindgen generate \
        --library {{arm64_lib}} \
        --language kotlin \
        --out-dir {{bindings_out}}

pack: copy bindings
    @echo " Android package ready."
