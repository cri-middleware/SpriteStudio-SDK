# Test sounds

Where the audio under `tests/*/sound/` comes from.

| File | Origin |
|---|---|
| `overall/sound/test.mp3` | **Synthesized** by [`scripts/generate-test-audio.py`](../scripts/generate-test-audio.py). A 30-second loop |
| `allPartsV7/sound/se_sys_49.mp3` | **Synthesized** by the same script. A 0.8-second chime |

The synthesized files contain no third-party audio: the script computes every sample itself. To change
one, edit the script and run it again rather than replacing the file by hand, and copy the length it
prints into the `<msec>` of the project's `soundlist.ssse` (one directory above `sound/`).

Both files replace earlier ones that had been brought in from elsewhere. A snippet scan matched them to
audio published in other projects — `test.mp3` byte for byte to `music_logo.mp3` in
[cocos2d-x-classic](https://github.com/cocos2d/cocos2d-x-classic)'s test resources, whose artwork
licence does not cover it — so they could not be shown to be ours to redistribute.
