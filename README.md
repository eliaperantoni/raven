<p align="center">
  <img src="banner.png">
</p>

<p align="center">
    An integrated graphics engine for 3D applications, powered by a custom Entity Component System framework.
</p>

---

<p align="center">
    Projects are created using an editor:
</p>

<p align="center">
    <img src="screenshots/editor.png" width="880">
</p>

<p align="center">
    and then exported as standalone executables:
</p>

<p align="center">
    <img src="screenshots/runtime.png" width="880">
</p>

## Features

+ Quite fast. The demo shown in the screenshots above (although very simple) runs at 7000 FPS with an average time-per-frame of 141µs on my computer.
+ Display textured 3D models with unlit shading.
+ Supports a vast amount of texture formats (thanks to the [image](https://github.com/image-rs/image) crate).
+ Supports a vast amount of 3D scene formats (thanks to the [Assimp](https://github.com/assimp/assimp) importing library).
+ Skybox (although it's currently hardcoded and cannot be changed).
+ With the editor you can: create new entities, rearrange their hierarchy (global transform is preserved when doing so), add new components, edit component values such as the transform or the name, remove components, import 3D scenes, create a new scene, open an existing scene, export the project.
+ The editor's layout is fully customizable (thanks to [Dear ImGui](https://github.com/ocornut/imgui)).
+ Scene components allow a scene to have entities that display other scenes, these can be imported from a file (such as .fbx, .obj, .gltf, etc..) or user-made. Inspiration for this comes directly from the [Godot](https://godotengine.org/) game engine.
+ Uses an in-house ECS library built from scratch. The crate lives under `raven_ecs`. It uses the same sparse-array technique as [entt](https://github.com/skypjack/entt).

## Building

Raven depends on C/C++ libraries Assimp and Dear ImGui. Once you have them installed, you can build Raven
with `cargo build --release -p raven_runtime && cargo build --release -p raven_editor`.

Please note that the building process for `raven_editor` assumes that the built executable for `raven_runtime` is available at `target/release/raven_runtime` relative to the working directory.
