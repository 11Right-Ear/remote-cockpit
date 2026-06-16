/// Wire protocol for remote-cockpit, Dart mirror of the Rust `protocol` crate.
///
/// Text frames carry a JSON envelope with an outer `type` field and
/// snake_case variant/field names. Unknown variants are tolerated
/// ([UnknownMessage]) so the client never crashes on a new server message.

// ---------------------------------------------------------------------------
// ClientMessage (client -> server)
// ---------------------------------------------------------------------------

library;

abstract class ClientMessage {
  const ClientMessage();
  Map<String, dynamic> toJson();
}

class AuthMessage extends ClientMessage {
  final String token;
  final String role; // "phone"
  final String deviceId;
  const AuthMessage({
    required this.token,
    required this.role,
    required this.deviceId,
  });
  @override
  Map<String, dynamic> toJson() => {
        'type': 'auth',
        'token': token,
        'role': role,
        'device_id': deviceId,
      };
}

class OpenSessionMessage extends ClientMessage {
  final String? shell;
  const OpenSessionMessage({this.shell});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'open_session',
        'shell': shell,
      };
}

class TerminalInputMessage extends ClientMessage {
  final String sessionId;
  final String data; // base64-encoded bytes
  const TerminalInputMessage({required this.sessionId, required this.data});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'terminal_input',
        'session_id': sessionId,
        'data': data,
      };
}

class CloseSessionMessage extends ClientMessage {
  final String sessionId;
  const CloseSessionMessage({required this.sessionId});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'close_session',
        'session_id': sessionId,
      };
}

class ListDirRequest extends ClientMessage {
  final String requestId;
  final String path;
  const ListDirRequest({required this.requestId, required this.path});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'list_dir',
        'request_id': requestId,
        'path': path,
      };
}

class ReadFileRequest extends ClientMessage {
  final String requestId;
  final String path;
  const ReadFileRequest({required this.requestId, required this.path});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'read_file',
        'request_id': requestId,
        'path': path,
      };
}

class ReadImageRequest extends ClientMessage {
  final String requestId;
  final String path;
  const ReadImageRequest({required this.requestId, required this.path});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'read_image',
        'request_id': requestId,
        'path': path,
      };
}

class PingMessage extends ClientMessage {
  final int tsMs;
  const PingMessage({required this.tsMs});
  @override
  Map<String, dynamic> toJson() => {
        'type': 'ping',
        'ts_ms': tsMs,
      };
}

// ---------------------------------------------------------------------------
// ServerMessage (server -> client)
// ---------------------------------------------------------------------------

abstract class ServerMessage {
  const ServerMessage();

  /// Parse a JSON object into a [ServerMessage]. Unknown types become
  /// [UnknownMessage] (never throws).
  static ServerMessage fromJson(Map<String, dynamic> json) {
    final type = json['type'];
    switch (type) {
      case 'auth_ok':
        return AuthOkMessage(
          sessionId: json['session_id'] as String,
          serverTimeMs: json['server_time_ms'] as int,
        );
      case 'auth_fail':
        return AuthFailMessage(reason: json['reason'] as String);
      case 'session_opened':
        return SessionOpenedMessage(sessionId: json['session_id'] as String);
      case 'session_closed':
        return SessionClosedMessage(sessionId: json['session_id'] as String);
      case 'session_error':
        return SessionErrorMessage(
          sessionId: json['session_id'] as String,
          message: json['message'] as String,
        );
      case 'danger_warn':
        return DangerWarnMessage(
          sessionId: json['session_id'] as String,
          command: json['command'] as String,
          pattern: json['pattern'] as String,
        );
      case 'desktop_online':
        return DesktopOnlineMessage(deviceId: json['device_id'] as String);
      case 'desktop_offline':
        return DesktopOfflineMessage(deviceId: json['device_id'] as String);
      case 'pong':
        return PongMessage(
          tsMs: json['ts_ms'] as int,
          serverTimeMs: json['server_time_ms'] as int,
        );
      case 'error':
        return ErrorMessage(
          code: json['code'] as String,
          message: json['message'] as String,
        );
      case 'terminal_output':
        return TerminalOutputMessage(
          sessionId: json['session_id'] as String,
          data: json['data'] as String,
        );
      case 'dir_listing':
        return DirListResponse(
          requestId: json['request_id'] as String,
          path: json['path'] as String,
          entries: (json['entries'] as List<dynamic>)
              .map((e) => FileEntry.fromJson(e as Map<String, dynamic>))
              .toList(),
        );
      case 'file_content':
        return FileContentResponse(
          requestId: json['request_id'] as String,
          path: json['path'] as String,
          content: json['content'] as String?,
          truncated: json['truncated'] as bool? ?? false,
          error: json['error'] as String?,
        );
      case 'image_content':
        return ImageContentResponse(
          requestId: json['request_id'] as String,
          path: json['path'] as String,
          mimeType: json['mime_type'] as String?,
          dataBase64: json['data_base64'] as String?,
          truncated: json['truncated'] as bool? ?? false,
          error: json['error'] as String?,
        );
      default:
        return UnknownMessage(type: type?.toString() ?? '');
    }
  }
}

