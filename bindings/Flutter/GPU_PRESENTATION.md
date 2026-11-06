# Flutter external texture control

This layer connects the existing `NativeDocument` worker owner to the desktop
texture plugins. It adds no document parser, input client, CPU renderer, or
Flutter-engine Skia pointer dependency. The host keeps its existing input,
selection, IME and semantics owner and supplies the same `NativePageViewport` to
both the input target and `NativeTextureView`.

The integration depends on the P89 snapshot ABI, P99 Linux plugin plus its GL
state restoration correction, P100 macOS plugin, P101 Windows plugin, and P102
owner dispatcher ABI 2. The native metadata capability
`presentation.fullStampChecks: true` is required. An older library, unsupported
platform, missing plugin, or exhausted GPU budget selects the supplied CPU
reference path. A document/resource replay error stays a visible diagnostic.

## Ownership and call boundaries

`NativeDocument.capturePresentation` sends configure and capture through its
existing owner FIFO after accepted edits. It captures an immutable snapshot with
the document ID, revision, font generation, view generation, device generation,
and device scale. `NativePresentation` calls only process-wide, thread-safe
capabilities and `rhwp_presentation_request` metadata operations:

```text
snapshotInfo    {op, snapshot}          -> {ok, info}
snapshotCheck   {op, snapshot, stamp}   -> {ok, current}
snapshotRelease {op, snapshot}         -> {ok}
```

The client never calls legacy HOST functions on the Dart thread. The owner
dispatcher and metadata client must use the same canonical native library path.
The standard `NativeOwnerTransport` exposes that identity; a custom transport
must explicitly implement `NativePresentationOwnerIdentity`. A capability bit
alone cannot establish that opaque handles belong to the same module.

The metadata module remains loaded after document/widget disposal so a late
capture can release its handle. Metadata replies are limited to 1 MiB and
requests to 64 KiB; native strings and temporary request bytes are freed even
when decoding fails. No pixel bytes or native device pointers cross a platform
channel.

One Dart UI isolate retains at most four caller-owned, capture-pending, or
ownership-ambiguous snapshots. A successful submit transfers each new snapshot
exactly once when `acceptedStamp` matches the full captured stamp. Where present,
`acceptedSnapshots` must also contain its ID. A definite rejection releases the
caller handle. Timeout, malformed response, transport loss, or an older status
does not prove rejection: a bounded reconciliation query may establish
acceptance, otherwise the handle stays retained. Late replies remain observed
after widget disposal. An uncertain release is not retried blindly.

There are at most three unresolved platform calls per controller and eight per
UI isolate. A timeout keeps its credit until the original call settles. Polling
is serialized, captures coalesce to the latest request, and publication has a
deadline. Terminal failure cancels recurring work. Recreating a controller does
not reset native process-wide resource budgets or confer ownership of ambiguous
handles.

## Platform receipts and visible frames

All plugins use `create`, `submit`, `status`, and `close` on these channels:

| Platform | Channel | Plugin class | Producer publication evidence |
| --- | --- | --- | --- |
| Linux | `rhwp_native/texture_linux` | `RhwpTexturePlugin` | matching `renderedStamp` |
| macOS | `rhwp_native/texture_macos` | `RhwpMetalPlugin` | `hasFrame` and matching `acceptedStamp` |
| Windows | `rhwp_native/texture_windows` | `RhwpD3dPlugin` | `hasFrame` and matching `renderedStamp` |

Creation sends only `{libraryPath}`. Submission sends
`{textureId, stamp, width, height, clearColor, placements}`. A placement contains
`{snapshot, scale, originX, originY, backgroundColor}`. Geometry uses the shared
96-DPI page transform multiplied once by device pixel ratio. Admission rejects
nonfinite geometry, dimensions above 4096, more than 4,194,304 target pixels, or
out-of-range scale/coordinates before allocating a snapshot. Platform budgets
can reject a smaller target, including Windows' charge for both GPU surfaces.

Linux first-submit readiness differs from macOS/Windows: its GL context is
created by `FlTextureGL.populate`, so the controller must not wait for an absent
`ready` key before submitting the first frame. An accepted submit alone never
uncovers a texture. After producer publication, the controller performs full
native `snapshotCheck`; `snapshotInfo.current` alone does not check the configured
view/device. It repeats the full check while displayed and compares the current
Dart document and presentation generation before every uncover.

`NativeDocument.rasterInvalidations` hides the old texture synchronously after
an edit is enqueued. Open, document replacement, revision/font changes, resize,
DPR change, explicit invalidation, viewport takeover, and close also cover it.
CPU `beginView` generations are independent: disposing a covered CPU view must
not invalidate a successful GPU view. Cached platform images may survive a
failed callback, so `Texture` stays in the paint tree **under an opaque cover**
while pending. Using `Offstage` or opacity zero would prevent Linux populate and
its normal cleanup path. Unmount may still leave a bounded native retire slot;
`close()` does not report GPU consumer completion.

Producer publication is not proof that Flutter imported or sampled the image.
In particular, macOS/Windows public callback lifetimes do not establish consumer
GPU completion. P100/P101 therefore expose
`continuousPresentationSupported: false` and fail closed after their immutable
frame budget is exhausted. They retain/quarantine published targets rather than
recycling them on callback return, unregister, close, or a device change. The
controller honors backpressure with CPU fallback and exposes no manual retire
operation. See ARCH-017/ARCH-018 and the platform technical documents for the
remaining consumer lifecycle qualification.

