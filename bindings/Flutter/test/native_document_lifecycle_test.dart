// Reference lifecycle decisions with asynchronous editor, file and texture gates.
import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/fake_transport.dart';
import 'support/fake_gpu_presentation.dart';
import 'native_document_view_test.dart' show pumpUntil;
import 'native_editor_view_test.dart' show ViewTransport;

class _Transport extends FakeTransport {
  bool dirty = false;
  @override
  Map<String, Object?> get info => {...super.info, 'isDirty': dirty};
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) {
    if ((request['command'] as Map?)?['op'] == 'new') dirty = false;
    return super.hostRequest(request, data: data);
  }
}

class _Files extends NativeFileStore {
  _Files(super.document, this.transport);
  final _Transport transport;
  bool cancelled = false;
  bool failSave = false;
  bool editDuringSave = false;
  int saves = 0;
  bool reading = false;
  int idleWaits = 0;
  Completer<void>? readGate;
  Completer<void>? idleGate;
  @override
  Future<void> get whenIdle {
    idleWaits++;
    return idleGate?.future ?? Future<void>.value();
  }

  @override
  Future<NativeSaveResult> save({
    String? destination,
    NativeLossApproval? approveLoss,
    NativeOverwriteApproval? approveOverwrite,
  }) async {
    saves++;
    if (failSave) throw StateError('publication failed');
    final saved = document.info!.revision;
    if (!cancelled) transport.dirty = false;
    if (editDuringSave) {
      transport.revision++;
      transport.dirty = true;
    }
    return NativeSaveResult(
      info: await document.refreshInfo(),
      cancelled: cancelled,
      path: destination,
      savedRevision: cancelled ? null : saved,
    );
  }

  @override
  Future<NativeDocumentInfo> openPath(
    String path, {
    NativeDocumentInfo? replacing,
  }) async {
    reading = true;
    await readGate?.future;
    return document.openBytes(Uint8List.fromList([1]), replacing: replacing);
  }
}