class AuthOkMessage extends ServerMessage {
  final String sessionId; // connection-scoped id (distinct from PTY session id)
  final int serverTimeMs;
  const AuthOkMessage({required this.sessionId, required this.serverTimeMs});
}

class AuthFailMessage extends ServerMessage {
  final String reason;
  const AuthFailMessage({required this.reason});
}

class SessionOpenedMessage extends ServerMessage {
  final String sessionId; // PTY session id — used in terminal_input etc.
  const SessionOpenedMessage({required this.sessionId});
}

class SessionClosedMessage extends ServerMessage {
  final String sessionId;
  const SessionClosedMessage({required this.sessionId});
}

class SessionErrorMessage extends ServerMessage {
  final String sessionId;
  final String message;
  const SessionErrorMessage({required this.sessionId, required this.message});
}

class DangerWarnMessage extends ServerMessage {
  final String sessionId;
  final String command;
  final String pattern;
  const DangerWarnMessage({
    required this.sessionId,
    required this.command,
    required this.pattern,
  });
}

class DesktopOnlineMessage extends ServerMessage {
  final String deviceId;
  const DesktopOnlineMessage({required this.deviceId});
}

class DesktopOfflineMessage extends ServerMessage {
  final String deviceId;
  const DesktopOfflineMessage({required this.deviceId});
}

class PongMessage extends ServerMessage {
  final int tsMs;
  final int serverTimeMs;
  const PongMessage({required this.tsMs, required this.serverTimeMs});
}

class ErrorMessage extends ServerMessage {
  final String code;
  final String message;
  const ErrorMessage({required this.code, required this.message});
}

class TerminalOutputMessage extends ServerMessage {
  final String sessionId;
  final String data; // base64 (small / out-of-band)
  const TerminalOutputMessage({required this.sessionId, required this.data});
}

class UnknownMessage extends ServerMessage {
  final String type;
  const UnknownMessage({required this.type});
}

/// One entry in a directory listing (Phase 2 file browser). Read-only metadata.
class FileEntry {
  final String name;
  final bool isDir;
  final int size;
  final int modifiedMs;
  const FileEntry({
    required this.name,
    required this.isDir,
    required this.size,
    required this.modifiedMs,
  });

  factory FileEntry.fromJson(Map<String, dynamic> json) => FileEntry(
        name: json['name'] as String,
        isDir: json['is_dir'] as bool,
        size: (json['size'] as num).toInt(),
        modifiedMs: (json['modified_ms'] as num).toInt(),
      );
}

/// Read-only directory listing (response to `list_dir`). Phase 2 file browser.
class DirListResponse extends ServerMessage {
  final String requestId;
  final String path;
  final List<FileEntry> entries;
  const DirListResponse({
    required this.requestId,
    required this.path,
    required this.entries,
  });
}

/// Read-only file content (response to `read_file`). `content` is null on
/// error; `truncated` is true if the size cap was hit.
class FileContentResponse extends ServerMessage {
  final String requestId;
  final String path;
  final String? content;
  final bool truncated;
  final String? error;
  const FileContentResponse({
    required this.requestId,
    required this.path,
    required this.content,
    required this.truncated,
    required this.error,
  });
}

/// Read-only image content (response to `read_image`). `dataBase64` is null on
/// error; `truncated` is true if the size cap was hit.
class ImageContentResponse extends ServerMessage {
  final String requestId;
  final String path;
  final String? mimeType;
  final String? dataBase64;
  final bool truncated;
  final String? error;
  const ImageContentResponse({
    required this.requestId,
    required this.path,
    required this.mimeType,
    required this.dataBase64,
    required this.truncated,
    required this.error,
  });
}
