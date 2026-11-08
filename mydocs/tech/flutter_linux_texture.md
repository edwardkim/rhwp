---
kind: reference
status: active
canonical: mydocs/tech/flutter_linux_texture.md
last_verified: 2026-11-07
---

# Linux Flutter GL texture adapter (P99)

`bindings/Flutter/linux` implements the public Linux `FlTextureGL` plugin callback.
It renders P89 immutable page snapshots through the P90 native GL presenter into
plugin-owned RGBA8 texture/FBO pairs. It does not use a browser, document layout
on the platform/raster thread, engine Skia pointers, a Flutter-private reset API,
CPU pixel buffers, Vulkan, or cross-context texture export.

## Thread and module ownership

- P97's dedicated native owner thread retains all legacy `NativeHost` TLS,
  worker submit/poll, document, and buffer operations. Dart isolates and GTK's
  platform thread must not call those handles directly. P102 must extend the
  dispatcher with typed `presentationConfigure`/`exportRenderSnapshot` commands.
- The GTK platform thread registers textures and handles the bounded method
  channel metadata below. `snapshotInfo` and `snapshotRelease` are P89's explicit
  cross-thread registry operations. The capability query is a handle-free query.
- Only `FlTextureGL.populate`, with Flutter's current EGL/GLES 3 or desktop
  GL 3.3 context, creates/renders/releases a P90 presenter. A GDK context is an
  optional retained owner; EGL/GLX current identity and raster OS thread are
  authoritative. A texture cannot acquire
  a second raster OS thread owner while its original owner lives. Each context
  has a distinct nonzero identity. A context switch or raster-owner exit requires
  the next device generation and a newly configured presentation stamp.
- The same canonical absolute native library path must be used by P97 and this
  plugin. `dlopen` resolves that same process module; the plugin never statically
  links a second native registry. A different path is rejected after the first
  accepted module. The native module, GL loaders, and plugin callback code remain
  mapped for process lifetime, including native/raster TLS teardown.