void main() {
  late _Transport transport;
  late NativeDocument document;
  late _Files files;
  late NativeDocumentLifecycle lifecycle;
  var decision = NativeUnsavedDecision.cancel;
  String? destination;
  Future<void> Function()? input;
  Future<bool> Function()? retire;
  Future<NativeUnsavedDecision> Function()? confirm;
  Future<String?> Function()? selectPath;

  setUp(() async {
    transport = _Transport();
    document = await NativeDocument.connect(transport);
    await document.newDocument();
    files = _Files(document, transport);
    decision = NativeUnsavedDecision.cancel;
    destination = null;
    input = null;
    retire = null;
    confirm = null;
    selectPath = null;
    lifecycle = NativeDocumentLifecycle(
      document: document,
      files: files,
      settleInput: () async => input?.call(),
      confirmUnsaved: (_) async => confirm?.call() ?? decision,
      chooseSavePath: () async => selectPath?.call() ?? destination,
      retireViewport: () async => retire?.call() ?? true,
    );
  });
  tearDown(() async {
    lifecycle.dispose();
    await document.dispose();
  });

  test(
    'cancel preserves the dirty document and allows a later decision',
    () async {
      transport.dirty = true;
      final id = document.info!.documentId;
      expect(await lifecycle.newDocument(), isFalse);
      expect(document.info!.documentId, id);
      expect(transport.closes, 0);
      decision = NativeUnsavedDecision.discard;
      expect(await lifecycle.newDocument(), isTrue);
      expect(document.info!.documentId, isNot(id));
    },
  );

  test('dirty confirmation waits for accepted input to drain', () async {
    final gate = Completer<void>();
    input = () async {
      await gate.future;
      transport.dirty = true;
      transport.revision++;
    };
    final id = document.info!.documentId;
    final next = lifecycle.newDocument();
    expect(lifecycle.isBusy, isTrue);
    expect(transport.commands.where((op) => op == 'new').length, 1);
    gate.complete();
    expect(await next, isFalse);
    expect(document.info!.isDirty, isTrue);
    expect(document.info!.documentId, id);
  });

  test('concurrent destructive intents are rejected', () async {
    final gate = Completer<void>();
    input = () => gate.future;
    final next = lifecycle.newDocument();
    expect(
      () => lifecycle.openPath('other.hwpx'),
      throwsA(isA<NativeCallException>()),
    );
    gate.complete();
    await next;
  });

  test(
    'an edit during discard confirmation invalidates that decision',
    () async {
      transport.dirty = true;
      final id = document.info!.documentId;
      confirm = () async {
        transport.revision++;
        return NativeUnsavedDecision.discard;
      };
      await expectLater(
        lifecycle.newDocument(),
        throwsA(isA<NativeCallException>()),
      );
      expect(document.info!.documentId, id);
      expect(document.info!.isDirty, isTrue);
      expect(lifecycle.error, isA<NativeCallException>());
    },
  );

  test('an edit during file reading prevents replacement', () async {
    final gate = files.readGate = Completer<void>();
    final id = document.info!.documentId;
    final opening = lifecycle.openPath('other.hwpx');
    await waitFor(() => files.reading);
    transport.revision++;
    transport.dirty = true;
    await document.refreshInfo();
    gate.complete();
    await expectLater(opening, throwsA(isA<NativeCallException>()));
    expect(document.info!.documentId, id);
    expect(transport.opened, isEmpty);
  });

  test('cancelled save-path selection keeps close cancellable', () async {
    transport.dirty = true;
    decision = NativeUnsavedDecision.save;
    expect(await lifecycle.close(), isFalse);
    expect(files.saves, 0);
    expect(transport.closes, 0);
    decision = NativeUnsavedDecision.discard;
    expect(await lifecycle.close(), isTrue);
    expect(transport.closes, 1);
  });

  test('content-loss cancellation preserves owner and dirty state', () async {
    transport.dirty = true;
    destination = 'saved.hwpx';
    decision = NativeUnsavedDecision.save;
    files.cancelled = true;
    expect(await lifecycle.close(), isFalse);
    expect(transport.closes, 0);
    expect(document.info!.isDirty, isTrue);
  });

  test('save failure prevents document replacement', () async {
    transport.dirty = true;
    destination = 'saved.hwpx';
    decision = NativeUnsavedDecision.save;
    files.failSave = true;
    final id = document.info!.documentId;
    await expectLater(lifecycle.newDocument(), throwsStateError);
    expect(document.info!.documentId, id);
    expect(document.info!.isDirty, isTrue);
    expect(transport.closes, 0);
  });

  test(
    'close keeps its source across a delayed destination decision',
    () async {
      transport.dirty = true;
      decision = NativeUnsavedDecision.save;
      selectPath = () async {
        await document.newDocument();
        return 'original-document.hwpx';
      };
      await expectLater(lifecycle.close(), throwsA(isA<NativeCallException>()));
      expect(
        files.saves,
        0,
        reason: 'Replacement B must never publish to A destination.',
      );
      expect(document.isDisposed, isFalse);
    },
  );

  test(
    'standalone save keeps its source across a destination decision',
    () async {
      selectPath = () async {
        await document.newDocument();
        return 'original-document.hwpx';
      };
      await expectLater(lifecycle.save(), throwsA(isA<NativeCallException>()));
      expect(files.saves, 0);
    },
  );

  test(
    'unconfirmed texture retirement keeps the view and owner alive',
    () async {
      retire = () async => false;
      await expectLater(lifecycle.close(), throwsA(isA<NativeCallException>()));
      expect(transport.closes, 0);
      expect(document.isDisposed, isFalse);
    },
  );

  test('post-checkpoint edits during save prevent close', () async {
    transport.dirty = true;
    destination = 'saved.hwpx';
    decision = NativeUnsavedDecision.save;
    files.editDuringSave = true;
    await expectLater(lifecycle.close(), throwsA(isA<NativeCallException>()));
    expect(document.isDisposed, isFalse);
    expect(document.info!.isDirty, isTrue);
    expect(transport.closes, 0);
  });

  test(
    'successful save retires the mounted viewport before owner close',
    () async {
      transport.dirty = true;
      destination = 'saved.hwpx';
      decision = NativeUnsavedDecision.save;
      final gate = Completer<void>();
      var retiring = false;
      retire = () async {
        retiring = true;
        expect(document.isDisposed, isFalse);
        await gate.future;
        return true;
      };
      final first = lifecycle.close();
      expect(identical(first, lifecycle.close()), isTrue);
      await waitFor(() => retiring);
      expect(transport.closes, 0);
      gate.complete();
      expect(await first, isTrue);
      expect(lifecycle.isClosed, isTrue);
      expect(transport.closes, 1);
      expect(await lifecycle.close(), isTrue);
      expect(transport.closes, 1);
    },
  );

  test('edits arriving during viewport retirement keep owner alive', () async {
    retire = () async {
      transport.revision++;
      transport.dirty = true;
      return true;
    };
    await expectLater(lifecycle.close(), throwsA(isA<NativeCallException>()));
    expect(transport.closes, 0);
    expect(document.isDisposed, isFalse);
  });

  test(
    'viewport retirement failure can be retried without owner loss',
    () async {
      retire = () async => throw StateError('platform close failed');
      await expectLater(lifecycle.close(), throwsStateError);
      expect(transport.closes, 0);
      retire = null;
      expect(await lifecycle.close(), isTrue);
    },
  );

  test('an owner-close error cannot resume a closing editor', () async {
    final otherTransport = _Transport();
    final otherDocument = await NativeDocument.connect(otherTransport);
    await otherDocument.newDocument();
    final otherLifecycle = NativeDocumentLifecycle(
      document: otherDocument,
      files: NativeFileStore(otherDocument),
      settleInput: () async {},
      confirmUnsaved: (_) async => NativeUnsavedDecision.discard,
      chooseSavePath: () async => null,
    );
    addTearDown(otherLifecycle.dispose);
    final gate = otherTransport.closeGate = Completer<void>();
    final closing = otherLifecycle.close();
    final observed = expectLater(closing, throwsStateError);
    await waitFor(() => otherTransport.closes == 1);
    gate.completeError(StateError('owner close failed'));
    await observed;
    expect(otherDocument.isDisposed, isTrue);
    expect(otherLifecycle.isClosed, isFalse);
    expect(
      () => otherLifecycle.save(),
      throwsA(isA<NativeDisposedException>()),
    );
  });

  test('a pending file operation is settled before replacement', () async {
    final gate = files.idleGate = Completer<void>();
    final id = document.info!.documentId;
    final next = lifecycle.newDocument();
    await waitFor(() => files.idleWaits > 0);
    expect(document.info!.documentId, id);
    gate.complete();
    expect(await next, isTrue);
  });

  for (final platform in const [
    NativeTexturePlatform.linux,
    NativeTexturePlatform.macos,
    NativeTexturePlatform.windows,
  ]) {
    testWidgets(
      'lifecycle close preserves completed automatic ${platform.name} retirement',
      (tester) async {
        final plugin = FakeTexturePlugin(platform)..install();
        final owner = await NativeDocument.connect(GpuTransport());
        final opening = owner.newDocument();
        await pumpUntil(tester, () => owner.info != null);
        await opening;
        final texture = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
        );
        final ownerLifecycle = NativeDocumentLifecycle(
          document: owner,
          files: NativeFileStore(owner),
          settleInput: () async {},
          confirmUnsaved: (_) async => NativeUnsavedDecision.discard,
          chooseSavePath: () async => null,
          retireViewport: () async {
            await texture.close();
            return texture.phase == NativeTexturePhase.closed &&
                !texture.retirementPending;
          },
        );
        try {
          await tester.pumpWidget(
            MaterialApp(
              home: SizedBox(
                width: 300,
                height: 240,
                child: NativeTextureView(
                  controller: texture,
                  viewport: NativePageViewport.fit(
                    const NativePageSize(600, 800),
                    const Size(300, 240),
                  ),
                  cpuFallback: (_) => const Text('CPU reference'),
                ),
              ),
            ),
          );
          await pumpUntil(tester, () => texture.canDisplay);
          plugin.statusError = 'backpressure';
          await pumpUntil(
            tester,
            () =>
                texture.needsCpuFallback &&
                plugin.closed &&
                !texture.shouldPaintTexture,
          );
          expect(texture.retirementPending, isFalse);
          expect(plugin.calls.where((call) => call == 'close'), hasLength(1));
          expect(owner.isDisposed, isFalse);

          final closing = expectLater(
            ownerLifecycle.close(),
            completion(isTrue),
          );
          await pumpUntil(
            tester,
            () => owner.isDisposed || ownerLifecycle.error != null,
          );
          await closing;
          expect(ownerLifecycle.isClosed, isTrue);
          expect(texture.phase, NativeTexturePhase.closed);
          expect(texture.retirementPending, isFalse);
          expect(texture.retirementDiagnostic, isNull);
          expect(plugin.calls.where((call) => call == 'close'), hasLength(1));
        } finally {
          await tester.pumpWidget(const SizedBox());
          texture.dispose();
          ownerLifecycle.dispose();
          await owner.dispose();
          plugin.uninstall();
          await tester.pump();
        }
      },
    );
  }

  testWidgets('view reports its own initialized editor and detach', (
    tester,
  ) async {
    final transport = ViewTransport();
    final document = await NativeDocument.connect(transport);
    final opened = document.newDocument();
    await pumpUntil(tester, () => document.info != null);
    await opened;
    NativeEditorController? observed;
    var detached = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: NativeEditorView(
            document: document,
            onControllerChanged: (editor) {
              observed = editor;
              if (editor == null) detached++;
            },
          ),
        ),
      ),
    );
    await pumpUntil(tester, () => observed?.ready == true);
    final editor = observed!;
    await tester.pumpWidget(const SizedBox());
    expect(observed, isNull);
    expect(detached, greaterThan(0));
    await document.dispose();
    // The view owns editor disposal even though the host observed it.
    await tester.pump();
    expect(editor.ready, isFalse);
  });
}
