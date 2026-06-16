/// Terminal screen: an xterm.dart terminal wired to the cockpit client.

library;

// Event streams (xterm/WS) fire on the event loop; every BuildContext use is
// guarded by a mounted check. flutter_lints still flags this — the proper
// refactor (events via ChangeNotifier) is deferred beyond V1.
// ignore_for_file: use_build_context_synchronously

import 'dart:async';
import 'dart:convert';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:xterm/xterm.dart';

import '../protocol/messages.dart';
import '../services/cockpit_client.dart';

class TerminalScreen extends StatefulWidget {
  const TerminalScreen({super.key});

  @override
  State<TerminalScreen> createState() => _TerminalScreenState();
}

class _TerminalScreenState extends State<TerminalScreen> {
  late final Terminal _terminal;
  StreamSubscription? _bytesSub;
  StreamSubscription? _msgSub;

  // Android soft-keyboard bridge. xterm.dart 4.0's TerminalView only injects
  // Enter when the IME reports TextInputAction.done, which most Android IMEs
  // (vivo etc.) don't emit for the default emailAddress keyboard — so Enter was
  // silently dropped while letters worked. Fix: disable TerminalView's IME
  // entirely (hardwareKeyboardOnly: true) and route all soft-keyboard input
  // through this bottom field, forwarding chars via terminal.textInput and
  // Enter via terminal.keyInput(TerminalKey.enter). Desktop keeps using its
  // hardware keyboard and shows no field.
  final TextEditingController _inputController = TextEditingController();
  final FocusNode _inputFocus = FocusNode();
  String _lastInput = '';

  @override
  void initState() {
    super.initState();
    final client = context.read<CockpitClient>();
    _terminal = Terminal(maxLines: 5000);
    _terminal.onOutput = (data) => client.sendInput(utf8.encode(data));

    _bytesSub = client.terminalBytes.listen((bytes) {
      // PTY output bytes -> terminal (allowMalformed tolerates non-UTF8 bytes).
      _terminal.write(utf8.decode(bytes, allowMalformed: true));
    });

    _msgSub = client.messages.listen((msg) {
      if (msg is DangerWarnMessage) {
        if (!context.mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('DANGER: ${msg.command}\n(pattern: ${msg.pattern})'),
            backgroundColor: Colors.red,
            duration: const Duration(seconds: 5),
          ),
        );
      } else if (msg is SessionClosedMessage || msg is DesktopOfflineMessage) {
        if (!context.mounted) return;
        Navigator.of(context).maybePop();
      }
    });
  }

  @override
  void dispose() {
    _bytesSub?.cancel();
    _msgSub?.cancel();
    _inputController.dispose();
    _inputFocus.dispose();
    super.dispose();
  }

  void _onInputChanged(String value) {
    if (value.length > _lastInput.length) {
      // Characters appended -> forward to the PTY (cmd echoes them back).
      _terminal.textInput(value.substring(_lastInput.length));
    } else if (value.length < _lastInput.length) {
      // Characters removed -> emit backspaces so the PTY line tracks the field.
      for (var i = 0; i < _lastInput.length - value.length; i++) {
        _terminal.keyInput(TerminalKey.backspace);
      }
    }
    _lastInput = value;
  }

  void _onInputSubmitted(String value) {
    _terminal.keyInput(TerminalKey.enter);
    _inputController.clear();
    _lastInput = '';
    _inputFocus.requestFocus(); // keep the soft keyboard open for the next command
  }

  @override
  Widget build(BuildContext context) {
    final showBridge = Platform.isAndroid;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Terminal'),
        leading: IconButton(
          icon: const Icon(Icons.logout),
          tooltip: 'Disconnect',
          onPressed: () async {
            await context.read<CockpitClient>().close();
            if (context.mounted) Navigator.of(context).pop();
          },
        ),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: TerminalView(
                _terminal,
                // Desktop: hardware keyboard (bypasses the Flutter 3.44
                // IME/TextInput bug). Android: also true — the phone's soft
                // keyboard is bridged via the input field below, because
                // xterm.dart 4.0 drops the Enter key for non-`done` IME actions.
                hardwareKeyboardOnly: true,
                autofocus: !showBridge,
                backgroundOpacity: 1,
                theme: TerminalTheme(
                  foreground: Colors.white,
                  background: const Color(0xFF1E1E1E),
                  cursor: Colors.white,
                  selection: const Color(0x55FFFFFF),
                  // ANSI palette (xterm)
                  black: Colors.black,
                  red: Colors.red,
                  green: Colors.green,
                  yellow: Colors.yellow,
                  blue: Colors.blue,
                  magenta: Colors.purple,
                  cyan: Colors.cyan,
                  white: Colors.white,
                  brightBlack: Colors.grey,
                  brightRed: Colors.redAccent,
                  brightGreen: Colors.greenAccent,
                  brightYellow: Colors.yellowAccent,
                  brightBlue: Colors.blueAccent,
                  brightMagenta: Colors.purpleAccent,
                  brightCyan: Colors.cyanAccent,
                  brightWhite: Colors.white,
                  searchHitBackground: Colors.yellow,
                  searchHitBackgroundCurrent: Colors.orange,
                  searchHitForeground: Colors.black,
                ),
                // Hardware keyboard input flows to terminal.onOutput via the widget.
              ),
            ),
            if (showBridge)
              Container(
                color: const Color(0xFF2A2A2A),
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: TextField(
                  controller: _inputController,
                  focusNode: _inputFocus,
                  autofocus: true,
                  maxLines: 1,
                  textInputAction: TextInputAction.go,
                  style: const TextStyle(
                    color: Colors.white,
                    fontFamily: 'monospace',
                    fontSize: 16,
                  ),
                  decoration: const InputDecoration(
                    hintText: '输入命令，回车发送',
                    hintStyle: TextStyle(color: Colors.white54),
                    border: InputBorder.none,
                  ),
                  onChanged: _onInputChanged,
                  onSubmitted: _onInputSubmitted,
                ),
              ),
          ],
        ),
      ),
    );
  }
}
