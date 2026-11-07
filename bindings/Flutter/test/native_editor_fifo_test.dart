import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';
import 'package:rhwp_native_reference/src/native_editor_controller.dart';

import 'support/fake_transport.dart';

class EditingTransport extends FakeTransport {
  final submitted = <Map<String, Object?>>[];
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    data,
  }) async {
    final command = request['command'];
    if (command is Map) {
      submitted.add(Map<String, Object?>.from(command));
      if (command['op'] == 'replace') {
        commands.add('replace');
        return {'ok': true, 'requestId': ++serial};
      }
    }
    return super.hostRequest(request, data: data);
  }

  void edited(int request) {
    revision++;
    completions.add({
      'ok': true,
      'requestId': request,
      'kind': 'edited',
      'outcome': {'revision': revision},
    });
  }

  void information(int request) => completions.add({
    'ok': true,
    'requestId': request,
    'kind': 'info',
    'info': {...info, 'isDirty': true},
  });
}

void main() {
  test(
    'next edit waits for authoritative info and binds its completed revision',
    () async {
      final transport = EditingTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      transport.autoComplete = false;
      final first = document.request('replace', {'text': 'one'});
      final second = document.request('replace', {'text': 'two'});
      await waitFor(
        () => transport.commands.where((op) => op == 'replace').length == 1,
      );
      transport.edited(transport.serial);
      await waitFor(() => transport.commands.last == 'info');
      expect(transport.commands.where((op) => op == 'replace').length, 1);
      transport.information(transport.serial);
      expect((await first)['kind'], 'edited');
      await waitFor(
        () => transport.commands.where((op) => op == 'replace').length == 2,
      );
      final target = transport.submitted.last['target']! as Map;
      expect(target['revision'], 1);
      expect(document.info!.isDirty, isTrue);
      transport.edited(transport.serial);
      await waitFor(() => transport.commands.last == 'info');
      transport.information(transport.serial);
      await second;
      expect(document.info!.revision, 2);
    },
  );
  test(
    'accepted editor request never migrates to a replacement document',
    () async {
      final transport = EditingTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final replacement = document.newDocument();
      final edit = document.request('replace', {'text': 'old document input'});
      final rejected = expectLater(
        edit,
        throwsA(
          isA<NativeCallException>().having(
            (e) => e.code,
            'code',
            'staleDocument',
          ),
        ),
      );
      await replacement;
      await rejected;
      expect(transport.commands, isNot(contains('replace')));
    },
  );
  test(
    'failed post-edit info invalidates stale metadata until explicit refresh',
    () async {
      final transport = EditingTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      transport.autoComplete = false;
      final edit = document.request('replace', {'text': 'one'});
      final rejected = expectLater(edit, throwsA(isA<NativeCallException>()));
      await waitFor(() => transport.commands.last == 'replace');
      transport.edited(transport.serial);
      await waitFor(() => transport.commands.last == 'info');
      transport.completions.add({
        'ok': false,
        'requestId': transport.serial,
        'code': 'readFailed',
        'error': 'info unavailable',
      });
      await rejected;
      expect(document.info, isNull);
      transport.autoComplete = true;
      final info = await document.refreshInfo();
      expect(info.revision, 1);
    },
  );
  test(
    'a new editor lease invalidates old input owner and requests are bounded',
    () async {
      final transport = EditingTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final old = NativeEditorController(document);
      final next = NativeEditorController(document);
      addTearDown(next.dispose);
      addTearDown(old.dispose);
      expect(
        old.initialize,
        throwsA(
          isA<NativeCallException>().having(
            (e) => e.code,
            'code',
            'staleDocument',
          ),
        ),
      );
      expect(
        () => document.request('replace', {'text': 'a' * (256 * 1024)}),
        throwsA(
          isA<NativeCallException>().having(
            (e) => e.code,
            'code',
            'requestLimit',
          ),
        ),
      );
      expect(() => document.request('new'), throwsArgumentError);
      expect(
        () => document.request('replace', {'target': const {}}),
        throwsArgumentError,
      );
    },
  );
}
