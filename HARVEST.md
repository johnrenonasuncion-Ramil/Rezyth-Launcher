# Harvest from pull request #2 (Volt Launcher) — ledger

**Analysis target updated 2026-11-08**: the user named **PR #2** the branch to harvest. PR #1
(still open, untouched) points at an older snapshot of the same fork; PR #2's head is the newer
one and is where the in-game **Volt Settings** hub and **Volt Boost** live.

- Head: `entitybrian69-bit/Volt-launcher`, branch `arena/0cae2b73-volt-launcher`, `9e855468`
  (2026-11-08, "Volt Replay: Replay Studio").
- Base: this repository's `android` @ `7c38f33c1` (Starling 0.2.3); merge base `c3905131`.
- Head-only: **339 commits**, 695 files, +162,084/−; dry-run merge: **418 conflicted paths**
  (87 content conflicts under `crates/`, 11 add/add, 10 rename/delete), same structural
  collision as PR #1 (old monolithic `lib.rs`, its own launcher frontend, workflow renames,
  wasm SDK) — so the verdict stands: **harvest per feature, never merge**.

The head's launcher frontend, its Gradle host, the wasm mod SDK and the vendored winit patch stay
**out** (denied by instruction, or removed by design — see [SYNC.md](SYNC.md)). Everything below is
harvested from `9e855468`.

## Where the requested features live in the head

- **"Volt Settings" button and screen (in-game)**: `options_ui.rs` (`Page::VoltSettings`,
  a frosted-glass hub on the Options screen), `options_volt_pages.rs` (789 lines: `VoltVisuals`,
  `VoltQol`, Volt Boost, Volt Waypoints, Volt Minimap, Volt Shaders pages), `graphics_style.rs`
  (`volt_settings`, `volt_mods` styles), and ~85 switches as `extras::Item` variants in
  `extras.rs` (3,195 lines). The launcher's own copy of the same page (`launcher_pages.rs`
  "Launch settings"/Boost) is the denied frontend and is not ported.
- **Volt Boost ("the optimizer with profiles")**: `volt_features.rs` §4 — `VoltBoostPreset`
  (`Off`, `Balanced`, `Competitive PvP`, `Ultra FPS`, `Battery Saver`, `Low-RAM Rescue`) plus the
  switches behind it (ADPF, big.LITTLE thread boost with timer slack, Volt Reflex, pipeline
  prewarm, dynamic menu/idle FPS, chunk-upload cap, entity/item/block-entity caps, LOD boost,
  particle budget, RAM purge, thermal guard, keep-awake...). `horizon-threads` gains
  `set_thread_boost` (Android `PR_SET_TIMERSLACK`).
- Also in `volt_features.rs` (1,563 lines): **Volt Waypoints** (store, death waypoints,
  Overworld↔Nether sync, projection) and **Volt Minimap** (frames, terrain modes, entity radar,
  world clock) — engines first, their in-game pages after.
- **The native mods (30+)**: not in-repo sources — **33 official registry mods** woven into the
  APK at build time (`bundled_native.rs` `WOVEN`: jei, horizon_physics, jade, veinminer,
  appleskin, status_effect_hud, toggle_sprint, inventory_profiles, shulker_peek, mousetricks,
  itemnew, eatinganimation, enchantmenthints, light_overlay, right_click_crops, smartf3,
  armor_status, crosshair_indicator, hitbox_plus, invisible_frames, freelook, voicechat,
  worldedit, horizonmatica, visuality_horizon, held_item_info, carryblock, chatheads, fast_items,
  customhud, coords_hud, biomecompass, skins_3d), plus 4 compiled-in engine mods (`BUNDLED`:
  zoom, fps_overlay, keystrokes, minimap). The head's `weave` job downloads the pinned `.hnmod`
  bundles from `horizon-rust/horizon-mods` releases, verifies each against the registry snapshot
  (`docs/native-mods/horizon-registry-63daa89b…json`, 799 lines), and weaves all of them into the
  engine library in **one** horizon-mixins pass (`tools/native_mods/compat.py` 543 lines,
  `build_set.py` 201, their tests 444). The woven mods register on the in-game Mods page through
  the `native_mods` hook point and are on until the player turns them off.

## Ported (verified by the `android` workflow on `feat/volt-harvest`)

