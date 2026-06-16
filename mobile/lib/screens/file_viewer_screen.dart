/// Phase 2 file viewer: read-only text file content.

library;

// ignore_for_file: use_build_context_synchronously

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';

import '../protocol/messages.dart';
import '../services/cockpit_client.dart';

class FileViewerScreen extends StatefulWidget {
  final String path;
  const FileViewerScreen({super.key, required this.path});

  @override
  State<FileViewerScreen> createState() => _FileViewerScreenState();
}

class _FileViewerScreenState extends State<FileViewerScreen> {
  static const _uuid = Uuid();
  late final String _requestId;
  StreamSubscription<ServerMessage>? _sub;

  bool _loading = true;
  String? _content;
  bool _truncated = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _requestId = _uuid.v4();
    final client = context.read<CockpitClient>();
    _sub = client.messages.listen((msg) {
      if (msg is FileContentResponse && msg.requestId == _requestId) {
        setState(() {
          _loading = false;
          _content = msg.content;
          _truncated = msg.truncated;
          _error = msg.error;
        });
      }
    });
    client.requestReadFile(_requestId, widget.path);
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
            '无法读取：$_error',
            textAlign: TextAlign.center,
            style: const TextStyle(color: Colors.redAccent),
          ),
        ),
      );
    }
    final content = _content ?? '';
    return Column(
      children: [
        if (_truncated)
          Container(
            width: double.infinity,
            color: const Color(0xFF5D4037),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            child: const Text(
              '文件过大，仅显示前 256 KB',
              style: TextStyle(color: Colors.orangeAccent, fontSize: 12),
            ),
          ),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(8),
            child: SelectableText(
              content,
              style: const TextStyle(
                fontFamily: 'monospace',
                fontSize: 13,
                height: 1.3,
              ),
            ),
          ),
        ),
      ],
    );
  }
}
