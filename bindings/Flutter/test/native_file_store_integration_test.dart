// Actual Dart owner dispatcher + Native worker serialization/recovery; no OS UI.
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

const _owner = String.fromEnvironment('RHWP_OWNER_LIBRARY');
const _native = String.fromEnvironment('RHWP_NATIVE_LIBRARY');

void main() {
  test(
    'actual worker save/reopen and separate recovery preserve checkpoints',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'rhwp-native-save-',
      );
      addTearDown(() => directory.delete(recursive: true));
      final transport = await NativeOwnerTransport.connect(
        dispatcherLibrary: _owner,
        nativeLibrary: _native,
      );
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      var editor = NativeEditorController(document);
      addTearDown(() => editor.dispose());
      await editor.initialize();
      await document.request('pasteText', {'text': '저장 😀'});
      final files = NativeFileStore(document);
      final path = '${directory.path}${Platform.pathSeparator}saved.hwpx';
      final saved = await files.save(
        destination: path,
        approveLoss: (_) async => true,
      );
      expect(saved.cancelled, isFalse);
      expect(saved.info.isDirty, isFalse);
      expect(saved.savedRevision, document.info!.revision);
      expect(await File(path).length(), greaterThan(0));
      editor.dispose();
      await files.openPath(path);
      editor = NativeEditorController(document);
      await editor.initialize();
      final context = await document.request('imeEditContext');
      expect(
        ((context['context'] as Map)['context'] as Map)['text'],
        contains('저장 😀'),
      );
      await document.request('pasteText', {'text': '복구'});
      final original = await File(path).readAsBytes();
      final recovery = await files.writeRecovery(
        '${directory.path}${Platform.pathSeparator}recovery',
      );
      expect(recovery, isNotNull);
      expect(document.info!.isDirty, isTrue);
      expect(await File(path).readAsBytes(), original);
      editor.dispose();
      await files.openRecovery(recovery!);
      expect(files.sourcePath, isNull);
      expect(document.info!.isDirty, isTrue);
      expect(document.info!.revision, 0);
      editor = NativeEditorController(document);
      await editor.initialize();
      final recovered = await document.request('imeEditContext');
      expect(
        ((recovered['context'] as Map)['context'] as Map)['text'],
        contains('복구'),
      );
    },
    skip: _owner.isEmpty || _native.isEmpty
        ? 'Set RHWP_OWNER_LIBRARY and RHWP_NATIVE_LIBRARY to run actual worker persistence.'
        : false,
  );
}