| Feature | Commit | Notes |
|---|---|---|
| F2 screenshots into the system gallery | `4f9df3e54` | `android/gallery.rs`: MediaStore copy into `Pictures/HorizonAndroid` (Android 10+), legacy `Pictures` with the runtime write permission (Android 9-), background thread, chat report through `horizon.gallery.*` keys. The game keeps its own copy in `screenshots/`. |
| Local diagnostics | `e8189768d` | `android/diagnostics.rs`: bounded, redacted log copy and session state in the game folder's `performance/`; clean-exit mark; next start warns if the last run died. Logger gained a `set_sink` hook (horizon-client-render). |
| Android exit history | `e8189768d` | `android/exit_reports.rs`: `ActivityManager.getHistoricalProcessExitReasons` (Android 11+) into `performance/android-exits.json`, with bounded system traces for crashes and ANRs and the last log's and crash reports' tails beside them. |
| Exit-history borrow + unused imports | `a848813fc` | Caught by this branch's own Android build (run #42456739633). |
| Harvest ledger | `b0e7112e3` | This file, first written for PR #1, now aimed at PR #2. |

CI: `android.yml` run **42457716326** built the APK green at `a848813fc`.

### Follow-ups from the newer head

- `gallery.rs` evolved after PR #1's snapshot: `save(file, copy_image)` also puts the MediaStore
  **content URI on the clipboard** (`ClipData.newUri`) after publishing. Fold into our port as an
  optional extra step behind the same background copy.

## Already covered by this code base (checked in `9e855468`, not ported)

- Touch F3/F5 buttons, editable custom keys, perspective: this tree's `touch.rs` already has
  `Button::Debug`, perspective and the editable `touch-layout.json` editor.
- Winit Android mouse/wheel/modifier forwarding: this tree's input hook (`android.rs`'s `input`
  module) takes mouse sources and forwards keys itself; no vendored winit patch.
- Mobile GUI-atlas packing: this tree packs within the GPU's real limit (`hud.rs`
  `atlas_limit`/`set_gpu_texture_side`) and halves sprites under pressure.
- Render scale/aspect preferences: `graphics.rs` `resolution_scale` (25–200%).
- Accounts/Keystore, IME (TextMirror), gamepad haptics, DNS, secure store, prepare/install,
  import inbox, update installer, native-mod build service: all exist here.

## Candidates that need a decision (not ported)

