/// Connection state for the cockpit UI.

library;

import 'package:flutter/foundation.dart';

enum ConnStatus { disconnected, connecting, connected, error }

/// Named `Cockpit*` to avoid clashing with `dart:async`'s `ConnectionState`.
class CockpitConnectionState extends ChangeNotifier {
  ConnStatus _status = ConnStatus.disconnected;
  String? _error;
  String? _ptySessionId;

  ConnStatus get status => _status;
  String? get error => _error;
  String? get ptySessionId => _ptySessionId;

  void connecting() {
    _status = ConnStatus.connecting;
    _error = null;
    notifyListeners();
  }

  void connected(String ptySessionId) {
    _status = ConnStatus.connected;
    _ptySessionId = ptySessionId;
    _error = null;
    notifyListeners();
  }

  void failed(String message) {
    _status = ConnStatus.error;
    _error = message;
    notifyListeners();
  }

  void reset() {
    _status = ConnStatus.disconnected;
    _error = null;
    _ptySessionId = null;
    notifyListeners();
  }
}
