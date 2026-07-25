> [日本語](./10_editor_integration.ja.md) | **English**

# Editor Integration & Conversion

When porting a Player to a game engine (UE5, Godot, Unity, and so on), it matters not only that playback works at runtime, but also that you build a workflow for "how a `.sspj` is imported in the editor and converted into an optimized `.ssab`".

This chapter explains the best practices for embedding `libssconverter` into an engine's editor extension to achieve **directory-wide batch conversion** and **cancellation tied to a progress bar**. The content is based on the knowledge gained from the official Godot and Unity implementations.

## 1. Overall design philosophy

The `libssconverter` API is asynchronous by design, processing work on a background worker thread. To avoid blocking the editor's UI (the main thread), take the following architecture.

1. **Create the Context and start**: create a context with `ss_converter_create` and start processing (`ss_converter_convert` or `ss_converter_scan_dir`). The function returns immediately.
2. **Poll from the main loop**: watch `ss_converter_is_finished` inside the editor's Update/Process event (per-frame processing) and update the progress dialog.
3. **Destroy safely**: once finished, receive the result and free the memory with `ss_converter_destroy`.

## 2. Recursive, directory-wide batch conversion

Being able to find and convert every `.sspj` inside a folder at once - for example when a user drops several folders into the editor - is very convenient. `libssconverter` provides fast C-API traversal functions.

### Traversal flow
1. Start a background scan with `ss_converter_scan_dir`.
2. While polling from the editor, use `ss_converter_scan_found` and `ss_converter_scan_visited` to display progress such as "Searching… n found".
3. When the scan reports `finished`, loop over `ss_converter_scan_result_count` and `ss_converter_scan_result_at` to collect the list of `.sspj` paths found.

### Scheduling parallel conversions

When converting many discovered `.sspj` files, issuing `ss_converter_convert` for all of them at once causes CPU load and memory usage to spike.
We therefore recommend a design that limits the number of running Contexts to the CPU core count (say a maximum of 4-8) and keeps feeding conversion jobs into the free slots (a pump).

## 3. Progress display and cancellation

When converting or scanning a large project containing thousands of files, being able to cancel mid-way is essential.

### Rules for implementing cancellation

When the editor's "Cancel" button is pressed, immediately call **`ss_converter_abort(ctx)`** for every running context. This lets the Rust-side worker thread receive the abort request safely.

> [!WARNING]
> **Important caveat (preventing use-after-free)**
> Even after you call `ss_converter_abort`, it takes a little time for the worker thread to stop completely. Therefore **you must not call `ss_converter_destroy` immediately after aborting**.
> Always keep polling and **call `ss_converter_destroy(ctx)` only once `ss_converter_is_finished(ctx) == true`**. Failing to do so risks a crash, for example through access to freed memory.

### Unity (C#) implementation example

In Unity, poll with `EditorApplication.update` and pick up cancellation through the `Progress` API.

```csharp
// Cancellation callback (called from the Progress UI)
private static bool OnCancel() {
    _canceled = true;
    _pendingJobs.Clear(); // clear the waiting queue
    unsafe {
        foreach (var ctxPtr in _activeContexts) {
            var ctx = (SpriteStudio.SsConverter.Context*)ctxPtr;
            // issue an abort instruction to the running context
            SpriteStudio.SsConverter.NativeMethods.ss_converter_abort(ctx);
        }
    }
    return true; // do not Destroy here; let the polling loop collect it safely
}

// Update polling loop
private static void Poll() {
    for (int i = _activeContexts.Count - 1; i >= 0; i--) {
        unsafe {
            var ctx = (SpriteStudio.SsConverter.Context*)_activeContexts[i];

            // wait until finished (after issuing Abort, wait here until it becomes Finished)
            if (!SpriteStudio.SsConverter.NativeMethods.ss_converter_is_finished(ctx)) continue;

            try {
                // process the result (registering into the source map and so on)
            } finally {
                // always destroy after it is finished, even if an exception occurs
                SpriteStudio.SsConverter.NativeMethods.ss_converter_destroy(ctx);
                _activeContexts.RemoveAt(i);
            }
        }
    }
}
```

### Godot (C++) implementation example

For Godot, use `NOTIFICATION_PROCESS` (the per-frame Update) and cooperate with a custom ProgressDialog.

```cpp
void SSImporter::_notification(int p_what) {
    if (p_what != NOTIFICATION_PROCESS) return;

    if (_is_converting) {
        // detect whether Cancel was pressed on the progress dialog
        if (!_convert_canceling && _import_dialog->is_canceled()) {
            _convert_canceling = true;
            for (int i = 0; i < _active_ctx.size(); i++) {
                ss_converter_abort((Context *)_active_ctx[i]); // abort request
            }
        }

        // poll the active conversions
        for (int i = _active_ctx.size() - 1; i >= 0; i--) {
            Context *ctx = (Context *)_active_ctx[i];

            // even after an abort request, wait until it has completely finished
            if (!ss_converter_is_finished(ctx)) continue;

            // if it was not aborted, process the result
            CConverterError result = ss_converter_get_result(ctx);
            if (result == CConverterError::Success) {
                // ... handling on success ...
            }

            // the thread is guaranteed to have ended, so destroy safely
            ss_converter_destroy(ctx);
            _active_ctx.remove_at(i);
        }
    }
}
```

## 4. Error handling and self-recovery

In an editor integration, if the converter's polling loop stops because of an exception or error, the background worker thread and the C memory (the Context) leak permanently.

- **Exception handling in C# and similar**: even if an exception is thrown while processing the result on the C# side after conversion completes (enumerating files, for example), it is important to design for self-recovery - use a `try / finally` block so that `ss_converter_destroy` is always called and the polling loop never stops.

## 5. Integrating with editor extensions and the workflow

To improve development efficiency, we strongly recommend integrating `ssconverter` as an editor extension of the game engine.

- **Godot (C++) examples**:
    - `EditorImportPlugin`: detects `.sspj`, calls `ssconverter` automatically, and converts it into an internal `.ssab` resource.
    - `EditorInspectorPlugin`: lets you preview animations and check texture overrides in the inspector.
- **Workflow**: ideally, build an environment (hot reload) where the latest animation is reflected in the game editor the moment a designer saves in SpriteStudio.

---
[Player Porting Guide Top](../README.md)
