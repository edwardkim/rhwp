import 'dart:convert';

import 'package:flutter/services.dart';

/// UTF-16 offsets always address the exact bounded native context, including
/// its preview. Never clamp a surrogate or silently trim the platform oldText.
void checkNativeTextBoundary(String text, int offset) {
  if (offset < 0 ||
      offset > text.length ||
      (offset > 0 &&
          offset < text.length &&
          text.codeUnitAt(offset - 1) >= 0xd800 &&
          text.codeUnitAt(offset - 1) <= 0xdbff &&
          text.codeUnitAt(offset) >= 0xdc00 &&
          text.codeUnitAt(offset) <= 0xdfff)) {
    throw const FormatException(
      'IME offset splits a Unicode scalar or context.',
    );
  }
}

void checkNativeText(String text) {
  for (var i = 0; i < text.length; i++) {
    final unit = text.codeUnitAt(i);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      if (++i >= text.length ||
          text.codeUnitAt(i) < 0xdc00 ||
          text.codeUnitAt(i) > 0xdfff) {
        throw const FormatException('Unpaired UTF-16 surrogate.');
      }
    } else if (unit >= 0xdc00 && unit <= 0xdfff) {
      throw const FormatException('Unpaired UTF-16 surrogate.');
    }
  }
}

int scalarToUtf16(String text, int scalar) {
  if (scalar < 0 || scalar > text.runes.length) {
    throw const FormatException('Invalid native scalar.');
  }
  return text.runes
      .take(scalar)
      .fold(0, (units, rune) => units + (rune > 0xffff ? 2 : 1));
}

class NativeImeSnapshot {
  NativeImeSnapshot(Map<String, Object?> json)
    : expected = Map.unmodifiable(json) {
    final context = json['context']! as Map;
    final text = context['text']! as String;
    checkNativeText(text);
    if (utf8.encode(text).length > 16384 || text.runes.length > 4096) {
      throw const FormatException('Native IME context exceeds its bound.');
    }
    final cursor = scalarToUtf16(text, context['cursorScalar']! as int);
    final anchor = scalarToUtf16(text, context['anchorScalar']! as int);
    final preedit = json['preedit'] as Map?;
    final composing = preedit == null
        ? TextRange.empty
        : TextRange(
            start: preedit['start']! as int,
            end: preedit['end']! as int,
          );
    if (composing.isValid) {
      checkNativeTextBoundary(text, composing.start);
      checkNativeTextBoundary(text, composing.end);
      if (composing.start > composing.end) {
        throw const FormatException('Reversed preedit.');
      }
    }
    value = TextEditingValue(
      text: text,
      selection: TextSelection(baseOffset: anchor, extentOffset: cursor),
      composing: composing.isCollapsed ? TextRange.empty : composing,
    );
  }
  final Map<String, Object?> expected;
  late final TextEditingValue value;
}

({TextRange range, String text}) nativeTextDifference(
  String oldText,
  String newText,
) {
  var start = 0;
  while (start < oldText.length &&
      start < newText.length &&
      oldText.codeUnitAt(start) == newText.codeUnitAt(start)) {
    start++;
  }
  if (start > 0 &&
      start < oldText.length &&
      oldText.codeUnitAt(start) >= 0xdc00 &&
      oldText.codeUnitAt(start) <= 0xdfff) {
    start--;
  }
  var oldEnd = oldText.length, newEnd = newText.length;
  while (oldEnd > start &&
      newEnd > start &&
      oldText.codeUnitAt(oldEnd - 1) == newText.codeUnitAt(newEnd - 1)) {
    oldEnd--;
    newEnd--;
  }
  if (oldEnd < oldText.length &&
      oldText.codeUnitAt(oldEnd) >= 0xdc00 &&
      oldText.codeUnitAt(oldEnd) <= 0xdfff) {
    oldEnd++;
    newEnd++;
  }
  return (
    range: TextRange(start: start, end: oldEnd),
    text: newText.substring(start, newEnd),
  );
}

