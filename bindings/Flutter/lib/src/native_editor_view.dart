import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';

import 'native_document.dart';
import 'native_document_view.dart';
import 'native_editor_controller.dart';
import 'native_viewport.dart';

typedef NativeEditorViewportBuilder = Widget Function(
  BuildContext context,
  NativeDocument document,
  int page,
  NativePageViewport viewport,
  WidgetBuilder cpuFallback,
);

/// A single-page input/semantics reference surface, over CPU or a caller's
/// external-texture viewport. Real platform IME/AT qualification is separate.
class NativeEditorView extends StatefulWidget {
  const NativeEditorView({
    super.key,
    required this.document,
    this.page = 0,
    this.viewportBuilder,
  });
  final NativeDocument document;
  final int page;
  final NativeEditorViewportBuilder? viewportBuilder;
  @override
  State<NativeEditorView> createState() => _NativeEditorViewState();
}

class _NativeEditorViewState extends State<NativeEditorView>
    with TextInputClient, DeltaTextInputClient {
  final _focus = FocusNode();
  NativeEditorController? _editor;
  TextInputConnection? _connection;
  StreamSubscription<NativeDocumentInfo>? _changes;
  NativePageSize? _page;
  NativePageViewport? _geometry;
  Object? _error;
  int _binding = 0;
  int? _boundDocument;
  Size _size = Size.zero;
  Object? _dragTicket;
  bool _dragActive = false;
  Offset? _dragLatest;

  @override
  void initState() {
    super.initState();
    _focus.addListener(_focusChanged);
    _listen();
  }

  void _listen() {
    _changes = widget.document.changes.listen((info) {
      if (info.documentId != _boundDocument) unawaited(_bind());
    });
    unawaited(_bind());
  }

  Future<void> _bind() async {
    final binding = ++_binding;
    _boundDocument = widget.document.info?.documentId;
    _closeInput();
    _dragTicket = null;
    _dragLatest = null;
    _dragActive = false;
    final old = _editor;
    old?.removeListener(_changed);
    old?.dispose();
    _editor = null;
    _page = null;
    if (_boundDocument == null || !mounted || widget.document.isDisposed) {
      return;
    }
    final editor = NativeEditorController(widget.document);
    _editor = editor;
    editor.addListener(_changed);
    try {
      await editor.initialize();
      final page = await widget.document.pageSize(widget.page);
      if (!mounted || binding != _binding) return;
      setState(() {
        _page = page;
        _error = null;
      });
      _openInput();
    } catch (error) {
      if (mounted && binding == _binding) {
        setState(() {
          _error = error;
        });
      }
    }
  }

  void _changed() {
    if (!mounted) return;
    final editor = _editor;
    setState(() {
      _error = editor?.error;
    });
    if (editor?.ready == true) {
      _openInput();
      if (editor!.pending == 0) _connection?.setEditingState(editor.value);
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _inputGeometry();
      });
    } else {
      _closeInput();
    }
  }

  void _perform(Future<void> Function() action) {
    try {
      unawaited(
        action().catchError((Object error, StackTrace stack) {
          if (mounted) {
            setState(() {
              _error = error;
            });
          }
        }),
      );
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error;
        });
      }
    }
  }

  void _openInput() {
    if (!_focus.hasFocus ||
        _editor?.ready != true ||
        _connection?.attached == true) {
      return;
    }
    _connection = TextInput.attach(
      this,
      const TextInputConfiguration(
        inputType: TextInputType.multiline,
        inputAction: TextInputAction.newline,
        enableDeltaModel: true,
        autocorrect: false,
        enableSuggestions: false,
      ),
    );
    _connection!.setEditingState(_editor!.value);
    _connection!.show();
    _inputGeometry();
  }

  void _closeInput() {
    _connection?.close();
    _connection = null;
  }

  void _focusChanged() {
    if (_focus.hasFocus) {
      _openInput();
    } else {
      _closeInput();
      final editor = _editor;
      if (editor != null) _perform(editor.focusLost);
    }
    if (mounted) setState(() {});
  }

  void _inputGeometry() {
    final connection = _connection, geometry = _geometry;
    final box = context.findRenderObject();
    if (connection == null ||
        !connection.attached ||
        geometry == null ||
        box is! RenderBox ||
        !box.hasSize) {
      return;
    }
    connection.setEditableSizeAndTransform(box.size, box.getTransformTo(null));
    final caret = _editor?.caret;
    if (caret != null && caret['page'] == widget.page) {
      final rect = geometry.caret(caret);
      connection.setCaretRect(rect);
      if (_editor!.value.composing.isValid) connection.setComposingRect(rect);
    }
  }

  KeyEventResult _key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final editor = _editor;
    if (editor == null) return KeyEventResult.ignored;
    final keyboard = HardwareKeyboard.instance;
    final shortcut =
        (keyboard.isControlPressed || keyboard.isMetaPressed) &&
        !keyboard.isAltPressed;
    if (shortcut) {
      final key = event.logicalKey;
      if (key == LogicalKeyboardKey.keyC) {
        _perform(editor.copy);
      } else if (key == LogicalKeyboardKey.keyX) {
        _perform(() => editor.copy(cut: true));
      } else if (key == LogicalKeyboardKey.keyV) {
        _perform(editor.paste);
      } else if (key == LogicalKeyboardKey.keyA) {
        _perform(() => editor.command('selectAll'));
      } else if (key == LogicalKeyboardKey.keyZ) {
        _perform(
          () => editor.command(keyboard.isShiftPressed ? 'redo' : 'undo'),
        );
      } else if (key == LogicalKeyboardKey.keyY) {
        _perform(() => editor.command('redo'));
      } else {
        return KeyEventResult.ignored;
      }
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.escape) {
      _perform(editor.focusLost);
      return KeyEventResult.handled;
    }
    // Let the platform IME own editing keys while composing; handling them here
    // as native document commands would apply the same input twice.
    if (editor.value.composing.isValid && !editor.value.composing.isCollapsed) {
      return KeyEventResult.ignored;
    }
    final command = {
      LogicalKeyboardKey.arrowLeft: 'moveLeft',
      LogicalKeyboardKey.arrowRight: 'moveRight',
      LogicalKeyboardKey.arrowUp: 'moveUp',
      LogicalKeyboardKey.arrowDown: 'moveDown',
      LogicalKeyboardKey.home: 'home',
      LogicalKeyboardKey.end: 'end',
      LogicalKeyboardKey.backspace: 'deleteBackward',
      LogicalKeyboardKey.delete: 'deleteForward',
      LogicalKeyboardKey.enter: 'splitParagraph',
    }[event.logicalKey];
    if (command == null) return KeyEventResult.ignored;
    _perform(() => editor.command(command, extend: keyboard.isShiftPressed));
    return KeyEventResult.handled;
  }

  void _point(Offset local, {bool extend = false}) {
    final geometry = _geometry, editor = _editor;
    if (geometry == null || editor == null) return;
    final point = geometry.toNative(local);
    if (point.dx < 0 ||
        point.dy < 0 ||
        point.dx > geometry.page.width ||
        point.dy > geometry.page.height) {
      return;
    }
    _perform(
      () => editor.point(widget.page, point.dx, point.dy, extend: extend),
    );
  }

  void _drag(Offset local) {
    _dragLatest = local;
    if (_dragTicket != null) return;
    final ticket = Object();
    _dragTicket = ticket;
    final binding = _binding;
    final editor = _editor, geometry = _geometry;
    if (editor == null || geometry == null) {
      _dragTicket = null;
      return;
    }
    final point = geometry.toNative(local);
    _perform(() async {
      try {
        await editor.point(widget.page, point.dx, point.dy, extend: true);
      } finally {
        if (identical(_dragTicket, ticket)) {
          _dragTicket = null;
          final latest = _dragLatest;
          _dragLatest = null;
          if (mounted &&
              binding == _binding &&
              latest != null &&
              latest != local) {
            _drag(latest);
          }
        }
      }
    });
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      _size = Size(constraints.maxWidth, constraints.maxHeight);
      final page = _page;
      final geometry = page == null || !_size.isFinite || _size.isEmpty
          ? null
          : NativePageViewport.fit(page, _size);
      _geometry = geometry;
      Widget cpuFallback(BuildContext context) =>
          NativeDocumentView(document: widget.document, page: widget.page);
      final editor = _editor;
      final semantics = editor?.semantics;
      final nodes = (semantics?['nodes'] as List?) ?? const [];
      final body = nodes
          .take(64)
          .map((node) => (node as Map)['text']! as String)
          .join('\n');
      return Focus(
        focusNode: _focus,
        onKeyEvent: _key,
        child: Semantics(
          container: true,
          label: semantics?['complete'] == true ? '문서 본문' : '문서 본문 일부',
          value: body,
          child: Semantics(
            container: true,
            textField: true,
            multiline: true,
            label: '현재 문단 입력',
            focused: _focus.hasFocus,
            readOnly: editor?.ready != true,
            value: editor?.value.text ?? '',
            onSetSelection: editor?.ready == true
                ? (selection) =>
                      _perform(() => editor!.setLocalSelection(selection))
                : null,
            onSetText: editor?.ready == true
                ? (text) => _perform(
                    () => editor!.setEditingValue(
                      TextEditingValue(
                        text: text,
                        selection: TextSelection.collapsed(offset: text.length),
                      ),
                    ),
                  )
                : null,
            child: _SelectionSemantics(
              selection: editor?.value.selection,
              child: Listener(
                onPointerDown: (event) {
                  if ((event.buttons & 1) != 0) {
                    _dragActive = true;
                    _focus.requestFocus();
                    _point(event.localPosition);
                  }
                },
                onPointerMove: (event) {
                  if (_dragActive && (event.buttons & 1) != 0) {
                    _drag(event.localPosition);
                  }
                },
                onPointerUp: (event) {
                  if (_dragActive) {
                    _dragActive = false;
                    if (_dragTicket != null) {
                      _dragLatest = event.localPosition;
                    } else {
                      _point(event.localPosition, extend: true);
                    }
                  }
                },
                onPointerCancel: (_) {
                  _dragActive = false;
                  _dragLatest = null;
                },
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    if (geometry != null && widget.viewportBuilder != null)
                      widget.viewportBuilder!(
                        context,
                        widget.document,
                        widget.page,
                        geometry,
                        cpuFallback,
                      )
                    else
                      cpuFallback(context),
                    if (_geometry != null && editor != null)
                      IgnorePointer(
                        child: CustomPaint(
                          painter: _Overlay(
                            editor,
                            _geometry!,
                            widget.page,
                            _focus.hasFocus,
                          ),
                        ),
                      ),
                    if (_error != null)
                      Align(
                        alignment: Alignment.bottomCenter,
                        child: ColoredBox(
                          color: const Color(0xccffffff),
                          child: Text('입력 상태: $_error'),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
      );
    },
  );
  @override
  void didUpdateWidget(NativeEditorView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(widget.document, oldWidget.document)) {
      unawaited(_changes?.cancel());
      _listen();
    } else if (widget.page != oldWidget.page) {
      unawaited(_bind());
    }
  }

  @override
  TextEditingValue? get currentTextEditingValue => _editor?.value;
  @override
  AutofillScope? get currentAutofillScope => null;
  @override
  void updateEditingValueWithDeltas(List<TextEditingDelta> deltas) {
    final editor = _editor;
    if (editor != null) _perform(() => editor.applyDeltas(deltas));
  }

  @override
  void updateEditingValue(TextEditingValue value) {
    final editor = _editor;
    if (editor != null) _perform(() => editor.setEditingValue(value));
  }

  @override
  void performAction(TextInputAction action) {
    if (action == TextInputAction.newline) {
      final editor = _editor;
      if (editor != null) _perform(() => editor.command('splitParagraph'));
    }
  }

  @override
  void performSelector(String selectorName) {
    final editor = _editor;
    if (editor == null) return;
    final name = selectorName.replaceAll(':', '');
    final extend = name.endsWith('AndModifySelection');
    final base = name.replaceAll('AndModifySelection', '');
    final command = {
      'moveLeft': 'moveLeft',
      'moveRight': 'moveRight',
      'moveUp': 'moveUp',
      'moveDown': 'moveDown',
      'moveToBeginningOfLine': 'home',
      'moveToEndOfLine': 'end',
      'deleteBackward': 'deleteBackward',
      'deleteForward': 'deleteForward',
      'insertNewline': 'splitParagraph',
      'selectAll': 'selectAll',
      'undo': 'undo',
      'redo': 'redo',
    }[base];
    if (command != null) {
      _perform(() => editor.command(command, extend: extend));
    }
  }

  @override
  void performPrivateCommand(String action, Map<String, dynamic> data) {}
  @override
  void updateFloatingCursor(RawFloatingCursorPoint point) {}
  @override
  void showAutocorrectionPromptRect(int start, int end) {}
  @override
  void connectionClosed() {
    _connection = null;
    final editor = _editor;
    if (editor != null) _perform(editor.focusLost);
  }

  @override
  void dispose() {
    _binding++;
    _closeInput();
    unawaited(_changes?.cancel());
    final editor = _editor;
    editor?.removeListener(_changed);
    if (editor != null) {
      try {
        unawaited(
          editor
              .focusLost()
              .then<void>((_) {}, onError: (Object _, StackTrace _) {})
              .whenComplete(editor.dispose),
        );
      } catch (_) {
        editor.dispose();
      }
    }
    _focus.removeListener(_focusChanged);
    _focus.dispose();
    super.dispose();
  }
}

class _Overlay extends CustomPainter {
  _Overlay(this.editor, this.geometry, this.page, this.focused);
  final NativeEditorController editor;
  final NativePageViewport geometry;
  final int page;
  final bool focused;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.clipRect(Offset.zero & size);
    for (final selection in editor.rectangles) {
      if (selection['page'] == page) {
        canvas.drawRect(
          geometry.rectangle(selection['bounds']! as Map),
          Paint()..color = const Color(0x4d3278e6),
        );
      }
    }
    final caret = editor.caret;
    if (focused && caret != null && caret['page'] == page) {
      canvas.drawRect(
        geometry.caret(caret),
        Paint()..color = const Color(0xff202020),
      );
    }
  }

  @override
  bool shouldRepaint(_Overlay oldDelegate) => true; // Snapshot fields change on one controller.
}

class _SelectionSemantics extends SingleChildRenderObjectWidget {
  const _SelectionSemantics({required this.selection, required super.child});
  final TextSelection? selection;
  @override
  RenderObject createRenderObject(BuildContext context) =>
      _SelectionSemanticsBox(selection);
  @override
  void updateRenderObject(
    BuildContext context,
    covariant _SelectionSemanticsBox renderObject,
  ) {
    renderObject.selection = selection;
  }
}

class _SelectionSemanticsBox extends RenderProxyBox {
  _SelectionSemanticsBox(this._selection);
  TextSelection? _selection;
  set selection(TextSelection? selection) {
    if (_selection == selection) return;
    _selection = selection;
    markNeedsSemanticsUpdate();
  }

  @override
  void describeSemanticsConfiguration(SemanticsConfiguration config) {
    super.describeSemanticsConfiguration(config);
    final selection = _selection;
    if (selection != null && selection.isValid) {
      config.textSelection = selection;
    }
  }
}
