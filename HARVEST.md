# Harvest from pull request #1 (Volt Launcher) — ledger

PR #1 is not a merge: its head is `entitybrian69-bit/Volt-launcher`, a rebranded parallel Android
app ("Volt Launcher") forked 6,785 commits behind this branch, whose foundation is the launcher UI
this port excludes. All eleven upstream merges it cites are already ancestors of `android`; its own
work is 143 non-merge commits. Resolving the PR as a merge would disable CI (six workflow renames),
resurrect removed subsystems (wasm mod SDK, wasmtime) and drag in a second launcher. The user chose
a harvest instead: port each non-frontend candidate onto this code base, one commit per feature,
verified by the `android` workflow on `feat/volt-harvest`.

The head's own launcher UI, its `catalog.rs`/`chooser.rs`/`shader_catalog.rs` pages, its Gradle
host (`packaging/android` Gradle files, `dev.horizon.android` activity), its 132-file `launcher/`
workspace, the wasm mod SDK (`horizon-mod-sdk`, `mods/official`, wasmtime), and its vendored
`third_party/winit` patch are **not** ported (out of scope by instruction, or removed by design —
see [SYNC.md](SYNC.md)).

## Ported

| Feature | Commit | Notes |
|---|---|---|
| F2 screenshots into the system gallery | `4f9df3e54` | `android/gallery.rs`: MediaStore copy into `Pictures/HorizonAndroid` (Android 10+), legacy `Pictures` with the runtime write permission (Android 9-), background thread, chat report through `horizon.gallery.*` keys. The game keeps its own copy in `screenshots/`. |
| Local diagnostics | `e8189768d` | `android/diagnostics.rs`: bounded, redacted log copy and session state in the game folder's `performance/`; clean-exit mark; next start warns if the last run died. Logger gained a `set_sink` hook (horizon-client-render). |
| Android exit history | `e8189768d` | `android/exit_reports.rs`: `ActivityManager.getHistoricalProcessExitReasons` (Android 11+) into `performance/android-exits.json`, with bounded system traces for crashes and ANRs and the last log's and crash reports' tails beside them. |
| Exit-history borrow + unused imports | `a848813fc` | Caught by this branch's own Android build. |

## Already covered by this code base (not ported)

- Touch F3/F5 buttons, editable custom keys, perspective: this tree's `touch.rs` (2,770 lines,
  editable `touch-layout.json`) already has `Button::Debug`, perspective and the layout editor.
- Winit Android mouse/wheel/modifier forwarding: this tree's input hook (`android.rs`'s `input`
  module) takes mouse sources and forwards keys to winit itself; no vendored winit patch needed.
- Mobile GUI-atlas packing: this tree packs the atlas within the GPU's real texture limit
  (`hud.rs`'s `atlas_limit`/`set_gpu_texture_side`, waiting on Android) and halves sprites under
  pressure (`layout_within`); the head's skyline packer is another solution to the same problem.
- Render-scale and aspect preferences: `graphics.rs` has `resolution_scale` (25–200%); the head's
  `display.rs` serves its launcher's settings pages.
- Accounts/Keystore, IME (TextMirror), gamepad haptics, DNS, secure store, prepare/install,
  import inbox, update installer: all exist here (`android.rs`, `android_lib.rs`, `StarlingActivity`).

## Candidates that need a decision (not ported)

- **GLES terrain fallback + optional Vulkan** (the head's largest renderer change): lets devices
  without Vulkan install and play, via a `terrain_gles.wgsl.inc` variant of the terrain shader
  selected per backend. Touches the manifest's `uses-feature` (Vulkan `required="true"` today),
  wgpu backend selection and shader generation; must be tested on a GL device before shipping.
- **SAF DocumentsProvider**: exposes the game folder to Android's file pickers. The head's
  allowlist models its own launcher's profiles and reports; ours would need a redesign for this
  game folder, and the game folder is already reachable through the storage setting and the
  import inbox.
- **TV / ARMv7 packaging**: leanback banner and category (manifest + drawable), 32-bit ABI.
  ARMv7 doubles the CI build; the GLES story above gates devices it would reach.
- **Performance recorder** (`performance/*.jsonl`) and the diagnostics export ZIP: their only
  triggers live in the head's launcher pages. A command-line or options-screen trigger would have
  to be designed here first; the native-tombstone summariser only feeds that export.

## Verify

```sh
gh workflow run android.yml -R johnrenonasuncion-Ramil/Rezyth-Launcher --ref feat/volt-harvest
```

Every port reuses this game's own hooks (`with_activity`, `platform::Hooks`, `logging`) and its
translation files (`horizon.gallery.*`); no head file was copied verbatim where its module layout
or its launcher's context differed.