- **GLES terrain fallback + optional Vulkan** (head's largest renderer change): lets devices
  without Vulkan install and play (`terrain_gles.wgsl.inc` variant, manifest `uses-feature`
  change, wgpu backend selection). Must be tested on a GL device; gates ARMv7/TV reach.
- **SAF DocumentsProvider**: exposes the game folder to Android's file pickers; the head's
  allowlist models its launcher's profiles, ours would need a redesign.
- **TV / ARMv7 packaging**: leanback banner/category (manifest + drawable), 32-bit ABI; ARMv7
  doubles the CI build.
- **Performance recorder / export ZIP / tombstone summariser**: triggers live in the head's
  launcher pages; an in-game trigger must be designed here first.

## Plan for the named priorities (Volt Boost first)

1. **Engines** — DONE: `volt_features.rs` ported into `horizon-client-render` and re-exported as
   `horizon_client::volt_features` (`b68b91738`, module declaration fixed `28582b3d0`; its
   dependencies exist here unchanged, and the minimap projection inverts this tree's own
   `compose`). Off by default; engines carry their own tests.
   **Volt Boost's big.LITTLE thread tuning** also landed (`28582b3d0`): `horizon-threads`' Android
   `favour` arm (50 us `PR_SET_TIMERSLACK`, on until `set_thread_boost`), the persisted
   `extras.threadBoost` switch, applied at the Android start. CI: run **42463514240** green.
2. **Volt Boost switches and effects** — the named set is complete
   (`39eb25aa6`–`c0b7bbf40`, CI runs **42467361511**–**42479850190** green). Landed: the
   persisted switches (`performanceHints`, `displayHz`, `threadBoost`, `voltPreset`, `dynamicFps`, `particleBudget`, `audioVoiceCap`, `autoDrs`, `thermalGuard`, `voltReflex`, `entityCap`, `itemRenderCap`, `blockEntityCap`, `menuBgMode`) and their effects — the ADPF
   gate with a proper session close, the display-rate request, the big.LITTLE timer-slack
   tuning, the frame-limiter idle cap, the particle budget (cap, distance cut, debris stop),
   the particle-sound voice cap, Auto-DRS and the thermal guard (resolution factor through the
   world post chain, particle thinning while hot), Volt Reflex (a device poll before the
   frame's sleep, Precise pacing while playing), the entity/item/block-entity nearest-first
   render caps, and the menu backdrop saver (FastDark skips the multi-pass blur).
   `apply_volt_preset` sets every one of them per the head’s profiles. Not ported, with reasons:
   the chunk-upload cap (our `UploadBudget` graphics setting already budgets uploads per frame,
   and the head’s own switch has no consumer in its tree), pipeline prewarm (also without a
   consumer in the head), and the Volt Settings hub UI (step 3, which makes every switch
   reachable in-game).
   **Tier 2/3 sweep complete** (`2cc6ad735`–`8c3a328ff`, CI runs **42484393177**–
   **42495038054** green): the animation throttle and active-animation cull, fast translucency,
   fast weather, the sign-text cull, the mob throttle, the hub shield, the chaos shield, the
   particle frustum cull, the memory saver, the streaming lookahead, the cave sky cull and
   frustum-priority meshing all landed with their engine features and their profile values.
   The `volt_boost_plus` mesh-pool group stays out: its five switches have no consumer in the
   head’s tree either (stored only), like pipeline prewarm.
3. **In-game UI** — DONE (`5dcf4008c`, fixed `fd9ec72d4`, CI run **42500485131** green):
   the Volt Settings hub (`Page::VoltSettings`) and its Boost page (`Page::VoltBoost`) on the
   game’s own Options screen, with every ported switch (27 items: the profile and all of them)
   two to a row through the same caption/value/cycle machinery the Extras page uses, off a
   translation key each (100 new bundled en_us keys). The Options screen gets a Volt Settings
   entry; the main menu gets a Volt Settings button that opens the same hub (in the game and
   the startup menu alike). The profile applies at once and the thread tuning follows the
   switch. The hub has room for the Waypoints and Minimap pages when their engines’ UI lands.
4. **The native-mod weave** — DONE: `tools/native_mods/compat.py` (registry verification:
   SHA-256 against the pinned snapshot, revocations, minisign when published, a bounded safe
   unzip, matching manifests) and `tools/native_mods/weave.py` (staging + the hook-API gate)
   ported from the head; the dispatch-only `.github/workflows/weave.yml` fetches the pinned
   registry, downloads the release bundles, verifies them and builds the APK through this
   repository’s own weaver (`packaging/android/native_build.sh`, one horizon-mixins pass, with
   `native_mod_adapt.py` bringing the mods’ calls up to date; the compiler runs at the newest
   consumer pin, Skins 3D’s, and vendored crates propagate into horizon-android).
   **All 33 weave and build green** (CI run **42521156965**, artifact `woven-apk`, 33.6 MB;
   every bundle staged, none rejected): jei, horizon_physics, jade, veinminer, appleskin,
   status_effect_hud, toggle_sprint, inventory_profiles, shulker_peek, mousetricks, itemnew,
   eatinganimation, enchantmenthints, light_overlay, right_click_crops, smartf3, armor_status,
   crosshair_indicator, hitbox_plus, invisible_frames, freelook, voicechat, worldedit,
   horizonmatica, visuality_horizon, held_item_info, carryblock, chatheads, fast_items,
   customhud, coords_hud, biomecompass, skins_3d.
   The last three needed their engine features ported first, all landed here: `Mob.skin_3d`,
   the `remote::Game::skin_3d_model` hook point (called where a player’s skin is applied,
   drawn in place of the flat outer layer) and `Downloader::image` for Skins 3D;
   `NativeShaders::picked`/`picking` and the pick-by-name wiring in `shader_packs.rs` for
   Visuality Horizon and Horizon Physics. The weave also drove the stale `Cargo.lock` fix, the
   `thread_local!` move out of `game.rs`’s top level (the compiler’s macro policy) and five
   `native_mod_adapt` rules.
## Verify

```sh
gh workflow run android.yml -R johnrenonasuncion-Ramil/Rezyth-Launcher --ref feat/volt-harvest
```

Every port reuses this game's own hooks (`with_activity`, `platform::Hooks`, `logging`, the
Options screens) and its translation files; no head file was copied verbatim where its module
layout or its launcher's context differed.
