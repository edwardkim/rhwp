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
view. Import `src/native_presentation.dart` and `src/native_texture_view.dart`
within the reference package; the host integration can re-export them from
`rhwp_native_flutter.dart`. The fallback builder belongs to the existing viewport
owner. It should not construct a second input/IME/semantics owner. Overlays may
remain outside the texture and need no raster invalidation when only selection
or caret state changes.

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
