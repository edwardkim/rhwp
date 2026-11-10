import 'dart:convert';
import 'dart:io';

import 'package:rhwp_native_reference/rhwp_native.dart';

/// Explicitly load a trusted supplied bundle and check actual owner/Native ABI.
/// This is not a renderer, Flutter plugin or desktop presentation qualification.
Future<void> main(List<String> args) async {
  if (args.length != 1) {
    stderr.writeln(
      'Usage: dart run tool/runtime_bundle_smoke.dart <trusted-bundle>',
    );
    exitCode = 64;
    return;
  }
  final bundle = await NativeRuntimeBundle.load(Directory(args.single));
  final owner = await bundle.connect();
  try {
    stdout.writeln(
      jsonEncode({
        'ok': true,
        'platform': bundle.platform,
        'architecture': bundle.architecture,
        'ownerAbi2': owner.capabilities['ownerPresentationCleanup'] == true,
        'nativeAbi': owner.capabilities['abiVersion'],
        'worker': owner.capabilities['worker'],
        'nativeSkia': owner.capabilities['nativeSkia'],
        'sourceRevisions': bundle.sourceRevisions,
        'runtimeLaunched': false,
        'presentationVerified': false,
      }),
    );
  } finally {
    await owner.close();
  }
}
