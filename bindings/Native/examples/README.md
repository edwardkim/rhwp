# Native reference host

`native_host.py` is a headless Python standard-library example of a standalone
host using the native C ABI. It uses neither WASM nor a browser/DOM. Supply an
explicit shared library built with the `native-skia` feature; the example does
not compile or locate a library automatically. Python 3.9 or newer is sufficient.

Run the blank-document flow from any working directory:

```sh
python3 /path/to/rhwp/bindings/Native/examples/native_host.py \
  --library /path/to/librhwp_native_ffi.so \
  --output /path/to/new-document.hwpx
```

Use the corresponding `.dylib` or `.dll` path on another platform. The path above
is illustrative: use the actual library produced by your build. Follow the
repository's build guidance and your local target-isolation configuration;
different physical workspaces must not accidentally share colliding Cargo build
artifacts. No machine-specific target directory is part of this example.

The default flow opens a blank document, renders a page region into owned RGBA
bytes, queries hit-test/caret geometry, enters body-text editing, sends the IME
sequence `ㄱ → 가 → 각`, commits it once, and performs undo/redo. It then prepares
HWPX bytes, copies and releases the native buffer, writes a new file, acknowledges
the saved revision, closes the session, reopens the file, and checks extracted
text through `rhwp_read_text`. The JSON summary includes render metadata, save
diagnostics and the round-trip result. A pixel hash is an artifact identity, not
evidence of visual equivalence with Hancom.

To view/query a supplied document and save a copy:

```sh
python3 /path/to/rhwp/bindings/Native/examples/native_host.py \
  --library /path/to/librhwp_native_ffi.so \
  --input /path/to/source.hwp \
  --output /path/to/new-copy.hwpx
```

`--input` never runs the editing demo on that document. Unsupported interaction
targets, such as inline objects, are reported explicitly while retaining the
viewer session. Renderer/format errors still fail the operation. Input files are
read only, and output uses exclusive creation: an existing destination, symlink
or source file is never overwritten. Password-protected input is outside this
small example's CLI surface.

The host owns filesystem I/O. A native save token is acknowledged only after the
new file is fully written, flushed and closed. A write failure cancels the token
and removes only the new partial file created by that call. An acknowledgement
failure preserves successfully written output. This is not a replace-existing
Save implementation or a crash-recovery journal; directory-entry crash durability
and platform-native save dialogs need their own host implementation.

Every native result string is freed with `rhwp_string_free`. Render and save
buffers are copied before `rhwp_session_buffer_release`. Sessions close in
`finally` blocks, including failure paths. All calls run on one thread because
session handles are thread-affine. `originX`/`originY` describe scaled page pixel
coordinates, while hit-test/caret use unscaled page-layout coordinates. This
example uses scale 1 for the region and passes layout coordinates directly.

Editing here is limited to supported body text. It does not imply support for
cell editing, inline-control affinity, paragraph/tab insertion, OS key bindings,
clipboard protocols, accessibility, windows, GPU presentation or an OS IME
candidate window. The IME events are supplied programmatically. HWPX may preserve
source/default caret metadata instead of saving the current editor focus; inspect
`caretSaved` and `caretPreservedFromSource` in the summary. Export content-loss
diagnostics are returned in `save.report` and should inform a real application's
save UI.

Focused validation:

```sh
# Native library not needed: exclusive-write, cleanup and checkpoint ordering.
python3 /path/to/rhwp/bindings/Native/tests/native_host_smoke.py --mock

# Actual native ABI: blank edit/save/reopen and read-only input/save-copy flow.
python3 /path/to/rhwp/bindings/Native/tests/native_host_smoke.py \
  --library /path/to/librhwp_native_ffi.so
```

Mock success does not validate native compilation, rendering, platform SDKs or OS
IME integration. Actual-library and target-platform execution must be reported
separately; the example is not a claim that macOS/Windows SDK integration or final
visual validation has already been completed.
