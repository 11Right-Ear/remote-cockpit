/// State for the Phase 2 file browser screen.

library;

import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import '../protocol/messages.dart';

enum FilesLoadStatus { idle, loading, loaded, empty, error }

class FilesBrowserState extends ChangeNotifier {
  static const _uuid = Uuid();

  FilesLoadStatus _status = FilesLoadStatus.idle;
  String? _error;
  String _currentPath = '';
  List<FileEntry> _entries = const [];

  /// The request_id of the in-flight `list_dir`, so stale responses (from an
  /// older path after the user navigated) are ignored.
  String? _pendingRequestId;

  FilesLoadStatus get status => _status;
  String? get error => _error;
  String get currentPath => _currentPath;
  List<FileEntry> get entries => _entries;

  /// Begin loading `path`. Returns the request_id the caller should send and
  /// watch for on the response.
  String beginLoad(String path) {
    _status = FilesLoadStatus.loading;
    _error = null;
    _currentPath = path;
    _pendingRequestId = _uuid.v4();
    notifyListeners();
    return _pendingRequestId!;
  }

  /// Apply a listing response, but only if it matches the pending request.
  void applyListing(String requestId, String path, List<FileEntry> entries) {
    if (requestId != _pendingRequestId) return;
    _currentPath = path;
    _entries = entries;
    _status = entries.isEmpty ? FilesLoadStatus.empty : FilesLoadStatus.loaded;
    _pendingRequestId = null;
    notifyListeners();
  }

  void fail(String message) {
    _status = FilesLoadStatus.error;
    _error = message;
    _pendingRequestId = null;
    notifyListeners();
  }
}
