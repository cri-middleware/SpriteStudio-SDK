> [日本語](./70_external_resources.ja.md) | **English**

# Managing External Resources and Textures

An implementation guide for replacing and referencing external resources, which large projects and dynamic presentation require.

## Texture resolution: `texture_hash` spans two hash spaces

The texture to bind when drawing always comes from `DrawBatch`'s `texture_hash()`. That value, however, lives in **one of two hash spaces**.

| Where `texture_hash` comes from | The matching ssab entry | When it happens |
|---|---|---|
| Hash of the cellmap name | `cellmaps()[].name_hash` | Normally (the default) |
| Hash of the texture **file name** | `external_textures()[].name_hash` | When an authored TextureChange (`TCHG`) applies to the part |

> **A cellmap name carries no file extension.** The cellmap of `common.ssce` is named `common`, so its hash is `fnv1a("common")`. A `TCHG`, on the other hand, names a texture *file* (`common.png` and so on), giving `fnv1a("common.png")`. Different strings mean entirely different values, so **the Player's texture map must be keyed by both**.

At initialization, therefore, load **both** `cellmaps()` and `external_textures()` and put them into the same map, each keyed by its `name_hash()` (see [Loading & Initialization](../30_core/10_initialization.md) §3). With those two tables at hand, every `texture_hash` resolves. The converter guarantees that every texture name a `TCHG` can select is present in `external_textures()`.

## Authored texture change (TextureChange / `TCHG`)

SpriteStudio's "texture change" attribute **keeps the cell the part references and swaps only the texture it samples from**. The typical use is preparing atlases with the same layout but different colors or costumes, and switching between them from the timeline.

### The Player normally needs no implementation

The runtime evaluates this attribute and **folds its value into the `DrawBatch` key**. As a result:

- A part under an override **cannot be merged into the same batch** as other parts sharing its cellmap; it is split into its own batch.
- Each batch's `texture_hash` names the texture that batch must actually bind.

In other words, **a texture change renders correctly with the ordinary drawing loop that binds by `texture_hash`**. You do not need to read per-part state and re-split batches yourself. All that is required is having loaded `external_textures()`, as described in the previous section.

### Specification

| Item | Behavior |
|---|---|
| Interpolation | **None (step-evaluated)**. It switches on the keyed frame and holds until the next key. |
| An empty texture name | The authored "no override". It is stored in the ssab as `0`, and the part keeps its cell's own texture (it can never collide with a real texture's hash). |
| Applicable parts | **Parts that resolved a cell** (`Normal` / `Nines` / `Mesh` / `Mask`). Kinds without a cell (`Shape` / `Text` / `Instance` / `Effect`) keep `texture_hash == 0`. |
| Cell rect and UVs | **Unchanged**. `ss_runtime_get_local_uvs` / `ss_runtime_get_cell_meta` keep returning values in the *original* cellmap's pixel space. |

> [!IMPORTANT]
> **The size of the replacement texture.** The UVs and cell rects the runtime returns stay in the original cellmap's pixel coordinates. A Player normally normalizes them by dividing by the size of the texture it **bound**, which means that if the replacement has a different resolution from the original cellmap, the same *pixel* rect samples a visually different region. A texture change assumes you swap in an atlas of the **same size and the same layout**.

### About `PartState.texture`

`FrameData`'s `parts[].texture` carries the same value (the fnv1a hash of the texture name, or `0` for no override). It is meaningful only while `UpdateAttributeFlags::AttributeTexture` is set.

A Player that draws by batches has no reason to read it. Only implementations that do not use `DrawBatch` — such as the `RenderMode::SceneGraph` approach of creating one engine node per part — need to look at it.

### Points to watch

- **The batch count changes from frame to frame.** Every time a texture-change key switches, the way batches split changes with it, so optimizations that assume a fixed batch count (reusing buffers, for example) break. This is the same caveat as cell replacement in [Part Overrides](./65_override_layer.md).
- **`external_textures()` is not "the list of external textures the author registered".** The converter also adds every texture name a `TCHG` points at — including a cellmap's own image, such as `common.png` — so that the Player can turn the hash back into a name. A Player that surfaces this list in its UI as "textures you can replace" will show additional entries (all of them legitimately replaceable).

### Verification

Use `tests/overall/TextureChange.ssae`. Two parts (`TO_TEXTURE` / `TO_CELLMAP`) reference the same `common` cellmap and switch to different textures at frame 1. Check that the picture matches the editor **and** that the frame produces **two separate batches**.

## Runtime texture replacement (a Player-side override)

Replacing a texture at runtime **from your program** rather than from the authoring data (costume changes, dynamic facial expressions, and so on). This is the Player's responsibility and is not part of the [Override Layer API](./65_override_layer.md).

- **How to implement**: on the Player side, keep a map of texture objects keyed by `texture_hash` (which, as described above, holds both cellmap-name hashes and `external_textures` hashes).
- **Resolution steps**:
    1. At initialization, load textures as usual according to the SSAB definitions and store them in the map.
    2. From your program, override a given key with a different texture object.
    3. **Clearing the override (restoring)**: when `nullptr` (or `null`) is specified as the override, reload the image from the original texture path defined in the SSAB (`image_path` in `cellmaps()`, or `name` in `external_textures()`) and restore the map entry to its initial state.
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
        // If it is not there, search external_textures() the same way and restore from it
    }
}
```

> A part under a texture change (`TCHG`) binds through a different hash on those frames. If the key you replace is not that hash, your override has no effect — the authored change wins.

## External SSAB references (External Instance)
The feature where an instance part references an animation located in *another* SSAB file (read together with [Instance Parts](./10_instance.md)).
- **How to implement**:
    1. Walk the parent SSAB's `external_instances()` section and list the referenced external SSAB names.
    2. Load those files and manage them as individual `Resource` objects.
    3. When setting up the instance part, search all loaded SSABs by `ref_anime_hash` to find the target animation, and initialize the child Context.

---
[Back to Table of Contents](../README.md)
