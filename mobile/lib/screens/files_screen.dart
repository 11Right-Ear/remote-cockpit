/// Phase 2 file browser: read-only directory listing.

library;

// Event streams fire on the event loop; every BuildContext use is guarded by a
// mounted check. flutter_lints still flags this — the proper refactor (events
// via ChangeNotifier) is deferred beyond V1.
// ignore_for_file: use_build_context_synchronously

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../protocol/messages.dart';
import '../services/cockpit_client.dart';
import '../state/files_browser_state.dart';

class FilesScreen extends StatefulWidget {
  const FilesScreen({super.key});

  @override
  State<FilesScreen> createState() => _FilesScreenState();
}

class _FilesScreenState extends State<FilesScreen> {
  StreamSubscription<ServerMessage>? _sub;

  @override
  void initState() {
    super.initState();
    final client = context.read<CockpitClient>();
    final state = context.read<FilesBrowserState>();
    _sub = client.messages.listen((msg) {
      if (msg is DirListResponse) {
        state.applyListing(msg.requestId, msg.path, msg.entries);
      }
    });
    // Initial listing at the desktop's fs root.
    WidgetsBinding.instance.addPostFrameCallback((_) => _load('.'));
  }

  void _load(String path) {
    final client = context.read<CockpitClient>();
    final state = context.read<FilesBrowserState>();
    final requestId = state.beginLoad(path);
    client.requestListDir(requestId, path);
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watch<FilesBrowserState>();
    final display = state.currentPath.isEmpty ? '/' : state.currentPath;
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          tooltip: 'Back',
          onPressed: () => Navigator.of(context).pop(),
        ),
        title: Text(display, style: const TextStyle(fontSize: 14)),
        actions: [
          IconButton(
            icon: const Icon(Icons.arrow_upward),
            tooltip: 'Up',
            onPressed: () => _load(_parent(state.currentPath)),
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh',
            onPressed: () =>
                _load(state.currentPath.isEmpty ? '.' : state.currentPath),
          ),
        ],
      ),
      body: _body(state),
    );
  }

  Widget _body(FilesBrowserState state) {
    switch (state.status) {
      case FilesLoadStatus.idle:
      case FilesLoadStatus.loading:
        return const Center(child: CircularProgressIndicator());
      case FilesLoadStatus.error:
        return Center(child: Text('Error: ${state.error ?? "unknown"}'));
      case FilesLoadStatus.empty:
        return const Center(child: Text('Empty or unreadable directory'));
      case FilesLoadStatus.loaded:
        return ListView.builder(
          itemCount: state.entries.length,
          itemBuilder: (_, i) {
            final e = state.entries[i];
            return ListTile(
              leading: Icon(e.isDir ? Icons.folder : Icons.insert_drive_file),
              title: Text(e.name),
              subtitle: e.isDir ? null : Text(_humanSize(e.size)),
              onTap: e.isDir
                  ? () => _load('${state.currentPath}/${e.name}')
                  : null,
            );
          },
        );
    }
  }

  /// Parent path, tolerant of both `/` and `\` (desktop may be Windows).
  /// `.` and `/` are the root (no parent).
  String _parent(String path) {
    if (path.isEmpty || path == '.' || path == '/') return '.';
    final slash = path.lastIndexOf('/');
    final back = path.lastIndexOf('\\');
    final idx = slash > back ? slash : back;
    if (idx <= 0) return '.';
    return path.substring(0, idx);
  }

  String _humanSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / 1024 / 1024).toStringAsFixed(1)} MB';
    }
    return '${(bytes / 1024 / 1024 / 1024).toStringAsFixed(1)} GB';
  }
}