Map<String, Object?> nativeImeDelta(
  NativeImeSnapshot snapshot,
  TextEditingValue before,
  TextEditingValue after,
) {
  final native = snapshot.value;
  if (native.text != before.text ||
      native.selection.baseOffset != before.selection.baseOffset ||
      native.selection.extentOffset != before.selection.extentOffset ||
      native.composing != before.composing) {
    throw const FormatException('Stale platform IME state.');
  }
  for (final value in [before, after]) {
    checkNativeText(value.text);
    if (utf8.encode(value.text).length > 16384 ||
        value.text.runes.length > 4096) {
      throw const FormatException('IME text exceeds native context.');
    }
    checkNativeTextBoundary(value.text, value.selection.baseOffset);
    checkNativeTextBoundary(value.text, value.selection.extentOffset);
    if (value.composing.isValid && !value.composing.isCollapsed) {
      checkNativeTextBoundary(value.text, value.composing.start);
      checkNativeTextBoundary(value.text, value.composing.end);
      if (value.composing.start > value.composing.end) {
        throw const FormatException('Reversed composing range.');
      }
    }
  }
  final difference = nativeTextDifference(before.text, after.text);
  final range = difference.range;
  final text = difference.text;
  Map<String, Object?> replacement(TextRange r, String t) => {
    'range': {'start': r.start, 'end': r.end},
    'text': t,
  };
  final selection = {
    'anchor': after.selection.baseOffset,
    'focus': after.selection.extentOffset,
  };
  final composing = after.composing;
  if (!composing.isValid || composing.isCollapsed) {
    return {
      'kind': 'commit',
      'replacement': replacement(range, text),
      'selection': selection,
    };
  }
  if (after.selection.start < composing.start ||
      after.selection.end > composing.end) {
    throw const FormatException('Selection lies outside the active preedit.');
  }
  final oldPreedit = before.composing;
  if (oldPreedit.isValid && !oldPreedit.isCollapsed) {
    final newEnd = oldPreedit.end + text.length - (range.end - range.start);
    if (range.start >= oldPreedit.start &&
        range.end <= oldPreedit.end &&
        composing.start == oldPreedit.start &&
        composing.end == newEnd) {
      return {
        'kind': 'continue',
        'replacement': replacement(range, text),
        'selection': selection,
      };
    }
    if (before.text == after.text && composing == oldPreedit) {
      return {
        'kind': 'continue',
        'replacement': replacement(TextRange.collapsed(oldPreedit.start), ''),
        'selection': selection,
      };
    }
  } else {
    if (before.text == after.text) {
      return {
        'kind': 'commit',
        'replacement': replacement(const TextRange.collapsed(0), ''),
        'selection': {
          'anchor': before.selection.baseOffset,
          'focus': before.selection.extentOffset,
        },
        'preedit': {
          'replacement': replacement(
            composing,
            after.text.substring(composing.start, composing.end),
          ),
          'selection': selection,
        },
      };
    }
    if (composing.start == range.start &&
        composing.end == range.start + text.length) {
      return {
        'kind': 'commit',
        'replacement': replacement(const TextRange.collapsed(0), ''),
        'selection': {
          'anchor': before.selection.baseOffset,
          'focus': before.selection.extentOffset,
        },
        'preedit': {
          'replacement': replacement(range, text),
          'selection': selection,
        },
      };
    }
  }
  // A committed prefix followed by a new preview can arrive in one platform
  // delta. The preview must lie in the newly inserted run; moving an existing
  // composing range without that evidence is rejected and resynchronized.
  if (composing.start >= range.start &&
      composing.end <= range.start + text.length) {
    final localStart = composing.start - range.start,
        localEnd = composing.end - range.start;
    final committed = text.substring(0, localStart) + text.substring(localEnd);
    if (committed.isEmpty && oldPreedit.isValid) {
      throw const FormatException('Ambiguous preedit restart.');
    }
    return {
      'kind': 'commit',
      'replacement': replacement(range, committed),
      'preedit': {
        'replacement': replacement(
          TextRange.collapsed(composing.start),
          text.substring(localStart, localEnd),
        ),
        'selection': selection,
      },
    };
  }
  throw const FormatException(
    'Unsupported composing transition; refresh the native context.',
  );
}
