# Desktop memory-pressure acceptance

The existing visible-desktop probe can inject bounded memory-pressure cycles
between its 32..64 publications. This exercises the real Flutter system channel,
the mounted NativeGpuLifecycle observer, the existing controller, and the native
trim method. The default remains zero cycles and preserves the existing probe.

The [combined Linux SDK runner](LINUX_QUALIFICATION.md#sdkgl-feature표시의-순차-실행)
also accepts --memory-pressure-cycles 2 during full consumer capture. It forwards
the option to the single 32-frame and two-tile three-stage collectors and checks
their individual memory receipts. Its combined range is 0..2; compile-only and
build-only reject positive cycles because those modes do not sample consumers.

Build the current committed probe and its matching Native GPU library with the
existing platform procedure, then add --memory-pressure-cycles 2 to its capture
command. One complete Linux capture example, from bindings/Flutter:

    python3 tool/qualify_linux_texture.py \
      --runner /private/rhwp-linux-build/runner/build/linux/x64/debug/bundle/rhwp_texture_qualification \
      --native /artifacts/librhwp_native_ffi.so \
      --dispatcher /private/rhwp-linux-build/librhwp_flutter_owner.so \
      --fixture /private/rhwp-linux-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx \
      --build-receipt /private/rhwp-linux-build/build-receipt.json \
      --frames 32 --memory-pressure-cycles 2 --output /private/rhwp-linux-memory

The same flag is supported by
[the Windows continuous collector](WINDOWS_CONTINUOUS_QUALIFICATION.md) and the
[macOS AppKit collector](macos/NATIVE_VIEW_PRESENTATION.md). For macOS:

    python3 macos/tests/qualify_native_view.py capture \
      --build-receipt /private/rhwp-macos-build/build-receipt.json \
      --frames 32 --memory-pressure-cycles 2 --output /private/rhwp-macos-memory

Use --tile-group on that macOS capture command when the matching probe supports
the existing two-AppKit-view case. These continuous cycles cover the existing
Linux single-texture, Windows immutable-transfer, and macOS AppKit single/pair
probes.

One to four cycles are accepted. They occur after publication 3 and between
later continuous publications, with a later externally sampled publication
after every cycle. They add no publication or compositor acknowledgement index.
For 32 frames and two cycles, pressure follows publications 3 and 17, and visible
publications 4 and 18 prove that the same native owner continues rendering.
The collector and shared memory validator must match the actual runner build's
recorded source hashes.

The primary Linux two-tile probe keeps its existing three-stage initial, edit,
and resize protocol. Its separate collector accepts zero to two pressure cycles:

    python3 tool/qualify_linux_tile_group.py \
      --runner /private/rhwp-linux-tile-build/runner/build/linux/x64/debug/bundle/rhwp_texture_qualification \
      --native /artifacts/librhwp_native_ffi.so \
      --dispatcher /private/rhwp-linux-tile-build/librhwp_flutter_owner.so \
      --fixture /private/rhwp-linux-tile-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx \
      --build-receipt /private/rhwp-linux-tile-build/build-receipt.json \
      --memory-pressure-cycles 2 --output /private/rhwp-linux-tile-memory

One cycle follows publication 1; two follow publications 1 and 2. Publications 2
and 3 then retain the existing full viewport/independent CPU strip/seam checks.
There is no extra capture, publication index, or synthetic 32-frame claim. Both
native tile identities, full stamps, per-tile idle progress, and ordered trim
members are retained. The aggregate byte totals are the sums of both receipts;
the zero-byte target still applies separately to each native owner. The shared
Windows tile/recovery probe retains its default behavior and does not accept
this Linux memory profile.

Each cycle records the current full presentation stamp, texture/device identity,
ordered tile identities when present, the real typed managedGpuTargets trim
receipt, and ProcessInfo.currentRss. A probe-only subclass observes the result of
super.trimCaches; it does not substitute a receipt or call the lifecycle
observer directly. The injected flutter/system memoryPressure message follows
the framework's normal binding dispatch.

The idle window has three explicit status observations separated by 200 ms,
plus one observation before pressure. It retains their process RSS, native
publication progress, and controller trace counts. The held source, native
publication counters, capture count, and submission count must remain stable.
Controller status-poll deltas are reported separately from the three explicit
probe polls; the collector does not infer zero controller polling. All cycles
finish before the existing bounded native close/unregister and zero Native
source/snapshot/frame/lease checks.

The receipt distinguishes an observedReclamation with positive managed bytes
from an observedNoop with zero bytes. A final busy receipt remains busy, even
when some bytes were reclaimed; pinned, quarantined, in-flight, and public
immutable resources retain the existing protection. Unsupported, pending,
timed-out, foreign-owner, inconsistent group, or malformed receipts fail the
requested memory acceptance. A noop can pass the dispatch/idle/continuation
checks; it does not prove that bytes were reclaimed.

Producer diagnostics remain publishedFramePostRenderSnapshot measurements,
including when sampled after a trim. They are not live cache totals. RSS is the
whole probe process's current resident memory and is not expected to become zero
at close. The receipt makes no RSS leak threshold or global memory-bound claim.
Physical VRAM remains unknown, consumer/driver allocations are excluded from
managed-target accounting, and consumerGpuCompletionVerified remains false.

    python3 -B test/gpu_memory_acceptance_contract.py
    flutter test test/desktop_memory_probe_test.dart

These portable checks validate routing and rejection behavior with synthetic
receipts. They are not runtime GPU or RSS measurements. A target OS receipt is
still required for actual acceptance. Linux Xvfb/Mesa can validate the software
GL path and must retain hardwareAccelerationVerified:false. Physical GPU runs
on Linux, Windows, and macOS, longer resource-retention sessions, device loss,
real OS IME/accessibility, and final Hancom comparisons remain separate work.