## Host wiring

Create one controller for one mounted viewport lifetime, outside `build`:

```dart
final metadata = await NativePresentation.connect(
  nativeLibrary: owner.nativeLibraryPath,
);
final controller = NativeTextureController(
  document: document,
  presentation: metadata,
  platform: NativeTexturePlatform.current()!,
);

NativeTextureView(
  controller: controller,
  viewport: sharedViewport,
  cpuFallback: buildExistingCpuViewport,
);
```

Check `NativeTexturePlatform.current()` for null before constructing this desktop
view. `package:rhwp_native_reference/rhwp_native_flutter.dart` exports these
adapters and `NativePageViewport`. The fallback builder belongs to the existing viewport
owner. It should not construct a second input/IME/semantics owner. Overlays may
remain outside the texture and need no raster invalidation when only selection
or caret state changes.

`NativeEditorView.viewportBuilder` receives `(context, document, page, viewport,
cpuFallback)`. Pass that exact viewport and fallback to `NativeTextureView` so
GPU pixels, pointer coordinates, caret overlays and IME candidate placement use
one transform. The example opts into this path with
`--dart-define=RHWP_GPU=true`; the default remains the CPU reference view. It
creates one controller outside `build` and closes/disposes it before the document.

Unmount closes the texture controller. Its creator must also call
`controller.dispose()` when releasing the notifier, and may await
`controller.close()` before disposing the document. Pending native/platform
completions keep their cleanup observers; neither awaiting close nor disposing
the notifier manufactures a consumer-complete signal. A terminal controller is
not automatically retried. The host can explicitly create a new one after a
meaningful recovery decision, subject to the unchanged native limits.

## Validation boundary

Mocked Flutter tests cover all three exact channel contracts, producer
publication followed by full stamp checking, FIFO capture after mutation,
immediate resize/source invalidation, independent CPU generations, late capture
and submit replies, disposal, definite rejection, ambiguous ownership, timeout,
RPC limits, diagnostic preservation, and adversarial viewport bounds. Widget
tests verify an opaque cover while pending and removal only after publication
and checking. These are adapter tests, not desktop compositor or screen-reader
qualification.

The pinned SDK is Flutter 3.47.9 (`c197ca5199e9ee5661e865596c9c07294dfab51c`),
Dart 3.13.7, engine `89c39edfdfae2949f6806e4a315c0c08522238b7`. Native full-metadata
and owner capture integration is validated separately against a fixed native
artifact. Real macOS/Windows SDK compilation, Flutter texture import/presentation,
OS accessibility/IME, GPU loss, and consumer retirement qualification remain
platform work. No shared `SkCanvas*`, private engine ABI, zero-copy claim, or
successful GPU sampling claim follows from these tests.

## Linux integration entry point

`test/support/linux_texture_probe.dart` is an executable entry point for a
disposable Linux Flutter runner, not an automatically discovered unit test.
Create a runner outside this package (`flutter create --platforms=linux`), add
`rhwp_native_reference` as a path dependency and `ffi: ^2.1.4` as a direct
dependency, and copy that entry point to the runner's `lib/main.dart`. Keep the
plugin's existing Linux CMake files; do not generate a runner over them. Build
the owner dispatcher with `tool/build_dispatcher.py --output <absolute-path>` and
build the runner with `flutter build linux --debug`.

Run the resulting bundle executable with
`RHWP_PROBE_NATIVE_LIBRARY=<canonical-absolute-native-library>` and
`RHWP_PROBE_OWNER_LIBRARY=<absolute-dispatcher-library>`. The native library must
enable the P89 worker snapshots and P90 GL presenter. The probe exercises an
initial Texture paint before capture, owner-thread new/edit/capture, actual
platform submit/publication, metadata verification, close, and disappearance of
the transferred snapshot ID. Its bounded JSON receipt starts with
`RHWP_LINUX_GPU_PROBE`. No document pixels travel through the method channel.

When `presentation.fullStampChecks` is present it requires `snapshotCheck` after
publication. A pre-P93 artifact instead reports `fullStampChecks: false` and
checks document/revision/font with raw `snapshotInfo` solely to qualify the
backend. This exception exists only in the test entry point; production
`NativeTextureController` still fails closed without full stamp checks. For an
ES 2 negative probe, select a renderer able to start on ES 2 and set
`RHWP_PROBE_EXPECT_UNSUPPORTED=1`. The probe requires explicit unsupported,
zero submitted frames, and normal close.

The actual Flutter 3.47.9 Linux runner exposed an EGL context with no current
`GdkGLContext`. P99's EGL/GLES correction is required: its desktop-only original
registered the texture but rejected its callback. With that correction, software
Mesa/llvmpipe replay and close succeeded on the default Impeller backend at GLES
3.0/3.2, and on the public Skia selection at GLES 3.2. Native strict-image/loader
artifact SHA-256 `26e81f691c34448e21c455107307aaf60e70e8e4dff8a2aab31574af2e97a56e`
was used for the final default-backend checks. Each rendered one edited document
frame, with zero whole-frame readbacks/release failures, no remaining raster
slots, `cleanupComplete: true`, and `registered: false`. A separate RAM-only GL
observer reported error zero before/after replay, after state restoration, and
on close; production code contains no GL-error drain. ES 2 was explicitly
rejected. Final pixel comparison and real GPU hardware qualification remain
separate from these results.
