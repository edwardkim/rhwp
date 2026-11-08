import 'dart:convert';
import 'dart:io';

import 'package:rhwp_native_reference/rhwp_native.dart';

Future<void> main(List<String> arguments) async {
  if (arguments.length < 2 || arguments.length > 3) {
    stderr.writeln(
      'Usage: dart run tool/native_ffi_smoke.dart <owner-library> <rhwp-native-library> [input.hwp(x)]',
    );
    exitCode = 64;
    return;
  }
  final transport = await NativeOwnerTransport.connect(
    dispatcherLibrary: arguments[0],
    nativeLibrary: arguments[1],
  );
  final document = await NativeDocument.connect(transport);
  NativeRasterFrame? retained;
  var before = 0;
  try {
    await document.newDocument();
    if (arguments.length == 3) {
      final input = await File(arguments[2]).open();
      try {
        await document.openBytes(await input.read(maxReferenceBytes + 1));
      } finally {
        await input.close();
      }
    }
    final info = await document.refreshInfo();
    if (info.pageCount <= 0) throw StateError('Native open returned no pages.');
    final size = await document.pageSize(0);
    final width = size.width < 64 ? size.width : 64.0;
    final height = size.height < 64 ? size.height : 64.0;
    retained = await document.renderRegion(
      page: 0,
      x: 0,
      y: 0,
      width: width,
      height: height,
      scale: 1,
    );
    if (retained == null ||
        retained.pixels.length != retained.stride * retained.height) {
      throw StateError('Native CPU render did not return a complete frame.');
    }
    before = retained.pixels.fold<int>(
      0,
      (sum, byte) => (sum + byte) & 0x7fffffff,
    );
    final second = await document.renderRegion(
      page: 0,
      x: 0,
      y: 0,
      width: width,
      height: height,
      scale: 1,
    );
    if (second == null) throw StateError('Repeated native render was skipped.');
  } finally {
    await document.dispose();
  }
  if (retained.pixels.fold<int>(0, (sum, byte) => (sum + byte) & 0x7fffffff) !=
      before) {
    throw StateError(
      'Dart-owned pixels did not survive native owner shutdown.',
    );
  }
  stdout.writeln(
    jsonEncode({
      'ok': true,
      'inputOpened': arguments.length == 3,
      'ownerThreadDispatcher': true,
      'nativeWorker': true,
      'frame': [retained.width, retained.height, retained.stride],
      'ownedPixelsSurviveClose': true,
      'browser': false,
      'presentationActorUsed': false,
    }),
  );
}
