// Genuine Windows consumer entry point; no mocked registrar or DPR override.
import 'package:flutter/material.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'linux_tile_group_probe.dart' show TileProbe;

void main() => runApp(
  const MaterialApp(home: TileProbe(platform: NativeTexturePlatform.windows)),
);
