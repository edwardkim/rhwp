---
kind: reference
status: active
canonical: mydocs/tech/flutter_windows_texture.md
last_verified: 2026-11-08
---

# Windows immutable D3D Flutter bridge

The opt-in backend implements a D3D12/Ganesh producer and Windows Flutter GPU
surface adapter. The #8739 integration recorded actual Windows SDK/native D3D12
producer checks at source `263b1e753c006d1631d25794588b6638fa1172ae` in
`mydocs/pr/assets/seo-rii-20261107/platform-execution/windows-final-correction.json`.
That earlier result does not qualify a newer plugin, Flutter consumer sampling,
or the ownership-transfer mode described below. New target execution must use
matching source/artifact receipts and `bindings/Flutter/GPU_QUALIFICATION.md`.

The default `legacyImmutable` mode retains four immutable/quarantined credits for
process lifetime. Explicit `immutableOwnershipTransfer` mode continuously
publishes fresh immutable allocations and releases creator references only after
opening and CPU descriptor quiescence. Neither mode claims consumer GPU
completion or rewrites/recycles a published target. No D3D12 pointer is passed as
a Flutter D3D11 resource.

## Why the bridge has a GPU copy

Locked `skia-safe` 0.156.0's `gpu::d3d::BackendContext` uses `IDXGIAdapter1`,
`ID3D12Device`, and `ID3D12CommandQueue`; its target is `ID3D12Resource`. Flutter's
public texture descriptor accepts a legacy DXGI shared handle or an
`ID3D11Texture2D*`, not that D3D12 interface. The public graphics adapter API gives
an adapter, not Flutter/ANGLE's actual D3D11 device. The
[ANGLE texture-client extension](https://github.com/google/angle/blob/main/extensions/EGL_ANGLE_d3d_texture_client_buffer.txt)
requires a direct D3D11 texture pointer to belong to the display's device when the
device extension is present. Matching adapter LUID alone does not satisfy that
same-device requirement.

The implemented path uses only public interfaces:

1. Obtain Flutter's adapter with `FlutterDesktopPluginRegistrarGetGraphicsAdapter`.
2. On a dedicated producer thread, create a D3D12 device/direct queue and a
   D3D11.1 device on that adapter. Compare the actual adapter LUIDs; reject
   multi-node or mismatching devices.
3. Create a shared DEFAULT-heap BGRA8 D3D12 resource with render-target and
   simultaneous-access flags, single sample/mip/layer, initially COMMON. Create
   its **NT handle**, then `ID3D11Device1::OpenSharedResource1` on the D3D11 device.
4. Native strict replay finishes into scratch before drawing into the shared
   target. Check flush and submit, then signal and wait a separate monotonic
   D3D12 queue fence. The simultaneous-access flag guarantees COMMON decay after
   completed command-list execution, including explicit transitions.
5. GPU-copy the opened source into a fresh D3D11 texture with legacy shared
   resource flags. Flush and wait for a `D3D11_QUERY_EVENT`, checking device
   removal and timeout. Neither wait is a Flutter consumer fence.
6. Publish that destination's **legacy** `IDXGIResource::GetSharedHandle` through
   `kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle`. Do not substitute the NT
   handle or a D3D12 resource pointer. Legacy handle values are borrowed from the
   resource and are not passed to `CloseHandle`; the created NT handle is owned.

Relevant Microsoft contracts: [shared D3D12 heaps](https://learn.microsoft.com/en-us/windows/win32/direct3d12/shared-heaps),
[CreateSharedHandle](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device-createsharedhandle),
[OpenSharedResource1](https://learn.microsoft.com/en-us/windows/win32/api/d3d11_1/nf-d3d11_1-id3d11device1-opensharedresource1),
[COMMON decay](https://learn.microsoft.com/en-us/windows/win32/direct3d12/using-resource-barriers-to-synchronize-resource-states-in-direct3d-12),
and [legacy shared handles](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiresource-getsharedhandle).
The two-device path and supported resource flags still require actual Windows
debug-layer/driver testing; implementing the interfaces is not evidence of that
qualification.

## Consumer lifetime and source identity

Evidence is pinned to Flutter **3.47.9**, framework revision
`c197ca5199e9ee5661e865596c9c07294dfab51c` (SDK engine revision
`89c39edfdfae2949f6806e4a315c0c08522238b7`):

- [public surface descriptor](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/common/public/flutter_texture_registrar.h)
  describes release after the handle has been opened.
- [D3D external texture implementation](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/windows/external_texture_d3d.cc)
  calls that callback after the EGL surface opening/binding attempt. It exposes no
  consumer GPU completion callback; the opening notification can also follow an
  unsuccessful import attempt.
- [texture registrar](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/windows/flutter_windows_texture_registrar.cc)
  asynchronously unregisters/removes the texture and notifies callback completion.
  This protects registration callback userdata, not texture sampling completion.
- [adapter APIs](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/windows/public/flutter_windows.h)
  provide the adapter and require releasing the returned reference.

In the default legacy mode, published targets never change. Resources and their native DLL lease remain
retained in a process-resident fixed pool; opening, unregistering, close, and
plugin recreation never refund a published or uncertain frame. Descriptor
release only increments notification counters. `consumerRetirement:false` and
`continuousPresentationSupported:false` report this limitation explicitly.
ARCH-018 records the remaining consumer-retirement gap.

## Opt-in immutable ownership transfer (P109)

Creation may explicitly include `publicationMode:"immutableOwnershipTransfer"`.
Omitting the field, or using `"legacyImmutable"`, keeps the original lifetime
policy. Each publication still allocates a fresh pair of D3D12 input and D3D11
shared destination resources, checks producer completion, and never modifies
those pixels after publication.

This mode uses a separate ownership guarantee from consumer GPU completion:

- The public descriptor contract requires the creator's resource reference
  through opening and explicitly permits releasing that reference afterward.
  [Microsoft's shared-handle contract](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiresource-getsharedhandle)
  describes the independent device resource that preserves shared video memory.
- The pinned engine's [DEPS](https://github.com/flutter/flutter/blob/89c39edfdfae2949f6806e4a315c0c08522238b7/DEPS)
  selects ANGLE `84027aca9b71c9ba335bd000dad1107b8810a511`.
  [SwapChain11's import](https://flutter.googlesource.com/third_party/angle/+/84027aca9b71c9ba335bd000dad1107b8810a511/src/libANGLE/renderer/d3d/d3d11/SwapChain11.cpp)
  opens the shared handle on the consumer D3D11 device and owns a separately
  queried texture interface. Its color-buffer reset releases that interface.
  [TextureHelper11 and GenericData](https://flutter.googlesource.com/third_party/angle/+/84027aca9b71c9ba335bd000dad1107b8810a511/src/libANGLE/renderer/d3d/d3d11/renderer11_utils.h)
  carry the owned COM reference through RAII. Creator-reference release does not
  rewrite the consumer-owned allocation or wait for its GPU sampling.
- The engine calls the descriptor's opening notification and then reads its
  visible dimensions. `SurfaceImportTickets` therefore keeps immutable descriptor
  storage and a strong frame pin until the next non-reentrant callback on the
  same raster OS thread, or confirmed unregister completion. Close alone cannot
  supply this CPU-quiescence evidence.
- CPU descriptor retirement does not release the last exported allocation. The
  pinned engine destroys its EGL surface on a null callback but keeps its cached
  HANDLE value. Each owner therefore pins its last exported frame and its existing
  credit through empty callbacks until a distinct valid replacement is offered
  or unregister is confirmed. This prevents a fresh allocation from recycling the
  cached HANDLE and bypassing the engine's import branch. These pins stay within
  the same four-frame/64-MiB accounting; they add no target pool or GPU fence.
- Opening may arrive on another thread. Never-reused opaque cookie values index
  a process-wide registry of at most eight descriptor records; a late duplicate
  performs a lookup and cannot dereference released userdata. Missing opening
  notifications remain charged and pinned. A raster-thread change/reentry fails
  closed. A contradictory unregister receipt during the callback remains pinned.

Legacy retained allocations and new producer/current/import-pending references
share the same process-wide **four-frame/64-MiB** admission bound. Existing legacy
or failed-producer quarantine is never forgiven by enabling the new mode.
The new mode's `producerOwnedFrames`/`producerOwnedBytes` account only plugin-owned
references. Successful transfer eventually removes those creator references;
**consumer-owned allocations and driver deferred memory are outside this bound**.
It is not a total GPU-memory promise or a consumer-retirement pool. Run legacy
and continuous qualification in separate runner processes so a legacy process's
permanent credits cannot contaminate the continuous test.

`continuousPresentationSupported:true` identifies this explicit immutable-transfer
path. `consumerRetirement:false`, `consumerGpuCompletionVerified:false`,
`descriptorReleaseIsGpuComplete:false`, and `targetReuse:false` remain unchanged
in meaning. `handedOffFrames` counts released creator-reference reservations;
it is not a count of completed GPU reads. Unexposed superseded frames are
reported separately. A temporary admission error returns `backpressure` with
`backpressureRetryable:true` while a non-permanent slot can be released; the host
may coalesce/retry within its existing deadline. An oversized target is the
permanent `targetTooLarge` error in this mode.

P97 retains legacy HOST/worker/session calls on its dedicated owner thread. This
plugin never moves those TLS handles to platform, producer, or raster callbacks.
The owner configures the P89 source and exports screen snapshots. The plugin uses
immutable IDs and `rhwp_d3d_frame_check` for the full document/revision/font/view/
device/DPR stamp, including same-source/profile checks. It rechecks before GPU
work, after producer completion, before publication, and in the Flutter callback.
Source changes can always occur after a check: P102 must compare its current
document stamp before showing a Texture and hide it on stale/error/backpressure.
A null callback result is not a general cached-image-clear or retirement signal.

The native DLL must be the same canonical path/module used by the P97 dispatcher;
a separately linked native copy has a different snapshot registry. Native and
plugin DLLs remain process-pinned while callbacks, detached producers, or opaque
quarantined GPU state can exist. No Flutter Skia object crosses the boundary.

## Budgets and failure atomicity

- Two producer threads and two active/unregistering registrations process-wide.
  Each producer holds at most one pending job and one in-flight job. Submitting
  supersedes the pending job; it does not cancel an executing GPU call.
- Four target reservations and 64 MiB charged target bytes process-wide, at most
  16 MiB per reservation. Both D3D12 source and D3D11 destination are charged.
  Row-pitch/64-KiB rounding and power-of-two allowance permit at most 8 MiB per
  surface; 1920×1080 fits, larger viewports may require CPU. Actual D3D12
  allocation info is checked against its allowance before allocation. These are
  target accounting bounds, not a total driver VRAM/RSS guarantee.
- Eight placements and 64 KiB metadata per request. Snapshot IDs/scalars are
  bounded; pointer integers are never JSON fields. Numeric overflow/nonfinite
  inputs and stale generations are rejected before snapshot ownership changes.
- Resource bundles enter their fixed pool slot **before** producer GPU work.
  Exceptions, close, and indeterminate completion therefore cannot drop possibly
  in-flight resources. Confirmed producer-complete, never-exposed targets and
  pending jobs can refund credit; published and uncertain targets cannot.
- Native strict replay preserves previous output by staging first. A stale error
  tagged `unpublishedTargetSafeToDiscard:true` occurs before target use or after
  checked producer completion. Other unproven errors quarantine their targets.
- A D3D12 completion failure retains native context, scratch/visible surfaces,
  backend COM references, renderer/font resources, and immutable snapshot quota.
  Native live+quarantined context credit has a separate process-wide limit of
  four. Owner-thread abandon/TLS teardown preserves that opaque state instead of
  releasing resources with unknown D3D12 GPU use. There is no forgiveness API.
- Close never joins the producer on the UI thread. `ownerFinished` remains false
  until it really exits. Explicit fence/event polling has a timeout; an internal
  Skia/driver call can outlive it. Its thread/module/resources stay pinned and
  cannot be reported as cancelled or reclaimed merely because the UI closed.

Native renderer/font/cache quotas additionally bound their tracked logical
allocations. Producer completion and `wholeFrameReadbacks:0` do not establish
Flutter import success, visible output, hardware execution, or consumer completion.

## Native C ABI

Enable `native-skia-d3d` on Windows. Default/CPU builds do not enable D3D12.
`skia-safe` 0.156.0 and `windows` 0.62.2 share the same COM types. Native exports:

- `rhwp_d3d_capabilities()` is separate from session capabilities.
- `rhwp_d3d_presenter_create(adapter, device, queue, generation, config, len)`
  clones typed COM interface references on a dedicated producer OS thread.
- `rhwp_d3d_presenter_render(handle, resource, request, len)` uses JSON
  `{stamp,target:{width,height,deviceGeneration},placements,clearColor}`.
- `rhwp_d3d_presenter_request(handle,{op:info|release|abandon},len)` stays on that
  owner thread. Info includes native context credit/quarantine counts.
- `rhwp_d3d_frame_check({stamp,snapshots:[1..8]},len)` is cross-thread metadata
  validation. It performs no document/Core/layout/GPU work.

Every returned string uses `rhwp_string_free`. Non-Windows/disabled symbols return
`featureDisabled` without reading their input spans. Successful frames explicitly
report producer completion, COMMON state and monotonic fence value, BGRA8/sRGB/
premultiplied/top-left metadata, `consumerGpuComplete:false`, and no readback.

## Flutter channel and P102 integration

Plugin class `RhwpD3dPlugin`, C registrar `RhwpD3dPluginRegisterWithRegistrar`,
channel `rhwp_native/texture_windows`, standard method codec. P101 changes neither
Dart nor pubspec. Add Windows `pluginClass: RhwpD3dPlugin` to the package's plugin
stanza and use a separate Flutter runner/example depending on it. The generated
registrant includes `<rhwp_native_reference/rhwp_d3d_plugin.h>` and links
`rhwp_native_reference_plugin`. Do not overwrite this plugin's Windows CMake
definition by generating a runner in the same directory. CMake builds no Rust
library and bundles none implicitly.

- `create({libraryPath:canonicalAbsolute,publicationMode?:string,closeAcknowledgementRequired?:bool})` registers asynchronously and returns
  `textureId`, `deviceGeneration`, `ready` and limits. Poll until ready; init can
  fail without accepting any snapshot IDs. A true optional close-ack request is
  negotiated only when both the top-level reply and limits echo
  `closeAcknowledgementRequired:true`. Older clients/plugins retain their prior
  close protocol. Negotiated calls include the returned `deviceGeneration` on
  every subsequent texture RPC, including submit/status/close/retireAck.
- `submit({textureId,stamp,width,height,clearColor,placements})` uses P99's six-key
  stamp and page placement schema. Scale is `zoom*DPR`; origins are physical
  NativeView page positions. View generation strictly advances. A successful
  response has `accepted:true`, `acceptedSerial`, `acceptedStamp`,
  `acceptedSnapshots`, and limits.
- `status({textureId})` reports `ready`, `closed`, `ownerFinished`, `inFlight`,
  `unregistered`, bounded `error`/`errorCode`, `deviceFailure`, device generation,
  the last acceptance receipt, and `hasFrame`/`renderedSerial`/`renderedStamp`.
  No current frame means false/0/null. Producer initialization or acceptance is
  not a published-frame receipt. Check rendered stamp and full snapshot source
  before uncovering Texture. Even that is not a consumer-import acknowledgment.
  `descriptorOpenedSerial` records opening for the current rendered frame; it
  does not acknowledge sampling or completion. `publicationMode` is returned both
  at the top level and in `limits`.
- `close({textureId})` stops admission, clears pending/current job references,
  wakes the producer and unregisters asynchronously. Negotiated closing records
  remain in the two-record admission bound until the client explicitly sends
  `retireAck({textureId,deviceGeneration})`. Owner completion and unregister must
  both be observed first. Successful ack replies echo both identity fields and
  `acknowledged:true`, `closed:true`, `ownerFinished:true`, `registered:false`,
  `unregistered:true`. Only then may the record become resource-free metadata in
  the recent-two terminal cache. Unobserved closing records cannot be evicted by
  creating more textures. Late/wrong generations cannot acknowledge a reused
  Flutter texture ID. Legacy non-negotiated records retain create-time automatic
  retirement. Neither
  close nor unregister refunds legacy immutable/quarantine credit. In transfer
  mode, confirmed unregister also retires CPU descriptor storage whose opening
  was observed; an unacknowledged import stays charged for a possible late reply.

Limits include outstanding/immutable/quarantined/pending-retire frames, reserved
target bytes, maximums, descriptor-open notification count, and explicit false
consumer-retirement flags and the selected publication mode. New transfer-mode
fields include producer-owned/import-pending counts and bytes, retained legacy
frames, transferred/unexposed reservation counters, and bounded descriptor ticket
counts. All are process-wide ownership accounting. Backpressure requires the CPU
reference path; it is not a GPU success. Keep strict `documentError`/limit/resource
diagnostics visible; do not label generic submission/allocation failure as actual
device loss. `deviceFailure:true` requires a positive device-removal report.

New distinct snapshot IDs transfer only after accepted success. Previously
accepted IDs reused in the same texture borrow existing leases. A producer handle
has one owner; export another handle for another texture. Admission rejections
leave new IDs with the caller. Preserve the acceptance ledger through late Dart
disposal. Transport/generic reply failure can be ambiguous: query the last
accepted stamp/IDs before releasing anything, and never infer ownership reversal
from disposal. Closing pending or finished producers releases unused IDs; the
immutable pixel pool holds GPU resources rather than historical document handles.

## Checks and remaining qualification

```bash
python3 -B bindings/Flutter/windows/tests/check_windows_bridge.py \
  --output /new/path/p101-checks --flutter-public /path/to/pinned/flutter/common/public
python3 -B bindings/Flutter/windows/tests/check_windows_bridge.py \
  --output /new/path/p101-sanitized --sanitize
```

These run the used portable generation/immutable-credit/native-metadata helpers
and immutable ownership-transfer descriptors,
ASan/UBSan when requested, a small standalone `rustc` build of the **production**
disabled FFI module, and real common Flutter/native header contract checks when
headers are supplied. They install no SDK, call no Cargo, and do not simulate a
Windows producer fence as successful GPU execution.

Portable tests include 128 consecutive fresh publications, dimensions read after
the opening callback, consumer-independent resource references, delayed/duplicate
opening, four-slot backpressure, shared legacy admission, producer-uncertain
quarantine, concurrent close, bounded descriptor records, and raster-thread
changes/reentry. They do not execute D3D or qualify a Windows consumer.

Remaining for this newer source: Windows feature/C++ compilation and DLL linkage, actual D3D12→D3D11 NT
sharing/legacy handle import on the chosen adapter, debug-layer state validation,
producer timeout/device-removal paths, Flutter sampling/colors/origin/DPI,
asynchronous engine shutdown, actual target accounting/driver memory, and real
hardware/software identification. Consumer retirement/target recycling remains
an API limitation until a verified completion contract exists. Continuous
immutable publication requires actual more-than-four-frame Flutter sampling with
matching artifact receipts and the ownership bounds above. Final OS/fixture qualification belongs
to the user's later validation.
