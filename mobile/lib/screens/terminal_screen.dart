/// Terminal screen: an xterm.dart terminal wired to the cockpit client.

library;

// Event streams (xterm/WS) fire on the event loop; every BuildContext use is
// guarded by a mounted check. flutter_lints still flags this — the proper
// refactor (events via ChangeNotifier) is deferred beyond V1.
// ignore_for_file: use_build_context_synchronously

import 'dart:async';
import 'dart:convert';

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
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
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
        child: TerminalView(
          _terminal,
          hardwareKeyboardOnly: true, // bypass IME/TextInput (broken on Flutter 3.44 desktop)
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
    );
  }
}
