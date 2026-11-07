import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_ime.dart';

NativeImeSnapshot snapshot(
  String text, {
  int cursor = 0,
  int? anchor,
  TextRange composing = TextRange.empty,
}) => NativeImeSnapshot({
  'revision': 4,
  'fontGeneration': 0,
  'compositionId': composing.isValid ? 9 : null,
  'selection': const {},
  'preedit': composing.isValid
      ? {'start': composing.start, 'end': composing.end}
      : null,
  'context': {
    'text': text,
    'cursorScalar': text.substring(0, cursor).runes.length,
    'anchorScalar': text.substring(0, anchor ?? cursor).runes.length,
    'selectedText': '',
    'position': const {
      'section': 0,
      'paragraph': 0,
      'scalarOffset': 0,
      'cellPath': [],
    },
    'absoluteStartScalar': 0,
    'textTruncated': false,
    'selectionTruncated': false,
  },
});

void main() {
  test(
    'non-BMP difference preserves whole scalar and unchanged prefix/suffix',
    () {
      final difference = nativeTextDifference('가😀나', '가😁나');
      expect(difference.range, const TextRange(start: 1, end: 3));
      expect(difference.text, '😁');
      expect(() => checkNativeTextBoundary('😀', 1), throwsFormatException);
      expect(scalarToUtf16('가😀나', 2), 3);
    },
  );
  test('unpaired UTF16 is rejected before JSON can replace it', () {
    expect(
      () => checkNativeText(String.fromCharCode(0xd800)),
      throwsFormatException,
    );
    expect(
      () => checkNativeText(String.fromCharCode(0xdc00)),
      throwsFormatException,
    );
  });
  test('starting preedit preserves original selection as cancel baseline', () {
    final state = snapshot('abc', cursor: 3, anchor: 0);
    final edit = nativeImeDelta(
      state,
      state.value,
      const TextEditingValue(
        text: 'ㅎ',
        selection: TextSelection.collapsed(offset: 1),
        composing: TextRange(start: 0, end: 1),
      ),
    );
    expect(edit['kind'], 'commit');
    expect(edit['selection'], {'anchor': 0, 'focus': 3});
    expect((edit['replacement']! as Map)['text'], '');
    expect(((edit['preedit']! as Map)['replacement']! as Map)['range'], {
      'start': 0,
      'end': 3,
    });
  });
  test(
    'preedit update continues one composition instead of committing previews',
    () {
      final state = snapshot(
        'ㅎ',
        cursor: 1,
        composing: const TextRange(start: 0, end: 1),
      );
      final edit = nativeImeDelta(
        state,
        state.value,
        const TextEditingValue(
          text: '하',
          selection: TextSelection.collapsed(offset: 1),
          composing: TextRange(start: 0, end: 1),
        ),
      );
      expect(edit['kind'], 'continue');
      expect((edit['replacement']! as Map)['text'], '하');
    },
  );
  test('composition commit does not start another preview', () {
    final state = snapshot(
      '하',
      cursor: 1,
      composing: const TextRange(start: 0, end: 1),
    );
    final edit = nativeImeDelta(
      state,
      state.value,
      const TextEditingValue(
        text: '한',
        selection: TextSelection.collapsed(offset: 1),
      ),
    );
    expect(edit['kind'], 'commit');
    expect(edit['preedit'], isNull);
  });
  test('commit plus following preedit splits the cancellation baseline', () {
    final state = snapshot(
      '하',
      cursor: 1,
      composing: const TextRange(start: 0, end: 1),
    );
    final edit = nativeImeDelta(
      state,
      state.value,
      const TextEditingValue(
        text: '한ㄱ',
        selection: TextSelection.collapsed(offset: 2),
        composing: TextRange(start: 1, end: 2),
      ),
    );
    expect((edit['replacement']! as Map)['text'], '한');
    final preedit = (edit['preedit']! as Map)['replacement']! as Map;
    expect(preedit['text'], 'ㄱ');
    expect(preedit['range'], {'start': 1, 'end': 1});
  });
  test('selection-only composition start retains existing selected bytes', () {
    final state = snapshot('abc', cursor: 2, anchor: 1);
    final edit = nativeImeDelta(
      state,
      state.value,
      state.value.copyWith(composing: const TextRange(start: 1, end: 2)),
    );
    expect(edit['selection'], {'anchor': 1, 'focus': 2});
    expect(((edit['preedit']! as Map)['replacement']! as Map)['text'], 'b');
  });
  test('selection inside preedit is a continue and keeps the text', () {
    final state = snapshot(
      'a한b',
      cursor: 2,
      composing: const TextRange(start: 1, end: 2),
    );
    final edit = nativeImeDelta(
      state,
      state.value,
      state.value.copyWith(selection: const TextSelection.collapsed(offset: 1)),
    );
    expect(edit['kind'], 'continue');
    expect((edit['replacement']! as Map)['text'], '');
  });
  test('stale text, surrogate selection and excessive context fail closed', () {
    final state = snapshot('😀', cursor: 2);
    expect(
      () => nativeImeDelta(state, state.value.copyWith(text: 'X'), state.value),
      throwsFormatException,
    );
    expect(
      () => nativeImeDelta(
        state,
        state.value,
        state.value.copyWith(
          selection: const TextSelection.collapsed(offset: 1),
        ),
      ),
      throwsFormatException,
    );
    expect(() => snapshot('a' * 4097), throwsFormatException);
  });
  test('moving an existing composing range without replacement evidence is rejected', () {
    final state = snapshot(
      'abcd',
      cursor: 2,
      composing: const TextRange(start: 1, end: 2),
    );
    expect(
      () => nativeImeDelta(
        state,
        state.value,
        state.value.copyWith(
          selection: const TextSelection.collapsed(offset: 3),
          composing: const TextRange(start: 2, end: 3),
        ),
      ),
      throwsFormatException,
    );
  });
}
