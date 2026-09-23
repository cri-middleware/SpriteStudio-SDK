> [日本語](./40_sequence.ja.md) | **English**

# Sequence

A sequence is a setting that plays several animations in order. ssconverter builds an ssqb from an ssqe. The Brain has no way to switch from one animation to the next by itself, so the Player manages how a sequence advances.

## 1. What an ssqb contains

The ssqb schema is in `libs/ssab/fbs/ssqb.fbs` (the file identifier is `"SSQB"`).

| Table | Field | Contents |
| :--- | :--- | :--- |
| `SsSequenceBinary` | `sequence_list` | The list of sequences |
| `Sequence` | `name` / `name_hash` | The sequence's name and hash |
| | `type` | Behavior at the end of the sequence (`Last` / `Keep` / `Top`) |
| | `list` | The items to play in order |
| `SequenceItem` | `ref_anime_pack` / `ref_anime_pack_hash` | Name and hash of the ssab that contains the animation |
| | `ref_anime_name` / `ref_anime_name_hash` | Name and hash of the animation |
| | `repeat_count` | How many times the item repeats |

Find the ssab an item references the same way as for an instance part's external reference ([External Resources and Textures §4](./30_external_resources.md#4-referencing-external-ssabs)).

## 2. Advancing playback

1. Set up the item's animation, set `repeat_count` as the loop count, and play it.
2. Every frame, detect the end of the item on the rising edge of `ss_runtime_is_finished` ([Playback Control §7](../30_core/20_playback_control.md#7-detecting-loops-and-the-end)).
3. Set up and play the next item. If the item is in another ssab, bind that ssab's resource first.
4. When the last item ends, stop or continue according to `type`.

---
[Back to contents](../README.md)
