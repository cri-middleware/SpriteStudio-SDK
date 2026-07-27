> [日本語](./30_implementation_roadmap.ja.md) | **English**

# Implementation Roadmap: verifying features with overall.sspj

When implementing a Player, the most important and most reliable verification method is **to constantly compare the display and behavior in the SpriteStudio 7 editor itself against the behavior of the Player you are implementing**.

For staged feature verification, **we strongly recommend using the test project `tests/overall/overall.sspj` bundled with this SDK**. That project covers an animation per feature, and working through the steps below lets you complete your Player's functionality with confidence.

## Step 1: Complete basic drawing (Basic)
First, aim to display this animation correctly.

- **Verify with**: `Basic.ssae`
- **Checkpoints**:
    - **Position / rotation / scale**: do child parts correctly follow the movement of their parent part?
    - **Opacity (alpha)**: when you lower a parent part's opacity, do the child parts fade along with it?

## Step 2: Compositing and color (BlendMode / PartsColor)
- **Verify with**: `BlendMode.ssae`, `PartsColor.ssae`
- **Checkpoints**:
    - **BlendMode**: are Add and Mul applied correctly against the background?
    - **PartsColor**: is coloring applied correctly, both per whole part and per vertex?

## Step 3: Advanced structure and deformation (Instance / Mask / Mesh)
Using the SDK's **Slot API** and **SoA buffers**, you can implement these as standard features.

- **Verify with**: `Instance.ssae`, `Mask.ssae`, `Mesh.ssae`
- **Checkpoints**:
    - **Instance**: is another animation looping inside the main animation (synchronized via the Slot API)?
    - **Mask**: are parts displayed only within the mask area (the batch drawing contract applied)?
    - **Mesh**: are vertex deformation and bone following (skinning) smooth (using the SoA buffers)?

## Step 4: Interactive features (UserData / Signal / Bounding)
Verify the features that are indispensable for game integration.

- **Verify with**: `UserData.ssae`, `Signal.ssae`, `Bounding.ssae`
- **Checkpoints**:
    - **UserData / Signal**: do events fire on the expected frames, and can your program detect them?
    - **Bounding**: does the collision frame follow the movement of the part?

## Step 5: Visual effects (Effect)
The final check for particle presentation.

- **Verify with**: `Effect.ssae`
- **Checkpoints**:
    - **Effect**: do particles such as sparks and smoke scatter and fade out over time (simulated via the Effect Slot)?

---

## Step 6: Verifying optional features
Check the following special parts and attributes as needed.
- **Sound**: does the sound play on the specified frame?
- **9Slice**: are the corners preserved when scaled up?
- **TextureChange**: in `TextureChange.ssae`, are two parts that reference the same cellmap drawn with different textures? (If you missed loading `external_textures()`, one of them does not draw.) → [Managing External Resources and Textures](../40_components/70_external_resources.md)

---

## 🚀 Future plans
As the SDK evolves, the following features are planned to be supported in turn.
- **Text parts**: dynamic text rendering and layout.
- **Extended shaders**: integration of non-standard custom post-processing.
- **Retrieving parts color from SoA**: parts color is currently obtained from the `FrameData.parts_color` table; support for obtaining it from an SoA buffer alongside the vertex data is planned.

---
[Back to Table of Contents](../README.md)
