# Raven

Raven is a graphics engine for 3D applications.

Projects are created using an editor:

![](screenshots/editor.png)

and then exported as standalone executables:

![](screenshots/runtime.png)

## Features

+ Quite fast. Demo shown in screenshots above (although very simple) runs at 7000 FPS with an average time-per-frame of 141µs on my computer.
+ Display textured 3D models (duh) with unlit shading (booo).
+ Supports a vast amount of texture formats (thanks to the [image](https://github.com/image-rs/image) crate).
+ Supports a vast amount of 3D scenes formats (thanks to the [Assimp](https://github.com/assimp/assimp) importing library).
+ Skybox (but it is hardcoded and you cannot change it, arggh).
+ With the editor you can: create new entities, rearrange their hierarchy (global transform is preserved when doing so), add new components, edit component values such as the transform or the name, remove components, import 3D scenes, create a new scene, open an existing scene, export the project.
+ The editor's layout is fully customizable (thanks to [Dear ImGui](https://github.com/ocornut/imgui)).
+ Scene components allow a scene to have entities that display other scenes. These can be imported from a file (such as .fbx, .obj, .gltf, etc..) or user-made.
+ Uses an in-house ECS library built from scratch. The crate lives under `raven_ecs`. It uses the same sparse-array technique as [entt](https://github.com/skypjack/entt).

## Building

Raven depends on C/C++ libraries Assimp and Dear ImGui. Once you have them installed, you can build raven
with `cargo build --release -p raven_runtime && cargo build --release -p raven_editor`.

Please note that the building process for `raven_editor` assumes that the built executable for `raven_runtime` is available at `target/release/raven_runtime` relative to the working directory.

## Closing remarks

I created Raven because I got curious to try out graphics programming, but also because I wanted to see what it took to make a game engine.

So I gave [Learn OpenGL](https://learnopengl.com/) a good read and then started programming.

Originally, I had bigger ambitions for Raven. It was going to have Python scripting, particles, physics, skeletal meshes and what not.

But the MVP I always had in mind was what Raven is today: a very barebone system made of an editor and the ability to export a standalone application for the project. With minimal capabilities such as displaying textured meshes that could be arranged in an hierarchy and moved about.

I was not going to abandon the project before reaching this goal. But now that I have, I'd like to move on to other projects.

The amount of actual graphics programming that got done during my time with Raven is very little. There were a lot of software engineering, planning and fighting the Rust compiler, but not the amounts of OpenGL I was longing for. And this is maybe the biggest reason that makes me not want to develop Raven further.

Nonetheless, the journey was beautiful and this project taught me an incredible amount of new things ranging from more advanced Rust topics such as declarative and procedural macros, C/C++ bindings, closure borrowing rules, the popular `serde` crate for serializing and deserializing, advanced traits and generics concepts, references variance. Then, of course, a great deal bout OpenGL, windowing libraries and finally ECS architectures.
