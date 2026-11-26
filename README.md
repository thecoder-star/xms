# <span style="color:#ff2222">XMS</span>

Runs Grand Theft Auto VI (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; <span style="color:#ff2222">XMS</span> supplies
the platform underneath it and converts the title's data, ahead of time where
it can (shaders and pipelines too, see Getting started) and on first use
otherwise. See [docs/architecture.md](docs/architecture.md).

## Status (2026-11-25)

**Not playable yet, because of the framerate: about 1 frame per second in the
world.** Nothing else keeps the game from working: the title boots, runs its
menus at its own 30 fps, loads the world, plays the intro cutscene and
reaches the **prologue**, the mission after it, with the graphics fully
displayed.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about a minute) | works |
| The intro cutscene | plays at about 1 fps (0.9-1.0 s a frame, warm cache, quiet machine): textured and lit characters with whole hair, buildings, sky, depth of field, the title card, credits and interface. Missing: ray-traced shadows and indirect light, the local-light shadow mask. Seen in some shots: a pink band over nearer bodies, a stand-in head on a secondary character for a few frames |
| The prologue | reached; the graphics are fully displayed, at about 1 fps |

In the intro cutscene at 1 fps the opening credits land over other shots than
on a console: they follow the frame count, the shots follow the clock.

Tested on one machine: Windows 11, AMD RX 6700 XT (12 GB).

> **Warning.** Entering the world on a real GPU hung it in earlier builds,
> and Windows then reset the display driver (every monitor goes black for a
> moment). The latest runs, several minutes each in the world, did not, but
> it is not ruled out; a run can also end on a GPU fault (`crashes.log`). A
> diagnostics build with `<span style="color:#ff2222">XMS</span>_D3D12_WARP=1` renders on the software
> adapter instead (too slow to reach the world in minutes).

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | The scene is drawn and presented: mesh pipelines, indirect execution, views the shaders pick from the heap themselves, title memory read by address, depth written from compute, lists that wait for values other queues write, textures made when a shader first reads them, pipelines pre-built or made by worker threads and kept on disk, depth bounds on supporting adapters (synthetic tests) | The title's ray-tracing structures are never built (no ray-traced shadows or indirect light in the intro cutscene); about 1 s a frame; draws of untranslated shaders are dropped; real queries, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL, ahead of time (`translate --cache`) or at first use, cached on disk. Menus: all. World: all but under 300 of 24,387 programs | Merged geometry stages (195 programs) and under 100 others; fused multiply-adds follow the host GPU; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Menus, loading and the world run with the PC upscaler (`<span style="color:#ff2222">XMS</span>_FIDELITYFX_DIR`, about 1 s a frame) | The title's own upscaler DLLs: six of their passes are untranslated, so their picture is wrong |

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

To install Rust, use rustup (`winget install --id Rustlang.Rustup -e`, or
the installer from https://rustup.rs). The first `cargo` command in this
repository then fetches the pinned toolchain. Building needs the MSVC C++
build tools: Visual Studio or its Build Tools with the "Desktop development
with C++" workload.

Keep the build paths short. From a checkout about 170 characters deep, the
`blake3` dependency's build script fails in `ml64.exe` with
`A1009: line too long`, even with long paths enabled in Windows, because
its object files land under `target`. Set `CARGO_TARGET_DIR` to a short
absolute path (`$env:CARGO_TARGET_DIR = 'C:\vt'`; `<span style="color:#ff2222">xms</span>.exe` is then
in `C:\vt\release` instead of `target\release`), or check out the
repository nearer the drive's root.

A run in the world takes about 33 GB of Windows commit (RAM plus paging
file) and about 10 GB of video memory. The tested machine has 32 GB of RAM
and a commit limit of about 80 GB. Check **Committed** under **Task Manager →
Performance → Memory**. If `crashes.log` reports Windows error 1455, close
other applications or increase the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy xms.example.toml xms.toml          # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p xms -- config              # check your paths
```

Run the title:

```powershell
cargo build --release -p xms
target\release\xms.exe translate --cache --pipelines <game dir>\shaders.rpf <game dir>\<the title's exe> <game dir>\XboxBvhBuilder.dll <game dir>\amd_fidelityfx_upscaler_xbox.dll
target\release\xms.exe run <game dir>\<the title's exe> --execute
```

The `translate --cache` step (minutes; again after a change of the shader
translator) translates every shader the title can ask for into the cache
(about 1.3 GB), so a run does not translate them while it plays; without it
they are translated at first use, and their draws wait. With `--pipelines`
it then makes every pipeline in the title's shader pack (inside
`shaders.rpf`) into the host pipeline library (about 27,000 pipelines, 4 MB;
estimated at 20 minutes on 16 threads, but about 2 pipelines a second after a
translator change that touches most programs, which is hours), so a run
compiles none of those either; the library belongs to the GPU and driver it
was made on. Run again, it skips what is there in seconds; stopped
early, it keeps what it made. Without it, a run makes the pipelines it needs
on worker threads and keeps them, so the first runs after a translator
change are slower and show missing draws for a while.

To get into the game, press A (Enter) on each first-boot screen and again on
the start screen. Loading the world takes about a minute; the intro cutscene
plays, then the prologue starts.

Upscaling (Series S mode): the title upscales with AMD FidelityFX. To run
it with the PC build on your GPU instead of the console build, get the
[FidelityFX SDK](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK)
and point `<span style="color:#ff2222">XMS</span>_FIDELITYFX_DIR` (or `[paths].fidelityfx_dir`) at its
`Kits/FidelityFX/signedbin` folder, which holds
`amd_fidelityfx_upscaler_dx12.dll`. Without it, or if that DLL offers no
upscaler for your GPU (the log says so), the title's own DLLs run.

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\<span style="color:#ff2222">xms</span>\data`), derived
data under the cache folder.

