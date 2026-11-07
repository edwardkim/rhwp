// Real dispatcher/Native worker contracts, without injecting an OS IME or GPU.
// Opt in with flutter test --dart-define=RHWP_OWNER_LIBRARY=/absolute/owner.so
// --dart-define=RHWP_NATIVE_LIBRARY=/absolute/native.so followed by this file.
import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

const _ownerLibrary = String.fromEnvironment('RHWP_OWNER_LIBRARY');
const _nativeLibrary = String.fromEnvironment('RHWP_NATIVE_LIBRARY');
const _missingLibraries = _ownerLibrary == '' || _nativeLibrary == '';
const _skipReason =
    'Set RHWP_OWNER_LIBRARY and RHWP_NATIVE_LIBRARY to fresh dispatcher/P98 libraries.';

Future<(NativeDocument, NativeEditorController, Future<void> Function())>
_openEditor() async {
  final transport = await NativeOwnerTransport.connect(
    dispatcherLibrary: _ownerLibrary,
    nativeLibrary: _nativeLibrary,
  );
  addTearDown(transport.close);
  expect(
    transport.capabilities['atomicImeEdits'],
    isTrue,
    reason: 'The opt-in native library must include P98 atomic IME edits.',
  );
  final document = await NativeDocument.connect(transport);
  addTearDown(document.dispose);
  await document.newDocument();
  final editor = NativeEditorController(document);
  var editorDisposed = false;
  void disposeEditor() {
    if (!editorDisposed) {
      editorDisposed = true;
      editor.dispose();
    }
  }

  addTearDown(disposeEditor);
  await editor.initialize();
  expect(editor.ready, isTrue);
  return (
    document,
    editor,
    () async {
      disposeEditor();
      await document.dispose();
    },
  );
}

typedef _MetadataNative = Pointer<Utf8> Function(Pointer<Uint8>, UintPtr);
typedef _MetadataDart = Pointer<Utf8> Function(Pointer<Uint8>, int);
typedef _FreeNative = Void Function(Pointer<Utf8>);
typedef _FreeDart = void Function(Pointer<Utf8>);

// Stats is an explicit cross-thread P89 metadata entry. This helper never calls
// a legacy session/worker/buffer API on Dart's UI isolate or replays any pixels.
Map<String, Object?> _presentationStats(String libraryPath) {
  final library = DynamicLibrary.open(libraryPath);
  final request = library.lookupFunction<_MetadataNative, _MetadataDart>(
    'rhwp_presentation_request',
  );
  final free = library.lookupFunction<_FreeNative, _FreeDart>(
    'rhwp_string_free',
  );
  final bytes = utf8.encode('{"op":"stats"}');
  final input = calloc<Uint8>(bytes.length);
  Pointer<Utf8> output = nullptr;
  try {
    input.asTypedList(bytes.length).setAll(0, bytes);
    output = request(input, bytes.length);
    if (output == nullptr) throw StateError('Native stats returned no JSON.');
    final raw = output.cast<Uint8>();
    var length = 0;
    while (length <= 65536 && raw[length] != 0) {
      length++;
    }
    if (length > 65536) throw StateError('Native stats exceeded 64 KiB.');
    final value = jsonDecode(output.toDartString(length: length)) as Map;
    expect(value['ok'], isTrue);
    return Map<String, Object?>.from(value['stats']! as Map);
  } finally {
    if (output != nullptr) free(output);
    calloc.free(input);
  }
}

void _expectNoPresentationResources(Map<String, Object?> stats) {
  for (final name in [
    'sources',
    'snapshotHandles',
    'retainedSnapshots',
    'snapshotEstimatedBytes',
    'frameHandles',
    'retainedFrames',
    'frameBytes',
    'leases',
    'inFlightReplays',
  ]) {
    expect(stats[name], 0, reason: '$name must be released after owner close.');
  }
}

