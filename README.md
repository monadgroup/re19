# Atlas source code dump

This is a dump of the source code for the engine, graphics tool and player for
[Atlas](https://www.pouet.net/prod_nfo.php?which=80996), our 64k demo released
with Macau Exports at Revision 2019. It's been updated to build with the latest
Rust nightly, but for the most part is identical to what we showed in the
[graphics breakdown stream](https://youtu.be/Y3d8jR_IwYw).

Goals of this dump:

 - You should be able to build and run a fully-functional tool and player.
 - You should be able to explore the code in this repo and maybe learn some
   things about how we built it.

To that end, we will accept pull requests and issue reports that help us
achieve these goals. Other PRs and issues will not be accepted. For instance,
a PR to fix a crash when starting the player is fine, but a PR that replaces
part of our post-processing stack is not.

## Building

You'll need a 32-bit, nightly build of Rust. The repo has been tested on the
`2021-10-01` nightly.
[Install Rustup](https://rustup.rs/) then run the following command in the repo
folder:

```
> rustup override set nightly-2021-10-01-i686
```

The repo contains two executable packages. The graphics tool provides features
including live shader reloading, a property editor and an animation system with
a timeline and curve editor. The player bundles up a synth, shaders and
animation data into an optimized executable that plays the demo from start to
end and can be krunched to under 64kb.

To build and launch the tool, run the following command from the `tool` folder:

```
> cargo run
```

To build the player:

 - Ensure the tool has run at least once to generate the export blob - see the
   section below on running the tool.
 - Ensure you have a [resource compiler](https://docs.microsoft.com/en-us/windows/win32/menurc/resource-compiler)
   available on your PATH. If you have Visual Studio installed, this can be
   achieved by using a Developer Command Prompt.
 - Run the following command from the `player` folder.
   ```
   > cargo build --release
   ```

This will create a `player.exe` inside the `target` folder in the repo.

## Using the tool

The tool will open the project file at `project/saves/000000-000000-head.save`
when started, and save to that file on close. By default this has the timeline
from the post-party final version of Atlas.

When closed, the tool minifies all shaders and generates a binary blob of
shaders and animation data to be packed into a player build. This blob is placed
in `project/data.blob`. Closing the tool will take a bit of time due to
minifying shaders, if you want to disable this set `MINIFY_SHADERS` to false
at the top of
[tool/src/exporter/shaders.rs](`https://github.com/monadgroup/re19/blob/main/tool/src/exporter/shaders.rs`).

For more info on using the tool have a look at the
[Atlas graphics breakdown video](https://youtu.be/Y3d8jR_IwYw).

## Known issues

 - There are A LOT of warnings. Some of these are just rushed coding, some of
   them are due to changes in Rust.

## License

The demo code is provided under the MIT license.

[laurentlb's Shader Minifier](https://github.com/laurentlb/Shader_Minifier) is
provided under the Apache 2.0 license.

[Un4seen Developments' BASS audio library](https://www.un4seen.com/) is provided under a free for
non-commercial use license.

[ocornut's dear imgui](https://github.com/ocornut/imgui) is provided under the
MIT license.

[cimgui](https://github.com/cimgui/cimgui) is provided under the MIT license.

[imgui-sys](https://github.com/imgui-rs/imgui-rs/tree/main/imgui-sys) is provided under the MIT license.

[WaveSabre](https://github.com/logicomacorp/WaveSabre) is provided under the MIT license.
