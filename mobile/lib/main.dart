/// remote-cockpit Android/desktop client.

library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'screens/login_screen.dart';
import 'services/cockpit_client.dart';
import 'state/connection_state.dart';
import 'state/files_browser_state.dart';

/// Dev-only: accept the gateway's self-signed TLS certificate.
/// (Phase 2 should replace this with a real CA / pinned cert.)
class DevHttpOverrides extends HttpOverrides {
  @override
  HttpClient createHttpClient(SecurityContext? context) {
    return super.createHttpClient(context)
      ..badCertificateCallback = (cert, host, port) => true;
  }
}

void main() {
  HttpOverrides.global = DevHttpOverrides();
  runApp(const RemoteCockpitApp());
}

class RemoteCockpitApp extends StatelessWidget {
  const RemoteCockpitApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        Provider<CockpitClient>(create: (_) => CockpitClient()),
        ChangeNotifierProvider<CockpitConnectionState>(
          create: (_) => CockpitConnectionState(),
        ),
        ChangeNotifierProvider<FilesBrowserState>(
          create: (_) => FilesBrowserState(),
        ),
      ],
      child: MaterialApp(
        title: 'Remote Cockpit',
        debugShowCheckedModeBanner: false,
        theme: ThemeData.dark(useMaterial3: true),
        home: const LoginScreen(),
      ),
    );
  }
}
