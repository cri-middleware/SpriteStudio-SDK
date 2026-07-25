> [日本語](./70_external_resources.ja.md) | **English**

# Managing External Resources and Textures

An implementation guide for replacing and referencing external resources, which large projects and dynamic presentation require.

## Texture override
The feature for dynamically replacing the texture of a particular part or cellmap at runtime.

- **How to implement**: on the Player side, keep a map of texture objects keyed by `cellmap_name_hash` (or by the cellmap name string).
- **Resolution steps**:
    1. At initialization, load textures as usual according to the SSAB definitions and store them in the map.
    2. From your program, override a given key with a different texture object.
    3. **Clearing the override (restoring)**: when `nullptr` (or `null`) is specified as the override, reload the image from the original texture path defined in the SSAB and restore the map entry to its initial state.
    4. When drawing, look the target texture up in the map using `DrawBatch`'s `texture_hash()` as the key and pass it to your draw call. This makes costume changes and dynamic facial expressions possible without modifying the original data.

```cpp
// Example implementation of a texture override (replacement) (C++)
void Player::set_cellmap_texture(const std::string& cellmap_name, Texture* override_texture) {
    uint32_t name_hash = fnv1a_hash(cellmap_name); // get the hash of the cellmap name

    if (override_texture != nullptr) {
        // 1. Overwrite the map with the specified texture
        textures_map[name_hash] = override_texture;
    } else {
        // 2. Clearing: reload the original image from the definition in the SSAB and restore it
        auto cellmaps = ssab_binary->cellmaps();
        for (uint32_t i = 0; i < cellmaps->size(); i++) {
            auto cellmap = cellmaps->Get(i);
            if (cellmap->name_hash() == name_hash) {
                // Resolve the relative path and reload the original texture
                std::string original_path = resolve_path(cellmap->image_path()->c_str());
                textures_map[name_hash] = Engine::load_texture(original_path);
                break;
            }
        }
    }
}
```

## External SSAB references (External Instance)
The feature where an instance part references an animation located in *another* SSAB file (read together with [Instance Parts](./10_instance.md)).
- **How to implement**:
    1. Walk the parent SSAB's `external_instances()` section and list the referenced external SSAB names.
    2. Load those files and manage them as individual `Resource` objects.
    3. When setting up the instance part, search all loaded SSABs by `ref_anime_hash` to find the target animation, and initialize the child Context.

---
[Back to Table of Contents](../README.md)
