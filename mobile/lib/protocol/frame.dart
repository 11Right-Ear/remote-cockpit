/// Binary-frame tagging, Dart mirror of Rust `protocol::frame`.
///
/// Binary frames are `[1-byte tag][payload bytes...]`. Currently only
/// [terminalTag] (0x01 = PTY stdout) is defined; the switch is written so new
/// streams (files, images) can be added without changing the wire format.

library;

import 'dart:typed_data';

/// First byte of a binary frame: identifies the stream.
const int terminalTag = 0x01;

/// A decoded binary frame.
class Frame {
  final int tag;
  final Uint8List payload;
  const Frame({required this.tag, required this.payload});
}

/// Decode a binary frame into [Frame], or `null` if the frame is empty.
Frame? decodeFrame(Uint8List frame) {
  if (frame.isEmpty) return null;
  return Frame(tag: frame[0], payload: Uint8List.sublistView(frame, 1));
}
