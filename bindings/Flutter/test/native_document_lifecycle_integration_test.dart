// Actual dispatcher/native save, replacement, recovery and close boundaries.
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

const _owner = String.fromEnvironment('RHWP_OWNER_LIBRARY');
const _native = String.fromEnvironment('RHWP_NATIVE_LIBRARY');

void main() {
  test(
    'real Native lifecycle preserves saved and recovered Korean edits',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'rhwp-lifecycle-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final transport = await NativeOwnerTransport.connect(
        dispatcherLibrary: _owner,
        nativeLibrary: _native,
      );
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      NativeEditorController? editor = NativeEditorController(document);
      addTearDown(() => editor?.dispose());
      await editor.initialize();
      await editor.setEditingValue(
        const TextEditingValue(
          text: '저장할 한글',
          selection: TextSelection.collapsed(offset: 6),
        ),
      );
      var decision = NativeUnsavedDecision.cancel;
      final output = '${directory.path}/saved.hwpx';
      var retired = false;
      final files = NativeFileStore(document);
      final lifecycle = NativeDocumentLifecycle(
        document: document,
        files: files,
        settleInput: () async {
          if (editor?.ready == true) await editor!.focusLost();
        },
        confirmUnsaved: (_) async => decision,
        chooseSavePath: () async => output,
        approveLoss: (_) async => true,
        retireViewport: () async {
          expect(document.isDisposed, isFalse);
          retired = true;
          return true;
        },
      );
      addTearDown(lifecycle.dispose);
      expect(await lifecycle.close(), isFalse);
      expect(document.info!.isDirty, isTrue);
      expect(retired, isFalse);
      final firstId = document.info!.documentId;
      decision = NativeUnsavedDecision.save;
      expect(await lifecycle.newDocument(), isTrue);
      editor.dispose();
      editor = null;
      expect(document.info!.documentId, isNot(firstId));
      expect(await File(output).exists(), isTrue);
      expect(files.sourcePath, isNull);

      // A new unsaved Native document is dirty even before its first edit.
      expect(document.info!.isDirty, isTrue);
      decision = NativeUnsavedDecision.discard;
      expect(await lifecycle.openPath(output), isTrue);
      editor = NativeEditorController(document);
      await editor.initialize();
      expect(editor.value.text, '저장할 한글');
      await editor.setEditingValue(
        const TextEditingValue(
          text: '복구할 한글',
          selection: TextSelection.collapsed(offset: 6),
        ),
      );
      final recovery = await lifecycle.writeRecovery(directory.path);
      expect(recovery, isNotNull);
      expect(document.info!.isDirty, isTrue);
      decision = NativeUnsavedDecision.discard;
      expect(await lifecycle.openPath(recovery!, recovery: true), isTrue);
      editor.dispose();
      editor = NativeEditorController(document);
      await editor.initialize();
      expect(editor.value.text, '복구할 한글');
      expect(document.info!.isDirty, isTrue);
      expect(files.sourcePath, isNull);
      expect(await lifecycle.close(), isTrue);
      expect(retired, isTrue);
      expect(document.isDisposed, isTrue);
    },
    skip: _owner.isEmpty || _native.isEmpty
        ? 'Provide dispatcher and Native shared library dart-defines.'
        : false,
  );
}