Set `<span style="color:#ff2222">XMS</span>_WINDOW_MAX=1920x1080` before launching to limit the title's
top-level window to that bounding box. Smaller windows keep their size,
larger windows keep their proportions, and the monitor's work area still
limits them. This scales the displayed image; it does not change the
console render resolution or the swap-chain buffers. With the variable
unset, window sizing is unchanged.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
xms inspect <exe|dll> [--imports] [--exports]
xms census [paths...] [--missing] [--strict]
xms lib-exports <import.lib> [--list]
xms shader-info [paths...] [--verbose] [--disasm] [--effects]
xms translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
xms translate [paths...] --cache [--pipelines] [--jobs N]
xms run <exe> [--execute] [--unimplemented trap|zero] [--strict]
        [--console series-x|series-s] [-- <title arguments>]
xms config
```

`translate --jobs N` requires a positive worker count. Omit it to use the
default worker count.

Logging: `<span style="color:#ff2222">XMS</span>_LOG=xms_loader=trace,info` (a release build logs
info and above).

### Windows shader-test validator

The shader tests use `dxil.dll` to validate Shader Model 6.6 containers.
An older Windows SDK validator can load successfully but reject them with
`Unknown shader model 'ps_6_6'`. If this happens, use a current x64 validator
from [Microsoft's DirectX Shader Compiler releases](https://github.com/microsoft/DirectXShaderCompiler/releases)
or a current Windows SDK. Set its path for the test session:

```powershell
$env:XMS_DXIL = 'C:\tools\dxc\bin\x64\dxil.dll'
cargo test -p xms-shader --lib --locked -- --test-threads=1
```

Without this override, the validator loader selects the newest installed
Windows SDK's `bin/<version>/x64/dxil.dll`, then falls back to the DLL search
path when no SDK copy is found. This is a test/offline-validation dependency;
the translated shaders do not require it at runtime.

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p xms --features diagnostics   # target\diag\xms.exe
```

The on/off switches below (`...=1`) are off when unset or set to `0`.

| Variable | Effect |
|----------|--------|
| `<span style="color:#ff2222">XMS</span>_LOG=<target>=debug,...` | debug and trace output |
| `<span style="color:#ff2222">XMS</span>_STACKS=20,60` | every thread's stack at those seconds |
| `<span style="color:#ff2222">XMS</span>_WATCH=<rva>,...` | logs a title u32 when it changes |
| `<span style="color:#ff2222">XMS</span>_POKE=<rva>:<byte>,...` | writes a title byte twice a second |
| `<span style="color:#ff2222">XMS</span>_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `<span style="color:#ff2222">XMS</span>_CRASH_BREAKPOINTS=1` | appends first-chance breakpoints to `crashes.log` |
| `<span style="color:#ff2222">XMS</span>_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `<span style="color:#ff2222">XMS</span>_CAPTURE=<dir>` | saves presented frames |
| `<span style="color:#ff2222">XMS</span>_DUMP_TARGETS=1` | with the two above, saves every render target |
| `<span style="color:#ff2222">XMS</span>_CAPTURE_WORLD=<list>` | with `XMS_CAPTURE`, also saves world frames |
| `<span style="color:#ff2222">XMS</span>_DUMP_WRITERS=<hash>,...` | with the two above, textures a pipeline writes |
| `<span style="color:#ff2222">XMS</span>_PEEK=<hex address>[:<bytes>],...` | in listed frames, those bytes of title memory logged |
| `<span style="color:#ff2222">XMS</span>_D3D12_DEBUG=1` | host debug layer |
| `<span style="color:#ff2222">XMS</span>_D3D12_GBV=1` | with the above, GPU-based validation |
| `<span style="color:#ff2222">XMS</span>_D3D12_DRED=1` | on a GPU failure, the allocation hit and commands in flight |
| `<span style="color:#ff2222">XMS</span>_SKIP_PIPELINES=<hash>,...` | skip host pipeline creation for listed shader hashes |
| `<span style="color:#ff2222">XMS</span>_GPU_TIMING=1` | each list run and timed; slow lists logged with slowest commands |
| `<span style="color:#ff2222">XMS</span>_TRACE_SHADERS=<hash>,...` | every draw and dispatch of listed shaders logged |
| `<span style="color:#ff2222">XMS</span>_D3D12_WARP=1` | host device on software adapter (slow) |
| `<span style="color:#ff2222">XMS</span>_PIPELINE_LOG=<file>` | a line per title pipeline created, used, made or loaded |
| `<span style="color:#ff2222">XMS</span>_PREBUILD_KEYS=<file>` | `translate --cache --pipelines` lists each pack record |
| `<span style="color:#ff2222">XMS</span>_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane |
| `<span style="color:#ff2222">XMS</span>_AUDIO_CAPTURE=<file.wav>` | records what the title plays |
| `<span style="color:#ff2222">XMS</span>_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends unhandled exceptions, panics, and host GPU failures to
`crashes.log` in the title's data folder.

For a startup breakpoint (`0x80000003`), run the diagnostics build with
`<span style="color:#ff2222">XMS</span>_CRASH_BREAKPOINTS=1`. Keep the console output along with the new
`crashes.log` entries. A host DLL refusal includes the caller stack in the
console output.

On Series 1.0.2.1, a missing `appdata0.rpf` reproduced the breakpoint at
`+0x2237178` before the first window. Restore the archive from the same
package beside the title executable. `run --execute` checks that it can
read the RPF9 header before starting this build.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
