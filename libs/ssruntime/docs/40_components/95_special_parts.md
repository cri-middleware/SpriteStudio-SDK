> [日本語](./95_special_parts.ja.md) | **English**

# Supporting Special Part Types

Implementation hints for special part kinds such as 9-slice, shape, and sound.

| Part type | Implementation hint |
| :--- | :--- |
| **9-slice** | The SoA `local_vertices` already contain coordinates with the 9-slice computation applied. Simply draw them the same way as an ordinary part. |
| **Shape** | Use the dedicated SoA buffers such as `ss_runtime_get_shape_vertices` to obtain the polygon's vertex data. |
| **Sound** | Watch `passed_event` inside your update loop and play the sound if `audios` events are included (see [Audio](./90_audio.md) for details). |

---
[Back to Table of Contents](../README.md)
