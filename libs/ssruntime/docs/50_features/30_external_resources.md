> [日本語](./30_external_resources.ja.md) | **English**

# External Resources and Textures

This page covers replacing textures (both authored replacement and replacement from your program at runtime) and handling instance parts that reference an animation in another ssab. Loading the textures themselves is covered in [Loading and Initialization §3](../30_core/10_initialization.md#3-loading-textures).

## 1. Texture keys

The texture to bind when drawing is always decided by `DrawBatch.texture_hash`. Its value is either the hash of a cellmap name or the hash of a texture file name ([Loading and Initialization §3](../30_core/10_initialization.md#3-loading-textures)).

**A cellmap name has no file extension.** The cellmap of `common.ssce` is named `common`, and its hash is `fnv1a("common")`. A texture change (`TCHG`), on the other hand, names a texture file (`common.png` and so on), so its hash is `fnv1a("common.png")`. The strings differ, so the two are entirely different values. **The Player's texture map must be keyed by both.** If you load both `cellmaps()` and `external_textures()`, every `texture_hash` is found. The converter guarantees that every texture name a `TCHG` points at is in `external_textures()`.

**A texture hash is unique only within one ssab.** Both a cellmap hash and an external texture hash are made from the name alone and say nothing about which ssab they belong to, so two ssab files can use the same hash for different images (both have a cellmap named `common`, for example). A Player that loads more than one ssab must keep one texture map per ssab, or key the map by ssab and hash together. When drawing a DrawBatch, look it up in the map of the ssab bound to that Context.

## 2. Authored texture change

SpriteStudio's "texture change" attribute (`TCHG`) **keeps the cell a part references and swaps only the texture it reads from**. It is used to prepare textures with the same layout but different colors or costumes, and to switch between them on the timeline.

### 2-1. The Player has nothing to do

The Brain computes this attribute and **includes its value in the conditions that split DrawBatches.** As a result:

- A part under a texture change **does not share a DrawBatch** with other parts that reference the same cellmap (it is split into a separate DrawBatch).
- Each DrawBatch's `texture_hash` points at the texture that DrawBatch must actually bind.

In other words, **a texture change draws correctly with the usual render loop that binds textures keyed by `texture_hash`.** There is no need to read per-part state and re-split DrawBatches. All you need is to have loaded `external_textures()` (§1).

### 2-2. Specification

| Item | Behavior |
|---|---|
| Interpolation | **None (step).** It switches on the key's frame and holds until the next key |
| Empty texture name | The authored "no replacement". Stored in the ssab as `0`, and the part uses its own cell's texture (it does not collide with a real texture's hash) |
| Parts it affects | **Parts that have a cell** (normal / 9-slice / mesh / mask). Parts without a cell (shape / text / instance / effect) keep `texture_hash == 0` |
| Cell source rectangle and UVs | **Unchanged.** `ss_runtime_get_local_uvs` / `ss_runtime_get_cell_meta` return values in the original cellmap's pixel coordinates |

> [!IMPORTANT]
> **The size of the replacement texture**: the UVs and cell rectangles the Brain returns stay in the original cellmap's pixel coordinates. A Player normally normalizes them by "dividing by the size of the bound texture". So **if the replacement has a different resolution from the original cellmap, the same pixel rectangle reads a different region of the picture.** Texture change assumes you replace a texture with one of the same size and the same layout.

### 2-3. `PartState.texture`

FrameData's `parts[].texture` holds the same value too (the hash of the texture name, or `0` with no replacement). It is valid only while `AttributeTexture` in `update_flag` is set.

A Player that draws with DrawBatches does not need to read it. Refer to it only in implementations that do not use DrawBatches, such as a Player in `SceneGraph` mode that creates an engine node per part.

### 2-4. Caveats

- **The number of DrawBatches changes from frame to frame.** Every time a texture change key switches, the split changes, so optimizations that assume "the number of DrawBatches is constant" (reusing buffers, for example) do not hold.
- **`external_textures()` is not a list of "external textures the author registered".** The converter also adds the texture names a `TCHG` points at (including a cellmap's own image, such as `common.png`), so that names can be looked up from hashes. A Player that shows this list in its UI as "replaceable textures" shows more entries (all of them valid replacement targets).

### 2-5. How to verify

Use `tests/overall/TextureChange.ssae`. Two parts (`TO_TEXTURE` / `TO_CELLMAP`) reference the same `common` cellmap, and each switches to a different texture at frame 1. Check that the picture matches the editor and that **the DrawBatches are split into two** on that frame.

## 3. Replacing textures at runtime

Replacing textures at runtime **from your program** rather than from the authoring (costume changes, facial expression variants and so on) is implemented by the Player. It is not part of [Part Overrides](./20_part_overrides.md).

1. At initialization, load the textures as the ssab defines them, and put them in a map keyed by `texture_hash` (§1).
2. To replace a texture, overwrite the texture under that key with another texture.
3. To clear the replacement, reload from the original path written in the ssab (`image_path` in `cellmaps()`, or `name` in `external_textures()`) and restore the map.
4. When drawing, look up the map by `DrawBatch.texture_hash` as usual. This allows costume changes and expression changes without modifying the original data.

```cpp
void Player::set_cellmap_texture(const std::string& cellmap_name, Texture* override_texture) {
    uint32_t key = ss_runtime_hash_string(cellmap_name.c_str());

    if (override_texture != nullptr) {
        textures[key] = override_texture;                       // replace
        return;
    }
    // clear: reload the original texture from the ssab definition
    auto cellmaps = ssab->cellmaps();
    for (uint32_t i = 0; i < cellmaps->size(); i++) {
        auto cellmap = cellmaps->Get(i);
        if (cellmap->name_hash() == key) {
            textures[key] = load_texture(resolve_path(cellmap->image_path()->c_str()));
            return;
        }
    }
    // if not found, search external_textures() the same way
}
```

`load_texture` / `resolve_path` are not real functions; they stand for the engine's texture loading and path resolution.

A part under a texture change (`TCHG`) is bound through a different hash, only on those frames. If the key you replaced is not that hash, the replacement has no effect (the authored change wins).

The steps and the example above are the form for a texture map that belongs to this Player alone and was built from one ssab.

**A Player whose textures live in a table shared by several Players (one table for the device or the whole engine) must not overwrite the texture under a key.** Overwriting it changes the texture for every Player playing the same ssab. Instead, keep a redirect table per Player and point the hash you replace at a key of this Player's own, one that collides with no other texture's key, and register the replacement texture in the shared table under that key. When drawing, look the texture hash up in the redirect table first, and fall back to the hash itself in the shared table. Pass every hash that names a texture through the same table, not only `DrawBatch.texture_hash` but also the ones effects and mask cutouts read. To clear a replacement, point the hash back at its original key in the redirect table; the original texture in the shared table was never touched, so there is nothing to reload.

Keep the redirect table per ssab as well (§1). An instance part's child built from another ssab looks its textures up in that ssab's table, so a replacement set on the parent's ssab does not reach it. To make it apply to the child too, set the replacement in the child ssab's table as well.

## 4. Referencing external ssabs

An instance part can reference an animation in another ssab ([Instance Parts §1](../40_parts/50_instance.md#1-preparing-the-child)).

1. Walk the parent ssab's `external_instances()` and collect the names of the referenced ssabs (`anime_pack_name`).
2. For each, load **`<anime_pack_name>.ssab` from the same directory as the parent ssab** and create a resource. Make each loaded ssab retrievable by `SsAnimeBinary.name_hash` (the hash of the ssab's name).
3. When preparing an instance part's child, look up the loaded ssab by `PartTypeInstance.ref_anime_pack_hash`. If it contains the animation `ref_anime_name_hash`, bind that ssab's resource to the child Context. If it is not found, search the parent ssab itself.

Even an embedded-mode ssab does not embed the ssabs it references ([Limitations §7](../10_introduction/20_limitations.md#7-ssab-in-embedded-mode)). Place the referenced ssab files alongside it.

---
[Back to contents](../README.md)
