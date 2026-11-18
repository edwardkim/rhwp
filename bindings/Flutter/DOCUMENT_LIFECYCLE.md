# Reference document lifecycle

`NativeDocumentLifecycle` connects the existing Native worker and
`NativeFileStore` to reference-host decisions. It owns no parser, platform
window, editor, or GPU device. The example supplies explicit save/discard/cancel,
destination, overwrite, and content-loss decisions.

## Replacement and close

Before reading dirty state, `settleInput` must drain the view's accepted editor
input. `NativeEditorView.onControllerChanged` exposes the view-owned initialized
editor for this barrier: call `focusLost()` and await it. The host observes this
controller and must not dispose it. A null notification detaches it.

The lifecycle then waits for a pending file operation and refreshes Native info.
A discard decision authorizes only the observed document/revision/font
checkpoint. An intervening edit invalidates the decision. Open/new operations
carry that checkpoint through file I/O and the Native FIFO, so a slow file read
cannot replace more recent edits. A save cancelled by destination selection,
overwrite approval, or loss approval preserves the document. A successful save
that leaves newer changes dirty also prevents replacement or close.

The host disables further interactive document actions while `isBusy` is true.
Concurrent destructive lifecycle calls are rejected. Changes submitted through
other code still require the checkpoint checks; disabling buttons is not the
document authority.

On close, keep `NativeTextureView` mounted under its normal opaque cover while
`retireViewport` awaits `NativeTextureController.close()` and confirms
`phase == NativeTexturePhase.closed` and `retirementPending == false`. It returns
true only for confirmed producer retirement/unregistration. A bounded Future
returning with `retirePending` must return false and preserve the mounted view
and worker. This receipt does not prove consumer GPU completion. Only then does the
lifecycle wait for file work, recheck the document and close its worker. A cancelled
decision or failed/unconfirmed retirement keeps that worker alive, and a repeated
in-progress close shares one Future. An error after Native owner closure starts
cannot restore editing: it is reported, the example detaches the closing document,
and the transport's existing close/cleanup observer remains responsible for it.
`dispose()` only releases lifecycle listeners: await `close()` before removing
the document view when normal retirement is possible. Abrupt platform process or
widget teardown still uses the existing bounded retire/quarantine behavior.
Texture registration/producer retirement is distinct from consumer GPU
completion, as described in [GPU presentation](GPU_PRESENTATION.md).

If retirement succeeds but a later checkpoint check rejects close, the example
removes the retired texture after its final frame and resumes the CPU view. It
keeps an unconfirmed registered texture mounted underneath its cover instead.

## Persistence and reference UI

The lifecycle delegates serialization, report approval, file publication,
acknowledgement, and recovery bytes to `NativeFileStore`. `writeRecovery` does
not acknowledge a user save. `openPath(path, recovery: true)` explicitly chooses
a recovery artifact; its provenance never becomes an overwrite destination.

The example keeps path entry and small reference action dialogs. It demonstrates
new/open/save/save-as/recovery/close with visible dirty state. It does not install
an OS window-close hook or promise crash recovery for a forcibly terminated
process. A downstream host must connect its window-close request to this same
awaited lifecycle.

## Verification

Run `flutter analyze` and `flutter test --concurrency=1`. The lifecycle tests
exercise delayed editor input, file reads and retirement, edits during discard
and save, cancellation, failed publication/retirement, and repeated close.

The real Native integration test saves Korean edits, replaces and reopens the
document, writes/opens recovery without clearing dirty, and verifies retirement
before owner close. Supply both libraries so this test actually executes:

```bash
flutter test --concurrency=1 \
  --dart-define=RHWP_OWNER_LIBRARY=/absolute/path/owner.so \
  --dart-define=RHWP_NATIVE_LIBRARY=/absolute/path/native.so \
  test/native_document_lifecycle_integration_test.dart
```

This is an in-process Native contract check. Real OS IME/accessibility, hardware
presentation and final Hancom/corpus qualification remain separate.
