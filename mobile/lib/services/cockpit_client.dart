/// WebSocket client to the gateway.
///
/// Connects over `wss://`, authenticates, opens a PTY session, and exposes:
/// - [messages]: inbound control-plane [ServerMessage]s
/// - [terminalBytes]: raw PTY output bytes (binary frames, tag 0x01)
/// - [sendInput]: user input -> `terminal_input` (base64)
///
/// V1 dev TLS (accepting the gateway's self-signed cert) is configured in
/// `main.dart` via [HttpOverrides].

library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:web_socket_channel/io.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import '../protocol/frame.dart';
import '../protocol/messages.dart';

class CockpitClient {
  WebSocketChannel? _channel;
  StreamSubscription? _sub;
  String? _ptySessionId; // PTY session id (from session_opened)

  final StreamController<ServerMessage> _messages =
      StreamController<ServerMessage>.broadcast();
  final StreamController<Uint8List> _terminalBytes =
      StreamController<Uint8List>.broadcast();

  /// Inbound control-plane messages.
  Stream<ServerMessage> get messages => _messages.stream;

  /// PTY output bytes (terminal stdout).
  Stream<Uint8List> get terminalBytes => _terminalBytes.stream;

  /// The PTY session id, once [session_opened] arrives.
  String? get ptySessionId => _ptySessionId;

  /// Connect, authenticate, and open a PTY session.
  ///
  /// Completes once `session_opened` arrives. Throws on `auth_fail`,
  /// unexpected close, or stream error.
  Future<void> connect(String url, String token, String deviceId) async {
    final ready = Completer<void>();
    _channel = IOWebSocketChannel.connect(Uri.parse(url));

    // Send Auth as the first frame.
    _send(AuthMessage(token: token, role: 'phone', deviceId: deviceId));

    var authed = false;
    _sub = _channel!.stream.listen(
      (event) {
        if (event is String) {
          final msg = ServerMessage.fromJson(
              jsonDecode(event) as Map<String, dynamic>);
          if (!authed) {
            if (msg is AuthOkMessage) {
              authed = true;
              _send(const OpenSessionMessage(shell: null));
            } else if (msg is AuthFailMessage) {
              if (!ready.isCompleted) {
                ready.completeError(Exception('auth failed: ${msg.reason}'));
              }
            }
            _messages.add(msg);
            return;
          }
          if (msg is SessionOpenedMessage && !ready.isCompleted) {
            _ptySessionId = msg.sessionId;
            ready.complete();
          }
          _messages.add(msg);
        } else if (event is List<int>) {
          final frame = decodeFrame(Uint8List.fromList(event));
          if (frame != null && frame.tag == terminalTag) {
            _terminalBytes.add(frame.payload);
          }
        }
      },
      onError: (Object e) {
        if (!ready.isCompleted) ready.completeError(e);
      },
      onDone: () {
        if (!ready.isCompleted) {
          ready.completeError(Exception('connection closed during handshake'));
        }
      },
      cancelOnError: true,
    );

    await ready.future;
  }

  /// Send user input bytes to the PTY as `terminal_input` (base64).
  void sendInput(List<int> bytes) {
    if (_ptySessionId == null || _channel == null) return;
    _send(TerminalInputMessage(
      sessionId: _ptySessionId!,
      data: base64Encode(bytes),
    ));
  }

  /// Request a read-only directory listing (Phase 2 file browser). The
  /// response arrives as a [DirListResponse] on [messages], correlated by
  /// `requestId`.
  void requestListDir(String requestId, String path) {
    _send(ListDirRequest(requestId: requestId, path: path));
  }

  /// Request a read-only file's content (Phase 2 file browser). The response
  /// arrives as a [FileContentResponse] on [messages], correlated by
  /// `requestId`.
  void requestReadFile(String requestId, String path) {
    _send(ReadFileRequest(requestId: requestId, path: path));
  }

  /// Request a read-only image preview. The response arrives as an
  /// [ImageContentResponse] on [messages], correlated by `requestId`.
  void requestReadImage(String requestId, String path) {
    _send(ReadImageRequest(requestId: requestId, path: path));
  }

  /// Close the connection.
  Future<void> close() async {
    await _sub?.cancel();
    await _channel?.sink.close();
    _ptySessionId = null;
  }

  void _send(ClientMessage msg) {
    _channel?.sink.add(jsonEncode(msg.toJson()));
  }
}
