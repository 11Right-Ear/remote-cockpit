/// Phase 2 image viewer: read-only image preview.

library;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';

import '../protocol/messages.dart';
import '../services/cockpit_client.dart';

class ImageViewerScreen extends StatefulWidget {
  final String path;
  const ImageViewerScreen({super.key, required this.path});

  @override
  State<ImageViewerScreen> createState() => _ImageViewerScreenState();
}

class _ImageViewerScreenState extends State<ImageViewerScreen> {
  static const _uuid = Uuid();
  late final String _requestId;
  StreamSubscription<ServerMessage>? _sub;

  bool _loading = true;
  Uint8List? _bytes;
  String? _mimeType;
  bool _truncated = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _requestId = _uuid.v4();
    final client = context.read<CockpitClient>();
    _sub = client.messages.listen((msg) {
      if (msg is ImageContentResponse && msg.requestId == _requestId) {
        Uint8List? bytes;
        String? error = msg.error;
        if (msg.dataBase64 != null) {
          try {
            bytes = base64Decode(msg.dataBase64!);
          } catch (e) {
            error = 'invalid image payload: $e';
          }
        }
        setState(() {
          _loading = false;
          _bytes = bytes;
          _mimeType = msg.mimeType;
          _truncated = msg.truncated;
          _error = error;
        });
      }
    });
    client.requestReadImage(_requestId, widget.path);
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final name = widget.path.split(RegExp(r'[/\\]')).last;
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          tooltip: 'Back',
          onPressed: () => Navigator.of(context).pop(),
        ),
        title: Text(name, style: const TextStyle(fontSize: 14)),
      ),
      body: _body(),
    );
  }

  Widget _body() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            '无法预览图片：$_error',
            textAlign: TextAlign.center,
            style: const TextStyle(color: Colors.redAccent),
          ),
        ),
      );
    }
    final bytes = _bytes;
    if (bytes == null) {
      return const Center(child: Text('No image data'));
    }
    return Column(
      children: [
        if (_truncated)
          Container(
            width: double.infinity,
            color: const Color(0xFF5D4037),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: const Text(
              '图片过大，仅支持 1 MB 以下预览',
              style: TextStyle(color: Colors.orangeAccent, fontSize: 12),
            ),
          ),
        if (_mimeType != null)
          Container(
            width: double.infinity,
            color: const Color(0xFF263238),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            child: Text(
              _mimeType!,
              style: const TextStyle(color: Colors.white70, fontSize: 12),
            ),
          ),
        Expanded(
          child: InteractiveViewer(
            minScale: 0.5,
            maxScale: 5,
            child: Center(
              child: Image.memory(bytes, gaplessPlayback: true),
            ),
          ),
        ),
      ],
    );
  }
}
