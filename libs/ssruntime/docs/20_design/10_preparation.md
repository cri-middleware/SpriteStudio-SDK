> [日本語](./10_preparation.ja.md) | **English**

# Preliminary Checks and Project Setup

Before you start writing a Player, check that the Brain can be used on your target engine, and prepare an empty project that does no more than build.

## 1. Check feasibility

The Brain is a C-API library written in Rust, and its per-frame output is a FlatBuffers buffer. Check the following three points for your target engine and language.

1. **Can Rust build for the platform?** Check that your target CPU architecture and platform are listed in [Rust's platform support](https://doc.rust-lang.org/beta/rustc/platform-support.html). If the SDK release has no library for that platform, build it yourself ([no_std Integration §1](./40_no_std_integration.md#1-build-configuration)).
2. **Can the engine's language call C functions (or WebAssembly)?** Check that the engine can load a native library or WebAssembly, and that its language can bind and call the functions.
3. **Does FlatBuffers code generation support the language?** Check [the languages FlatBuffers supports](https://flatbuffers.dev/support/).

## 2. Prepare the project

1. Create an empty project that builds, in your target engine and language.
2. Obtain `flatc`, and generate reading code for your language from the SDK schemas (`libs/ssab/fbs/` and `libs/ssruntime/fbs/`). Add the generated code and the FlatBuffers runtime library to the project.

    ```bash
    # For C++
    flatc --cpp -o ./src/gen/ libs/ssab/fbs/*.fbs
    flatc --cpp -o ./src/gen/ libs/ssruntime/fbs/*.fbs
    ```

3. Place the `libssruntime` static library (`.lib` / `.a`) or dynamic library (`.dll` / `.so` / `.dylib`) in the project and link it. When you use it from C / C++, add the header `ssruntime.h` to the include path.

## 3. Prepare a test ssab

1. Get `ssconverter-cli`.
2. Convert the SDK's `tests/overall/overall.sspj` to ssab. This project has an animation for each feature ([“Implementation steps” in the contents](../README.md#1-implementation-steps)).
3. Place the converted ssab in the project as an asset.

---
[Back to contents](../README.md)
