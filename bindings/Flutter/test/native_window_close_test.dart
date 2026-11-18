// Desktop exit waits for the actual host close result while its view is mounted.
import 'dart:async';
import 'dart:ui' show AppExitResponse;

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

void main() {
  testWidgets('duplicate exits share cleanup and keep the viewport mounted', (
    tester,
  ) async {
    final gate = Completer<bool>();
    var calls = 0;
    await tester.pumpWidget(
      NativeWindowClose(
        onClose: () {
          calls++;
          return gate.future;
        },
        child: const SizedBox(key: ValueKey('native-viewport')),
      ),
    );
    var exited = false;
    final first = tester.binding.handleRequestAppExit().then((value) {
      exited = true;
      return value;
    });
    final second = tester.binding.handleRequestAppExit();
    await tester.pump();
    expect(calls, 1);
    expect(exited, isFalse);
    expect(find.byKey(const ValueKey('native-viewport')), findsOneWidget);
    gate.complete(true);
    expect(await first, AppExitResponse.exit);
    expect(await second, AppExitResponse.exit);
  });

  testWidgets('cancelled close can be requested again', (tester) async {
    var calls = 0;
    await tester.pumpWidget(
      NativeWindowClose(
        onClose: () async => ++calls > 1,
        child: const SizedBox(),
      ),
    );
    expect(await tester.binding.handleRequestAppExit(), AppExitResponse.cancel);
    expect(await tester.binding.handleRequestAppExit(), AppExitResponse.exit);
    expect(calls, 2);
  });

  testWidgets('cleanup and diagnostic errors both cancel exit', (tester) async {
    Object? reported;
    await tester.pumpWidget(
      NativeWindowClose(
        onClose: () async => throw StateError('retirement pending'),
        onError: (error) {
          reported = error;
          throw StateError('diagnostic failed');
        },
        child: const SizedBox(),
      ),
    );
    expect(await tester.binding.handleRequestAppExit(), AppExitResponse.cancel);
    expect(reported, isA<StateError>());
    expect(tester.takeException(), isNull);
  });

  testWidgets('late result cannot authorize a replaced close owner', (
    tester,
  ) async {
    final old = Completer<bool>();
    await tester.pumpWidget(
      NativeWindowClose(onClose: () => old.future, child: const SizedBox()),
    );
    final exit = tester.binding.handleRequestAppExit();
    await tester.pumpWidget(
      NativeWindowClose(onClose: () async => false, child: const SizedBox()),
    );
    old.complete(true);
    expect(await exit, AppExitResponse.cancel);
    expect(await tester.binding.handleRequestAppExit(), AppExitResponse.cancel);
  });

  testWidgets('unmount cancels late authorization and removes the observer', (
    tester,
  ) async {
    final gate = Completer<bool>();
    await tester.pumpWidget(
      NativeWindowClose(onClose: () => gate.future, child: const SizedBox()),
    );
    final exit = tester.binding.handleRequestAppExit();
    await tester.pumpWidget(const SizedBox());
    gate.complete(true);
    expect(await exit, AppExitResponse.cancel);
    expect(await tester.binding.handleRequestAppExit(), AppExitResponse.exit);
  });
}
