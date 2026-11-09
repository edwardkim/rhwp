import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

void main() => runApp(const MaterialApp(home: NativeReferenceApp()));

class NativeReferenceApp extends StatefulWidget {
  const NativeReferenceApp({super.key});
  @override
  State<NativeReferenceApp> createState() => _NativeReferenceAppState();
}

class _NativeReferenceAppState extends State<NativeReferenceApp> {
  final _path = TextEditingController();
  final _pages = NativePagedViewportController();
  NativeDocument? _document;
  NativeGpuSession? _gpuSession;
  bool _gpuDetached = false;
  NativeTextureController? get _texture =>
      _gpuDetached ? null : _gpuSession?.controller;
  NativeDocumentLifecycle? _lifecycle;
  NativeEditorController? _editor;
  String? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    unawaited(_connect());
  }

  Future<void> _connect() async {
    if (_document != null) return;
    _lifecycle?.removeListener(_lifecycleChanged);
    _lifecycle?.dispose();
    _gpuSession?.removeListener(_gpuChanged);
    _gpuSession?.dispose();
    _gpuSession = null;
    _gpuDetached = false;
    _lifecycle = null;
    _editor = null;
    _pages.scrollTo(0, horizontal: 0);
    _pages.setZoom(1);
    setState(() {
      _busy = true;
      _error = null;
    });
    NativeDocument? document;
    NativeTextureController? texture;
    NativeTextureController Function()? createTexture;
    try {
      final bundlePath =
          Platform.environment['RHWP_RUNTIME_BUNDLE'] ??
          const String.fromEnvironment('RHWP_RUNTIME_BUNDLE');
      final transport = bundlePath.isEmpty
          ? await NativeOwnerTransport.connect(
              dispatcherLibrary: const String.fromEnvironment(
                'RHWP_OWNER_LIBRARY',
              ),
              nativeLibrary: const String.fromEnvironment(
                'RHWP_NATIVE_LIBRARY',
              ),
            )
          : await (await NativeRuntimeBundle.load(Directory(bundlePath)))
                .connect();
      document = await NativeDocument.connect(transport);
      await document.newDocument();
      if (const bool.fromEnvironment('RHWP_GPU')) {
        final platform = NativeTexturePlatform.current(
          preferContinuous: const bool.fromEnvironment('RHWP_GPU_CONTINUOUS'),
        );
        if (platform != null) {
          final presentation = await NativePresentation.connect(
            nativeLibrary: transport.nativeLibraryPath,
          );
          if (presentation.available) {
            final owner = document;
            createTexture = () => NativeTextureController(
              document: owner,
              presentation: presentation,
              platform: platform,
              trace: const bool.fromEnvironment('RHWP_GPU_TRACE')
                  ? NativeGpuTrace()
                  : null,
              useTileGroups:
                  platform != NativeTexturePlatform.macos &&
                  const bool.fromEnvironment(
                    'RHWP_GPU_TILES',
                    defaultValue: true,
                  ),
              allowImmutableOwnershipTransfer:
                  platform == NativeTexturePlatform.windows &&
                  const bool.fromEnvironment('RHWP_GPU_CONTINUOUS'),
            );
            texture = createTexture();
          }
        }
      }
      if (!mounted) {
        await _closeDocument(document, texture);
        return;
      }
      final gpu = texture == null
          ? null
          : NativeGpuSession(
              initialController: texture,
              createController: createTexture!,
            );
      gpu?.addListener(_gpuChanged);
      final owner = document;
      final lifecycle = NativeDocumentLifecycle(
        document: owner,
        files: NativeFileStore(owner),
        settleInput: () async {
          FocusManager.instance.primaryFocus?.unfocus();
          final editor = _editor;
          if (editor != null) await editor.focusLost();
        },
        confirmUnsaved: (_) => _unsavedDecision(),
        chooseSavePath: () => _askPath('저장할 HWP/HWPX 경로'),
        approveLoss: (report) => _approve(
          '저장 시 일부 내용이 보존되지 않습니다.',
          '${report['count']}개 항목의 손실 보고: $report',
        ),
        approveOverwrite: (_) =>
            _approve('기존 파일을 교체합니다.', '선택한 파일을 현재 문서로 교체하시겠습니까?'),
        retireViewport: () async {
          // Keep the view mounted underneath its cover until close returns.
          final texture = _texture;
          if (texture == null) return true;
          await texture.close();
          return texture.phase == NativeTexturePhase.closed &&
              !texture.retirementPending;
        },
      );
      lifecycle.addListener(_lifecycleChanged);
      setState(() {
        _lifecycle = lifecycle;
        _document = document;
        _gpuSession = gpu;
        _gpuDetached = false;
        _busy = false;
      });
    } catch (error) {
      await _closeDocument(document, texture);
      if (mounted) {
        setState(() {
          _error = '$error';
          _busy = false;
        });
      }
    }
  }

  Future<void> _closeDocument(
    NativeDocument? document,
    NativeTextureController? texture,
  ) async {
    try {
      await texture?.close();
    } finally {
      texture?.dispose();
      await document?.dispose();
    }
  }

  void _reportGpuTrace(NativeTextureController? texture) {
    final trace = texture?.trace;
    if (trace != null) debugPrint(jsonEncode(trace.toJson()));
  }

  void _gpuChanged() {
    if (mounted) setState(() {});
  }

  Future<void> _recoverGpu() async {
    final gpu = _gpuSession;
    if (gpu == null || _busy || _lifecycle?.isBusy == true) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final previous = gpu.controller;
      final replaced = await gpu.recover();
      _reportGpuTrace(previous);
      if (mounted) {
        setState(() {
          if (replaced) _gpuDetached = false;
          if (!replaced && gpu.recoveryDiagnostic != null) {
            _error = '${gpu.recoveryDiagnostic}';
          }
        });
      }
    } catch (error) {
      if (mounted) setState(() => _error = '$error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _lifecycleChanged() {
    if (mounted) setState(() {});
  }

  Future<bool> _approve(String title, String message) async {
    if (!mounted) return false;
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(title),
            content: SingleChildScrollView(child: Text(message)),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: const Text('취소'),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: const Text('계속'),
              ),
            ],
          ),
        ) ??
        false;
  }

  Future<NativeUnsavedDecision> _unsavedDecision() async {
    if (!mounted) return NativeUnsavedDecision.cancel;
    return await showDialog<NativeUnsavedDecision>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('변경 사항을 저장하시겠습니까?'),
            actions: [
              TextButton(
                onPressed: () =>
                    Navigator.pop(context, NativeUnsavedDecision.cancel),
                child: const Text('취소'),
              ),
              TextButton(
                onPressed: () =>
                    Navigator.pop(context, NativeUnsavedDecision.discard),
                child: const Text('폐기'),
              ),
              FilledButton(
                onPressed: () =>
                    Navigator.pop(context, NativeUnsavedDecision.save),
                child: const Text('저장'),
              ),
            ],
          ),
        ) ??
        NativeUnsavedDecision.cancel;
  }

  Future<String?> _askPath(String label) async {
    if (!mounted) return null;
    final input = TextEditingController(text: _path.text);
    try {
      final result = await showDialog<String>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(label),
          content: TextField(controller: input, autofocus: true),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('취소'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, input.text),
              child: const Text('선택'),
            ),
          ],
        ),
      );
      return result == null || result.trim().isEmpty ? null : result;
    } finally {
      // The dialog route can still paint its final frame after its result.
      WidgetsBinding.instance.addPostFrameCallback((_) => input.dispose());
    }
  }

  Future<void> _act(
    Future<bool> Function(NativeDocumentLifecycle) action,
  ) async {
    final lifecycle = _lifecycle;
    if (lifecycle == null || lifecycle.isBusy || lifecycle.isClosed) return;
    setState(() => _error = null);
    try {
      await action(lifecycle);
      if (mounted && lifecycle.isClosed) {
        setState(() => _document = null);
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = '$error';
          if (_document?.isDisposed == true) _document = null;
        });
      }
    } finally {
      final texture = _texture;
      if (mounted &&
          texture?.phase == NativeTexturePhase.closed &&
          !texture!.retirementPending) {
        // A later checkpoint failure can keep the document after its old GPU
        // viewport has already retired. Continue with the CPU reference view.
        setState(() => _gpuDetached = true);
      }
    }
  }

  Widget _documentView() {
    final view = NativePagedDocumentView(
      document: _document!,
      controller: _pages,
      gpuController: _texture,
      editable: _document!.transport.capabilities['atomicImeEdits'] == true,
      onControllerChanged: (editor) => _editor = editor,
      onViewportReady: (_) {
        if (mounted) setState(() {});
      },
    );
    final gpu = _gpuSession;
    // Keep this wrapper mounted across recovery and a retired CPU fallback.
    // Its observer owns neither the document nor renderer retirement.
    return gpu == null
        ? view
        : NativeGpuLifecycle(
            controller: gpu.controller,
            onPauseError: (error) {
              if (mounted) setState(() => _error = '$error');
            },
            child: view,
          );
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('rhwp Flutter Native Reference')),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(8),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _path,
                  enabled: !_busy && _lifecycle?.isBusy != true,
                  decoration: const InputDecoration(
                    labelText: 'HWP/HWPX 파일 경로',
                  ),
                ),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed:
                    _busy || _document == null || _lifecycle?.isBusy == true
                    ? null
                    : () => _act((lifecycle) => lifecycle.openPath(_path.text)),
                child: const Text('열기'),
              ),
            ],
          ),
        ),
        if (_document != null)
          Wrap(
            spacing: 8,
            children: [
              for (final entry
                  in <String, Future<bool> Function(NativeDocumentLifecycle)>{
                    '새 문서': (lifecycle) => lifecycle.newDocument(),
                    '저장': (lifecycle) => lifecycle.save(),
                    '다른 이름으로 저장': (lifecycle) async {
                      final path = await _askPath('저장할 HWP/HWPX 경로');
                      return path != null &&
                          await lifecycle.save(destination: path);
                    },
                    '복구 파일 열기': (lifecycle) =>
                        lifecycle.openPath(_path.text, recovery: true),
                    '복구 파일 만들기': (lifecycle) async {
                      final directory = await _askPath('복구 파일을 보관할 디렉터리');
                      return directory != null &&
                          await lifecycle.writeRecovery(directory) != null;
                    },
                    '문서 닫기': (lifecycle) => lifecycle.close(),
                  }.entries)
                TextButton(
                  onPressed: _busy || _lifecycle?.isBusy == true
                      ? null
                      : () => _act(entry.value),
                  child: Text(entry.key),
                ),
            ],
          ),
        if (_lifecycle?.sourcePath != null) Text(_lifecycle!.sourcePath!),
        if (_document != null)
          Wrap(
            spacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              TextButton(
                onPressed: _lifecycle?.isBusy == true || _pages.currentPage == 0
                    ? null
                    : () => _pages.jumpToPage(_pages.currentPage - 1),
                child: const Text('이전 쪽'),
              ),
              Text(
                '${_pages.currentPage + 1} / ${_document!.info?.pageCount ?? 0}쪽',
              ),
              TextButton(
                onPressed:
                    _lifecycle?.isBusy == true ||
                        _pages.currentPage + 1 >= _pages.pageCount
                    ? null
                    : () => _pages.jumpToPage(_pages.currentPage + 1),
                child: const Text('다음 쪽'),
              ),
              TextButton(
                onPressed: _lifecycle?.isBusy == true || _pages.zoom <= .25
                    ? null
                    : () => _pages.setZoom(
                        (_pages.zoom / 1.25).clamp(.25, 4.0).toDouble(),
                      ),
                child: const Text('축소'),
              ),
              Text('${(_pages.zoom * 100).round()}%'),
              TextButton(
                onPressed: _lifecycle?.isBusy == true || _pages.zoom >= 4
                    ? null
                    : () => _pages.setZoom(
                        (_pages.zoom * 1.25).clamp(.25, 4.0).toDouble(),
                      ),
                child: const Text('확대'),
              ),
            ],
          ),
        if (_gpuSession != null)
          TextButton(
            onPressed:
                _busy || _lifecycle?.isBusy == true || !_gpuSession!.canRecover
                ? null
                : _recoverGpu,
            child: const Text('GPU 다시 연결'),
          ),
        if (_document?.info?.isDirty == true) const Text('저장하지 않은 변경 사항'),
        if (_busy || _lifecycle?.isBusy == true)
          const LinearProgressIndicator(),
        if (!_busy && _document == null)
          TextButton(onPressed: _connect, child: const Text('문서 연결')),
        if (_error != null) Text(_error!),
        Expanded(
          child: _document == null
              ? const SizedBox.shrink()
              : AbsorbPointer(
                  absorbing:
                      _lifecycle?.isBusy == true ||
                      _gpuSession?.recovering == true,
                  child: _documentView(),
                ),
        ),
      ],
    ),
  );

  @override
  void dispose() {
    _path.dispose();
    _pages.dispose();
    _lifecycle?.removeListener(_lifecycleChanged);
    _lifecycle?.dispose();
    final document = _document;
    final gpu = _gpuSession;
    gpu?.removeListener(_gpuChanged);
    unawaited(() async {
      if (gpu != null) {
        final retired = await gpu.close();
        _reportGpuTrace(gpu.controller);
        if (!retired) {
          // Preserve an owner with uncertain native cleanup.
          return;
        }
        gpu.dispose();
      }
      await document?.dispose();
    }());
    super.dispose();
  }
}