void main() {
  test(
    'real Dart owner IME composes Hangul once and preserves scalar selections',
    () async {
      final (document, editor, _) = await _openEditor();
      await editor.setEditingValue(
        const TextEditingValue(
          text: 'ㅎ',
          selection: TextSelection.collapsed(offset: 1),
          composing: TextRange(start: 0, end: 1),
        ),
      );
      final composition = document.info!.compositionId;
      expect(composition, isNotNull);
      await editor.setEditingValue(
        const TextEditingValue(
          text: '하',
          selection: TextSelection.collapsed(offset: 1),
          composing: TextRange(start: 0, end: 1),
        ),
      );
      expect(document.info!.compositionId, composition);
      await editor.setEditingValue(
        const TextEditingValue(
          text: '한',
          selection: TextSelection.collapsed(offset: 1),
        ),
      );
      expect(document.info!.compositionId, isNull);
      expect(editor.value.text, '한');
      await editor.command('undo');
      expect(
        editor.value.text,
        isEmpty,
        reason: 'No intermediate jamo undo entry.',
      );
      expect(document.info!.details['canUndo'], isFalse);
      await editor.command('redo');
      expect(editor.value.text, '한');

      await editor.setEditingValue(
        const TextEditingValue(
          text: '한😀',
          selection: TextSelection.collapsed(offset: 3),
        ),
      );
      final appendedRevision = document.info!.revision;
      await editor.setEditingValue(
        editor.value.copyWith(
          selection: const TextSelection(baseOffset: 3, extentOffset: 1),
        ),
      );
      expect(document.info!.revision, appendedRevision);
      expect(
        document.info!.selection!['anchor'],
        containsPair('scalarOffset', 2),
      );
      expect(
        document.info!.selection!['focus'],
        containsPair('scalarOffset', 1),
      );
      final beforeReply = await document.request('imeEditContext');
      expect(beforeReply['ok'], isTrue);
      expect(beforeReply['kind'], 'imeEditContext');
      final before = Map<String, Object?>.from(beforeReply['context']! as Map);
      final valueBefore = editor.value;
      final invalid = valueBefore.copyWith(
        selection: const TextSelection.collapsed(offset: 2),
      );
      await expectLater(
        Future<void>.sync(() => editor.setEditingValue(invalid)),
        throwsFormatException,
      );
      // RPC correlation IDs advance on each query; the full Native editing
      // context, including its revision/selection/composition, must not change.
      final afterReply = await document.request('imeEditContext');
      expect(afterReply['ok'], isTrue);
      expect(afterReply['kind'], 'imeEditContext');
      expect(afterReply['context'], before);
      expect(document.info!.revision, appendedRevision);
      expect(editor.value, valueBefore);

      await editor.command('deleteBackward');
      expect(editor.value.text, '한');
      await editor.command('undo');
      expect(editor.value.text, '한😀');
      expect(
        editor.value.selection,
        const TextSelection(baseOffset: 3, extentOffset: 1),
      );
      expect(editor.pending, 0);
    },
    skip: _missingLibraries ? _skipReason : false,
    timeout: const Timeout(Duration(seconds: 90)),
  );

  test(
    'real IME invalidates full presentation stamps and releases snapshot credits',
    () async {
      final (document, editor, close) = await _openEditor();
      final presentation = await NativePresentation.connect(
        nativeLibrary: _nativeLibrary,
      );
      if (!presentation.available) {
        markTestSkipped(
          'This metadata contract additionally requires P93 fullStampChecks.',
        );
        return;
      }
      _expectNoPresentationResources(
        _presentationStats(presentation.libraryPath),
      );
      final capture = await document.capturePresentation(
        page: 0,
        viewGeneration: document.beginPresentation(),
        deviceGeneration: 1,
        deviceScale: 1.5,
        releaseSnapshot: presentation.releaseSnapshot,
      );
      final snapshot = capture['snapshot']! as int;
      try {
        final stamp = NativePresentationStamp.fromJson(
          Map<String, Object?>.from(capture['stamp']! as Map),
        );
        expect(await presentation.snapshotCheck(snapshot, stamp), isTrue);
        final changedScale = NativePresentationStamp.fromJson({
          ...stamp.toJson(),
          'deviceScale': 2.0,
        });
        expect(
          await presentation.snapshotCheck(snapshot, changedScale),
          isFalse,
        );
        await editor.setEditingValue(
          const TextEditingValue(
            text: '가',
            selection: TextSelection.collapsed(offset: 1),
          ),
        );
        expect((await presentation.snapshotInfo(snapshot))['current'], isFalse);
        expect(await presentation.snapshotCheck(snapshot, stamp), isFalse);
      } finally {
        await presentation.releaseSnapshot(snapshot);
      }
      await close();
      expect(document.isDisposed, isTrue);
      _expectNoPresentationResources(
        _presentationStats(presentation.libraryPath),
      );
    },
    skip: _missingLibraries ? _skipReason : false,
    timeout: const Timeout(Duration(seconds: 90)),
  );
}
