import 'dart:async';
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
  NativeDocument? _document;
  NativeTextureController? _texture;
  String? _error;
  bool _busy = true;

  @override
  void initState() {
    super.initState();
    unawaited(_connect());
  }

  Future<void> _connect() async {
    NativeDocument? document;
    NativeTextureController? texture;
    try {
      final transport = await NativeOwnerTransport.connect(
        dispatcherLibrary: const String.fromEnvironment('RHWP_OWNER_LIBRARY'),
        nativeLibrary: const String.fromEnvironment('RHWP_NATIVE_LIBRARY'),
      );
      document = await NativeDocument.connect(transport);
      await document.newDocument();
      if (const bool.fromEnvironment('RHWP_GPU')) {
        final platform = NativeTexturePlatform.current();
        if (platform != null) {
          final presentation = await NativePresentation.connect(
            nativeLibrary: transport.nativeLibraryPath,
          );
          if (presentation.available) {
            texture = NativeTextureController(
              document: document,
              presentation: presentation,
              platform: platform,
            );
          }
        }
      }
      if (!mounted) {
        await _closeDocument(document, texture);
        return;
      }
      setState(() {
        _document = document;
        _texture = texture;
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

  Future<void> _open() async {
    final document = _document;
    if (document == null) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final file = File(_path.text);
      if (await file.length() > maxReferenceBytes) {
        throw StateError('참조 앱은 16 MiB 이하 문서를 엽니다.');
      }
      final source = await file.open();
      try {
        final bytes = await source.read(maxReferenceBytes + 1);
        await document.openBytes(bytes);
      } finally {
        await source.close();
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = '$error';
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _busy = false;
        });
      }
    }
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
                  enabled: !_busy,
                  decoration: const InputDecoration(
                    labelText: 'HWP/HWPX 파일 경로',
                  ),
                ),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: _busy || _document == null ? null : _open,
                child: const Text('열기'),
              ),
            ],
          ),
        ),
        if (_busy) const LinearProgressIndicator(),
        if (_error != null) Text(_error!),
        Expanded(
          child: _document == null
              ? const SizedBox.shrink()
              : _document!.transport.capabilities['atomicImeEdits'] == true
              ? NativeEditorView(
                  document: _document!,
                  viewportBuilder: _texture == null
                      ? null
                      : (context, document, page, viewport, cpuFallback) =>
                            NativeTextureView(
                              controller: _texture!,
                              viewport: viewport,
                              page: page,
                              cpuFallback: cpuFallback,
                            ),
                )
              : NativeDocumentView(document: _document!),
        ),
      ],
    ),
  );

  @override
  void dispose() {
    _path.dispose();
    final document = _document;
    unawaited(_closeDocument(document, _texture));
    super.dispose();
  }
}
