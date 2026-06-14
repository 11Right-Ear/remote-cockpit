/// Login screen: gateway URL / token / device id, then connect.

library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/cockpit_client.dart';
import '../state/connection_state.dart';
import 'terminal_screen.dart';

class LoginScreen extends StatefulWidget {
  const LoginScreen({super.key});

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _urlCtl = TextEditingController(text: 'wss://127.0.0.1:8443/ws');
  final _tokenCtl = TextEditingController();
  final _deviceCtl = TextEditingController(text: 'dev-ws');
  bool _busy = false;

  @override
  void dispose() {
    _urlCtl.dispose();
    _tokenCtl.dispose();
    _deviceCtl.dispose();
    super.dispose();
  }

  Future<void> _connect() async {
    if (_tokenCtl.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Token is required')),
      );
      return;
    }
    setState(() => _busy = true);
    final client = context.read<CockpitClient>();
    final conn = context.read<CockpitConnectionState>();
    conn.connecting();
    try {
      await client.connect(_urlCtl.text.trim(), _tokenCtl.text.trim(),
          _deviceCtl.text.trim());
      conn.connected(client.ptySessionId ?? '');
      if (!mounted) return;
      await Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const TerminalScreen()),
      );
    } catch (e) {
      conn.failed(e.toString());
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Connect failed: $e')),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Remote Cockpit — Connect')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextField(
                  controller: _urlCtl,
                  decoration: const InputDecoration(
                    labelText: 'Gateway URL',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _tokenCtl,
                  decoration: const InputDecoration(
                    labelText: 'Token (JWT)',
                    helperText: 'phone-sim gen-token --device-id dev-phone '
                        '--role phone --secret dev-secret',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _deviceCtl,
                  decoration: const InputDecoration(
                    labelText: 'Device ID',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 24),
                FilledButton.icon(
                  onPressed: _busy ? null : _connect,
                  icon: _busy
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.power),
                  label: Text(_busy ? 'Connecting…' : 'Connect'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
