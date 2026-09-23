> [日本語](./60_editor_integration.ja.md) | **English**

# Editor Integration

This page explains how to build a mechanism that imports an sspj in the engine's editor and converts it to an ssab. You embed `libssconverter` in an editor extension and build **batch conversion of a whole directory** and **progress display with cancellation**. It is based on the implementations in SSPlayerForGodot and SSPlayerForUnity.

## 1. Overall structure

The `libssconverter` C-API is asynchronous by design and does its work on a separate thread. Use it as follows so that the editor's UI (the main thread) does not stall.

1. **Start**: create a Context with `ss_converter_create` and start the work (`ss_converter_convert` or `ss_converter_scan_dir`). The function returns immediately.
2. **Check every frame**: in the editor's per-frame processing, check `ss_converter_is_finished` and update the progress display.
3. **Destroy**: when it has finished, receive the result and free the Context with `ss_converter_destroy`.

## 2. Batch conversion by directory

When a user drops a folder into the editor, for example, find every sspj inside it and convert them together.

1. Start a scan on a separate thread with `ss_converter_scan_dir`.
2. Every frame, display progress such as "Searching… n found" with `ss_converter_scan_found` and `ss_converter_scan_visited`.
3. When the scan has finished, collect the paths of the sspj files found with `ss_converter_scan_result_count` and `ss_converter_scan_result_at`.

Calling `ss_converter_convert` for many sspj files all at once makes CPU load and memory use spike. Limit the number of Contexts running at once to about the number of CPU cores (4 to 8, for example), and add the next conversion whenever a slot frees up.

## 3. Cancellation

When scanning or converting a large project, being able to cancel midway is essential.

On cancel, call **`ss_converter_abort(ctx)`** on every running Context. The worker thread receives the abort request.

> [!WARNING]
> **Do not call `ss_converter_destroy` right after `ss_converter_abort`.** The thread takes a little time to stop. Keep checking every frame, and **call `ss_converter_destroy(ctx)` only after `ss_converter_is_finished(ctx)` becomes true.** If you do not, freed memory is read and the program crashes.

### 3-1. Unity (C#) example

Check every frame in `EditorApplication.update`, and receive cancellation from the `Progress` API.

```csharp
// Cancel (called from the Progress UI)
private static bool OnCancel() {
    _canceled = true;
    _pendingJobs.Clear(); // discard the waiting conversions
    unsafe {
        foreach (var ctxPtr in _activeContexts) {
            var ctx = (SpriteStudio.SsConverter.Context*)ctxPtr;
            // request an abort from the running Context
            SpriteStudio.SsConverter.NativeMethods.ss_converter_abort(ctx);
        }
    }
    return true; // do not destroy here; the per-frame check collects it
}

// Per-frame check
private static void Poll() {
    for (int i = _activeContexts.Count - 1; i >= 0; i--) {
        unsafe {
            var ctx = (SpriteStudio.SsConverter.Context*)_activeContexts[i];

            // wait until it finishes (after an abort request too, wait here for it to finish)
            if (!SpriteStudio.SsConverter.NativeMethods.ss_converter_is_finished(ctx)) continue;

            try {
                // process the result
            } finally {
                // always destroy a finished Context, even if an exception occurs
                SpriteStudio.SsConverter.NativeMethods.ss_converter_destroy(ctx);
                _activeContexts.RemoveAt(i);
            }
        }
    }
}
```

### 3-2. Godot (C++) example

Work with the progress dialog in `NOTIFICATION_PROCESS` (the per-frame processing).

```cpp
void SSImporter::_notification(int p_what) {
    if (p_what != NOTIFICATION_PROCESS) return;

    if (_is_converting) {
        // was Cancel pressed in the progress dialog?
        if (!_convert_canceling && _import_dialog->is_canceled()) {
            _convert_canceling = true;
            for (int i = 0; i < _active_ctx.size(); i++) {
                ss_converter_abort((Context *)_active_ctx[i]); // request an abort
            }
        }

        // check the running conversions
        for (int i = _active_ctx.size() - 1; i >= 0; i--) {
            Context *ctx = (Context *)_active_ctx[i];

            // after an abort request too, wait until it finishes
            if (!ss_converter_is_finished(ctx)) continue;

            CConverterError result = ss_converter_get_result(ctx);
            if (result == CConverterError::Success) {
                // ... handling on success ...
            }

            // the thread has ended, so it is safe to destroy
            ss_converter_destroy(ctx);
            _active_ctx.remove_at(i);
        }
    }
}
```

## 4. Collect even when an exception occurs

If an exception or error stops the per-frame check, the worker thread and the Context's memory are left unreleased. Even if processing the result (enumerating files in C#, for example) throws, always call `ss_converter_destroy` in `try` / `finally`, and do not let the check stop (§3-1).

## 5. Editor extensions

- **Automated import**: detect an sspj, call ssconverter automatically, and convert it into an ssab resource (Godot's `EditorImportPlugin`, for example).
- **Inspector**: let users preview animations and check texture replacements in the inspector (Godot's `EditorInspectorPlugin`, for example).
- **Hot reload**: ideally, build a mechanism that reflects a designer's save in SpriteStudio in the editor's animation right away.

---
[Back to contents](../README.md)
