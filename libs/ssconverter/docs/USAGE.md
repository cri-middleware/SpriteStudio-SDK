# libssconverter Integration Guide

`libssconverter` is a C-compatible library for converting SpriteStudio 7 project files (`.sspj`) into a runtime-optimized binary format (`.ssab`).
It is designed to be integrated into custom game engines, tools, or development editors for in-process conversion.

## 1. Preparation

### Linking the Library

`libssconverter` is implemented in Rust and is used via its C-API.
Link the static or dynamic library appropriate for your platform to your project.

- **Windows**: `ssconverter.lib` (and `ssconverter.dll` if shared)
- **macOS/Linux**: `libssconverter.a` (and `libssconverter.dylib`/`.so` if shared)

Also, add any necessary system libraries for each platform (e.g., pthread, dl).

### Including the Header

Include `ssconverter.h` in your project and include it in your source code.

```cpp
#include "ssconverter.h"
```

## 2. Basic Workflow

The `libssconverter` API is designed to be asynchronous, executing the conversion process in a background thread while the caller monitors completion via polling.

### Creating a Context

Create a `Context` object to manage the conversion process.

```cpp
Context* ctx = ss_converter_create();
```

### Starting the Conversion

Calling `ss_converter_convert` starts a worker thread internally and begins the conversion. This function returns immediately.

```cpp
const char* sspj_path = "C:/Assets/Character.sspj";
const char* output_dir = "C:/Project/Assets/SSAB/";

// Start conversion
// The 4th argument enables embedded output (textures packed into the .ssab); pass false for
// the conventional layout (textures referenced as sibling PNGs).
// The 5th argument is a log callback (nullptr if not needed)
ss_converter_convert(ctx, sspj_path, output_dir, /*embedded=*/false, [](const char* msg) {
    printf("SSConverter: %s\n", msg);
});
```

### Monitoring Progress

In your editor's main loop or frame update function, call `ss_converter_is_finished` to check for completion.

```cpp
void on_update() {
    if (ctx && ss_converter_is_finished(ctx)) {
        // Conversion is complete
        on_conversion_complete(ctx);
    }
}
```

### Retrieving Results and Cleanup

Once the conversion is complete, check the result and destroy the context.

```cpp
void on_conversion_complete(Context* ctx) {
    CConverterError result = ss_converter_get_result(ctx);
    
    if (result == CConverterError::Success) {
        printf("Conversion successful!\n");
    } else {
        // Retrieve error details
        char* err_msg = nullptr;
        uintptr_t len = 0;
        ss_converter_get_error(ctx, &err_msg, &len);
        if (err_msg) {
            printf("Error: %.*s\n", (int)len, err_msg);
            // Memory obtained via ss_converter_get_error is freed by ss_converter_destroy
        }
    }

    // Destroy the context (required)
    ss_converter_destroy(ctx);
}
```

## 3. Advanced Features

### Aborting Conversion

Call `ss_converter_abort` if a long-running process is cancelled by the user.

```cpp
ss_converter_abort(ctx);
```

## 4. Implementation Example: Godot Plugin Integration

In `SSPlayerForGodot`, `libssconverter` is called when a file is dropped into the editor to generate `.ssab` files in-process.

```cpp
// Excerpt from gd_ss_import_dock.cpp

void GdSsImportControl::_notification(int p_what) {
    if (p_what == NOTIFICATION_PROCESS) {
        if (is_importing) {
            bool all_finished = true;
            for (void* ctx : import_contexts) {
                if (!ss_converter_is_finished((Context*)ctx)) {
                    all_finished = false;
                    break;
                }
            }

            if (all_finished) {
                // Finalization and resource scanning
                is_importing = false;
                set_process(false);
                // ... cleanup contexts ...
            }
        }
    }
}
```

This allows multiple files to be converted in parallel (while `libssconverter` itself uses `rayon` for parallel processing, creating multiple contexts allows multiple projects to be handled concurrently).

## 5. Notes

- **Memory Management**: Always destroy contexts created with `ss_converter_create` using `ss_converter_destroy`.
- **Thread Safety**: While conversion happens in an internal thread, do not call `ss_converter_convert` multiple times simultaneously on the same `Context`.
- **Path Format**: It is recommended to use absolute paths when passing them to the library.