The [public callback header](https://api.flutter.dev/linux-embedder/fl__texture__gl_8h_source.html)
states that Flutter makes its GL context current before `populate`. The
[Linux implementation](https://api.flutter.dev/linux-embedder/fl__texture__gl_8cc_source.html)
exposes RGBA8 and no texture destruction callback. Flutter's
[GL embedder texture implementation](https://github.com/flutter/flutter/blob/master/engine/src/flutter/shell/platform/embedder/embedder_external_texture_gl.cc)
flushes its Ganesh work and invalidates its cached GL state before calling the
external callback; it borrows the result with TopLeft origin and premultiplied
RGBA. This adapter submits to that same current GL command queue and returns the
texture on that raster callback. It does not infer a cross-context fence from
`submit`, nor access Flutter's `GrDirectContext`. The public OpenGLES texture path
also supports Flutter's Impeller renderer; Vulkan and arbitrary external
consumers are outside this contract.

Flutter 3.47.9의 고정
[Linux GL manager](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/linux/fl_opengl_manager.cc)는
EGL context를 직접 생성하며 raster thread에 `GdkGLContext`를 등록하지 않는다. 따라서
`gdk_gl_context_get_current()`가 null이어도 유효한 callback이다. 이 구현은 current EGL
context를 먼저 확인하고, GLX 경로도 유지한다. 실제 API profile을 조회해 desktop GL 3.3
이상 또는 GLES 3.0 이상만 허용한다. 현재 engine의 EGL context는 GL manager 생성부터
해당 engine의 종료까지 유지되며 adapter가 소유·교체·파괴하지 않는다. owner thread와
현재 native context를 callback마다 다시 확인하고, 다른 context에서는 이전 presenter의
GL 함수를 호출하지 않는다. engine 종료 뒤 callback이 없으면 아래의 bounded abandon
규칙을 적용한다. public `fl_dart_project_set_enable_impeller(FALSE)`는 renderer만 바꾸며
이 EGL/GLES context를 desktop GL로 바꾸는 선택지가 아니다.

`GlStateGuard` additionally restores draw/read FBO, renderbuffer, viewport,
scissor, program, VAO/buffer bindings, blend/depth/stencil settings and masks,
pixel-store state, and texture/sampler bindings for at most 256 texture units.
The native presenter resets its own independent Ganesh state on every replay.
The guard uses ES float depth functions and profile/version-specific binding
and enable lists. It never issues desktop-only polygon mode, double depth,
1D/rectangle texture, or desktop enable queries on GLES. Its public tests cover
real desktop GDK and direct EGL GLES contexts, including a raster thread without
a current GDK context. Those state tests alone are not a Flutter compositor or
native document replay qualification. Capturing all
texture-unit bindings has a bounded CPU cost that still needs application
profiling. It makes no claim about unsupported GL extensions or other backends.

종료·resize에서 plugin 소유 texture/FBO를 실제로 삭제했다면 `GlStateGuard`의 해당 이름도
제외한다. 삭제된 이름을 다시 bind하면 `GL_INVALID_OPERATION`이 발생하거나 나중에 재사용된
이름에 잘못된 객체를 바인딩할 수 있다. 현재 callback context의 활성 guard에 있는 동일
texture 이름과 draw/read FBO 이름만 0으로 바꾸며 다른 live 객체·namespace·caller 상태는
보존한다. 실제 GDK GL 회귀는 중첩 guard에서 texture/draw FBO 삭제 후 error가 없고 다른
texture unit·read FBO가 그대로 유지되는지 검사한다.

## Method channel

Channel: `rhwp_native/texture_linux`, standard method codec. Integer fields use
nonnegative signed 64-bit values, dimension/color fields must fit u32. Arguments
are maps; pixel positions/scales may be finite doubles.

`create({libraryPath: absoluteCanonicalPath})` returns a map containing
`textureId`, `deviceGeneration` (initially 1), and status. It requires the native
`gpuGl.hostFboRendering` and `presentation.workerRenderSnapshots` capabilities.
Only two registered/retiring textures are retained by one plugin. The native
module path must match the P97 owner dispatcher.

`submit` takes the complete physical viewport, not a document handle:

```json
{
  "textureId": 1,
  "stamp": {
    "documentId": 7, "revision": 3, "fontGeneration": 2,
    "viewGeneration": 10, "deviceGeneration": 1, "deviceScale": 1.5
  },
  "width": 1200, "height": 900, "clearColor": 4283783002,
  "placements": [
    {"snapshot": 19, "scale": 1.875, "originX": 18, "originY": -10.5,
     "backgroundColor": 16777215}
  ]
}
```

Before `submit`, the P97 owner dispatcher configures the P89 worker presentation
source with exactly the same full stamp and exports the requested screen page
snapshots. `scale = zoom * deviceScale`; origins are NativeView `pageRect` logical
coordinates multiplied by device scale. NativeView remains responsible for page
visibility and input coordinate mapping. View generation must strictly increase
on each submission, including source/viewport/device changes. Snapshot source
must match document/revision/font, profile must be screen, and `current` must be
true. Registry validation runs again in native replay, so a later source change
does not publish a stale native frame.

A successful response has `accepted: true` and `acceptedSerial`. **At that
point ownership of each new distinct snapshot ID has transferred to the plugin.**
Reusing an ID already held by that texture borrows its existing lease. The caller
must not release an accepted ID. Duplicate placements share one lease. Snapshot
IDs that are no longer in the latest or in-flight frame are released, including
after close or a superseding source. A rejected method response does not consume
new IDs; the caller releases those IDs. It must not release existing plugin-owned
IDs just because a later view submission using them was rejected.

Serialize submissions and keep the ownership ledger through asynchronous Dart
disposal. A successful reply arriving after disposal still transfers ownership:
close that texture rather than releasing its accepted snapshots. A transport
failure is ambiguous; query `status.acceptedStamp` for the unique submitted view
generation before releasing IDs. If the channel/engine is already gone, do not
guess that ownership reverted. Plugin/owner shutdown retains native lifetime
safely; no original document is written or deleted by this adapter.

A producer snapshot handle has one owner. Do not transfer the same raw handle to
two textures; export another producer handle for the second texture. Reusing a
previously accepted handle in the same texture uses its existing lease.

`status({textureId})` returns the latest bounded status:

- `acceptedSerial`/`acceptedStamp` and `renderedSerial`/`renderedStamp`;
- `deviceGeneration`, `framesSubmitted`, `wholeFrameReadbacks`;
- `closing`, `cleanupComplete`, `registered`, `ownerThreadExit`;
- `liveRasterSlots`, `pendingRetireSlots`, `orphanedRasterSlots`, `rasterSlotLimit`
  (4 process-wide);
- bounded `code`/`message`, per-texture `orphanedOwners`, and `abandonedTargets`.

`snapshotReleaseFailures` is a process-module counter for failed P89 release
responses. Such a failure is exposed and prevents a successful `cleanupComplete`
claim; the process-wide native registry quota still bounds retained snapshots.

There is no unbounded event/result queue. Poll at a bounded UI cadence. Source or
stamp rejection uses `staleFrame`/`contextChanged`; query current owner info/device
generation and configure/export/submit again. Strict replay/resource errors keep
the last successful texture and a diagnostic. The plugin does not silently turn
such errors into a CPU placeholder. GPU/device errors remain explicit for the
application's renderer policy. A native whole-frame readback is a contract error
and is reported rather than counted as a successful GPU frame.

The maximum physical dimension is 4096, viewport area is at most 4,194,304 pixels,
and there are at most eight placements. Each texture holds one latest frame and
one in-flight frame, with at most sixteen distinct snapshot leases. Input is
validated before ownership changes. Two texture/FBO targets permit rejecting a
superseded in-flight result without replacing the last-good front texture. Normal
texture reuse/sampling is ordered on one GL queue; another texture callback is
not treated as a sampling-completion fence. Native presenter caches and scratch
surfaces have their own explicit quotas. These bounds are not a total RSS/VRAM
measurement, and process-wide P89 quotas may reject work sooner.

## Close and engine teardown

`close({textureId})` is asynchronous. It stops new submissions, drops unused
snapshot leases, and asks Flutter for another frame. While this texture is still
sampled, its own `populate` releases its presenter and deletes its targets under
the matching current context, then posts unregister to the original main context.
An [explicitly attached idle source](https://docs.gtk.org/glib/method.Source.attach.html)
performs that handoff; GLib
[`invoke_full`](https://docs.gtk.org/glib/method.MainContext.invoke_full.html)'s
possible synchronous shortcut is not used on the raster thread.
No UI thread waits for document/GPU work or calls a foreign-thread GL handle.
Keep the Texture widget mounted until `cleanupComplete` and `registered == false`
when normal close is possible. Query closed status before creating another
texture; creating may prune completed records.

If the widget is already offscreen/unmounted, or engine shutdown means no further
`populate`, public FlTextureGL provides no raster release callback. Plugin dispose
unregisters without blocking, but **does not report raster cleanup as complete**.
The bounded raster slot retains native module, context, bridge, and callback
userdata until its original OS thread's TLS cleanup, which abandons without GL or
callback calls. `pendingRetireSlots` stays nonzero and allocation is refused at
four live/deferred slots. A failed native cleanup also keeps its slot/callbacks
until an owner-thread retry or TLS teardown. Context-lost target names are counted
as `abandonedTargets` and reclaimed by context destruction, not by an unsafe
foreign-context `glDelete`. A UI timer or another texture's populate is not a
release fence. This is a public Linux API limitation, not a successful-release
claim or permission to reuse targets across contexts.

Abandon cannot prove that all GL names were reclaimed while a driver share group
survives. Therefore an abandoned owner consumes a process-lifetime **orphan credit**
even after its callback/native TLS state is gone. `orphanedRasterSlots` and the
texture's `orphanedOwners` expose this; that texture's `cleanupComplete` remains
false. The texture can be unregistered after native owner teardown, but its orphan
credit is not silently recycled. At four orphans, GPU allocation requires process
restart (the application can select CPU). This conservative bound prevents
repeated offscreen-close/context-loss cycles from accumulating unlimited opaque
driver resources. Normal matching-current release/delete returns its credit.
Public FlTextureGL offers no completion fence or share-group-destruction proof
that would safely relax this policy. These cases need application lifecycle
qualification, rather than a claim of complete GPU resource recovery.

첫 snapshot이 도착하기 전에 Texture가 그려지는 `notReady`와 정상 종료의 `closing`은
persistent renderer 오류로 기록하지 않는다. 하지만 callback의 FALSE 반환에는 항상
`GError`를 제공한다. 고정
[Flutter callback consumer](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/linux/fl_engine.cc#L466-L469)는
실패 시 오류 메시지를 읽으므로 null 오류 객체를 정상 대기의 의미로 반환하면 안 된다.

Flutter 3.47.9 revision `c197ca5199e9ee5661e865596c9c07294dfab51c`의
[GL texture 경로](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/embedder/embedder_external_texture_gl.cc)는
`MarkNewFrameAvailable`에서 이전 `last_image_`를 비운 뒤에만 새 populate를 호출한다.
그 callback이 실패하면 같은 Paint에서 이전 cached image를 다시 그리지 않는다. 따라서 현재
Ganesh/current-GL 명령 순서와 이 경로를 대조한 검증을 macOS의 이전-image 유지 동작과 혼동하지
않는다. 이 사실이 다른 context, 임의의 consumer, offscreen/TLS abandon의 완료를 증명하지는 않는다.

부모 Dart는 document/revision/font/view/device 변경이나 오류에서 `renderedStamp`와 현재 입력을
대조하고 이전 문서의 Texture를 숨겨야 한다. 새 frame 알림 전에는 엔진이 기존 이미지를 그대로
쓸 수 있고, adapter도 replay 실패 시 last-good front를 반환할 수 있다. Texture를 숨겨 populate가
더 이상 오지 않으면 위의 bounded pending-retire 경로를 사용하며 UI에서 cleanup 완료를 기다리면
안 된다. last-good 화면을 현재 source가 검증된 새 frame으로 간주하지 않는다.

## Parent/P102 integration

P99 deliberately changes neither production Dart nor `pubspec.yaml` nor the P97 dispatcher.
In the plugin package's pubspec, add the Linux registration stanza:

```yaml
flutter:
  plugin:
    platforms:
      linux:
        pluginClass: RhwpTexturePlugin
```

A separate Linux Flutter runner/example must depend on the package; do not run
`flutter create .` over `bindings/Flutter/linux/CMakeLists.txt`, which is the
plugin build definition. Flutter's generated registrant then includes
`<rhwp_native_reference/rhwp_texture_plugin.h>`, obtains the
`RhwpTexturePlugin` registrar, and calls
`rhwp_texture_plugin_register_with_registrar`. The CMake target is
`rhwp_native_reference_plugin`. A manual runner can perform the same registration
after its Flutter view/registrar exists and link that target. Pass the same
canonical native library path used by `NativeOwnerTransport`; no native module
is bundled or built by this plugin's CMake.

The Dart view should create `Texture(textureId: id)`, share NativeView viewport
and overlays/hit testing, and forward only snapshot/stamp metadata over this
channel. P102 must add owner-thread snapshot configure/export RPCs, track ownership
through late replies, react to device generation changes, choose explicit GPU/CPU
policy from capabilities/diagnostics, and perform asynchronous close. Actual app
IME overlays, pixel comparison of Flutter's sampled output, and real hardware
remain integration/qualification work rather than results of the portable tests below.

## Checks

Portable fake-ABI lifetime checks require only a C++17 compiler and pthreads:

```bash
python3 -B bindings/Flutter/linux/tests/check_linux_plugin.py --output /new/path/p99-checks
```

With GTK3/epoxy development packages and a Flutter Linux engine artifact directory:

```bash
python3 -B bindings/Flutter/linux/tests/check_linux_plugin.py \
  --output /new/path/p99-checks --flutter-engine /flutter/bin/cache/artifacts/engine/linux-x64 \
  --gl-state --gles-state
```

`--gl-state` needs a working display, for example a separately managed Xvfb with
`GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1`. `--gles-state` creates a directly current
EGL ES 3 context without GDK TLS. The same restoration/deleted-name tests run on
both profiles; ES depth/primitive-restart state is also changed and restored.
These report software Mesa GL state restoration, not hardware/native-Skia
rendering. `--dev-root` accepts a task-local
unpacked development prefix; `--native-include` can select a P90 public header
before source integration. The script compiles/links the actual plugin against
real Flutter headers/library with undefined symbols rejected. It does not invoke
Cargo or a full Flutter app build.

## 실제 Flutter EGL 회귀

Flutter 3.47.9 (`c197ca5199e9ee5661e865596c9c07294dfab51c`, Dart 3.13.7,
engine `89c39edfdfae2949f6806e4a315c0c08522238b7`)의 Linux runner를 별도로
생성하고 plugin을 실제 generated registrant에 연결했다. 최초 구현은 등록만 성공했고,
실제 callback에서 `gdk=0`, current EGL 존재, GLES 3.2인 조건을 거절했다. 기존 desktop
state guard를 direct EGL에서 실행하면 `glGetDoublev`를 호출하다 중단되는 RED도 보존했다.
단순히 context 검사만 제거하는 보정으로는 해결되지 않는다.

수정 후 C++17 `-Wall -Wextra -Werror` public-header compile/link, portable lifetime
13개 경우·226개 검사, desktop GL 4.5와 직접 생성한 GLES 3.0/3.1/3.2 state 복원 및
삭제된 texture/FBO 회귀가 통과했다. 실제 Flutter runner는 owner dispatcher를 통해 새
문서를 만들고 편집한 뒤 snapshot을 capture하고, 같은 process registry를 쓰는 plugin에
소유권을 이전했다. 기본 Impeller의 GLES 3.0/3.2와 public Skia 선택의 GLES 3.2에서
producer publication과 정상 종료를 확인했다. `MESA_GLES_VERSION_OVERRIDE=2.0`의
Skia runner에서는 명시적인 unsupported, 제출 frame 0, 정상 종료를 확인했다.

최종 native artifact는 P90 strict-image/loader 보정을 포함하며 SHA-256은
`26e81f691c34448e21c455107307aaf60e70e8e4dff8a2aab31574af2e97a56e`이다.
기본 Impeller 검증은 이 artifact를 사용했고, 앞선 public Skia 경로는
`0090604f98a05bf34bed4079e24677264251707843e4f44b8c1ccf2e26dbb02b`를 사용했다.
각 성공 실행의 frame 수는 1이고 whole-frame readback은 0이었다. 종료 후 snapshot ID가
registry에서 사라졌고 release failure·live/pending/orphaned raster slot은 모두 0,
`cleanupComplete == true`, `registered == false`였다. 해당 artifact에는 P93 full stamp
check가 아직 없으므로 native `snapshotInfo.current`와 document/revision/font만 대조한
backend 회귀이며, P102 production controller의 full-stamp 요구를 낮추지 않았다.

production source의 성공 실행과 별도로 RAM에만 GL-error 관측을 추가한 runner도 실행했다.
GLES 3.0/3.2의 첫 frame 이전 warmup, native replay 전후, state guard 복원 뒤, 정상 종료에서
모두 GL error 0이었다. 이 관측은 배포 source에서 GL 오류를 소비하거나 renderer 동작을
바꾸는 API로 남기지 않는다.

이 실행은 Xvfb의 software Mesa/llvmpipe 검증이다. 실제 하드웨어 가속, Flutter가 샘플링한
최종 픽셀의 시각 비교, offscreen/context-loss/engine-restart의 OS qualification을
완료한 것으로 보고하지 않는다. 테스트 runner·로그·source hash는 독립 증적에 보존하며,
재현용 P102 entry point는 owner ABI 2와 `NativeDocument.capturePresentation`이 있는
참조 패키지에서 실행한다. 이 entry point는 테스트 process 안에서만 pre-P93 raw metadata
경로를 허용하며 앱의 renderer 정책을 우회하는 production fallback API가 아니다.
