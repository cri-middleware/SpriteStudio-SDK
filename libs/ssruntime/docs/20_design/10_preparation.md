> [日本語](./10_preparation.ja.md) | **English**

# Preliminary checks
`libssruntime` is implemented in Rust, and the data format you receive from it is FlatBuffers. Please check the following.

# Rust build support
Identify the CPU architectures and platforms supported by the game engine you are implementing the Player for, and check that Rust can build for them.

https://doc.rust-lang.org/beta/rustc/platform-support.html

If the `libssruntime` release artifacts do not include a library binary for your CPU architecture / platform, please build it yourself.

# Binding native libraries from your programming language
Check the following for the game engine and programming language you are targeting.

- Check that the game engine can incorporate a native library or wasm.
- Check that your programming language can bind and call functions of a C/C++ library (or a wasm library).

# FlatBuffers support
Identify the programming language of the target game engine and check that the FlatBuffers `flatc` command supports it.

https://flatbuffers.dev/support/

---

# Before implementing the Player

Before you start implementing the Player, prepare the necessary tools and set up the project.

# Preparing the Player project
- **Create an empty project**: prepare an empty, buildable project in the target game engine or language.
- **Prepare FlatBuffers**:
    - Obtain the `flatc` command.
    - From each `.fbs` schema file the SDK provides (under the `libs/ssab/fbs/` and `libs/ssruntime/fbs/` directories), generate parsing code for the language your Player uses.
    - Incorporate the generated code and the FlatBuffers runtime library (`flatbuffers.h` and so on) into your project.

```bash
# Generation example for C++
flatc --cpp -o ./src/gen/ libs/ssab/fbs/*.fbs
flatc --cpp -o ./src/gen/ libs/ssruntime/fbs/*.fbs
```

- **Linking libssruntime**:
    - Build `libssruntime` and place the resulting static library (`.lib` / `.a`) or dynamic library (`.dll` / `.so` / `.dylib`) into your project.
    - Configure the linker so the library is linked into your artifact.
    - Add the `ssruntime.h` header file to your include path.

# Preparing an ssab file
- **Obtain the conversion tool**: prepare `ssconverter-cli`.
- **Prepare test data**: convert something like `tests/overall/overall.sspj` to create a `.ssab` file.
- **Register the asset**: register the `.ssab` you created in your Player project's asset directory.

# Integrating with the engine's own asset management (recommended)
Many game engines (Unity, Godot, Unreal Engine, and others) have their own resource-management systems.

- **Implement a custom importer / loader**:
    - Make the engine recognize the `.ssab` extension and load it automatically as binary data.
    - Example: implementing a `ResourceFormatLoader` in Godot.
    - This lets users assign assets by drag & drop in the editor, which greatly improves usability.

- **Keep the base directory**:
    - When using the `Reference` scheme (external texture references), being able to pass the Player the directory path that contains the `.ssab` file makes resolving texture load paths much easier.

# Coordinate-system setup and the "double flip" trap
You must set the correct flag with `ss_runtime_set_coordinate_system` (Y-Up = `0`, Y-Down = `1`) to match the coordinate system of the target engine (Unity is Y-Up, Godot is Y-Down, and so on).
If you overlook the fact that the vertical flip is already applied during local-space vertex computation and flip the Y axis again yourself in the world-matrix computation, you get a "double flip" (double mirror) and the display breaks. Align the engine's specification with the specification of the data provided, and take care that the transform is not applied twice.

# Verification: the build passes
At this stage, confirm that you can call APIs such as `ss_runtime_create()` and that the project builds and runs without errors.

---
[Back to Table of Contents](../README.md)
