import 'dart:async';

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:ur_registry_flutter/native_object.dart';
import 'package:ur_registry_flutter/ur_decoder.dart';

abstract class _State {}

class _InitialState extends _State {}

typedef SuccessCallback = void Function(NativeObject);
typedef FailureCallback = void Function(String);

class _Cubit extends Cubit<_State> {
  late final SupportedType target;
  final SuccessCallback onSuccess;
  final FailureCallback onFailed;
  final Widget? overlay;
  URDecoder urDecoder = URDecoder();
  bool succeed = false;

  _Cubit(
    this.target,
    this.onSuccess,
    this.onFailed, {
    this.overlay,
  }) : super(_InitialState());

  void receiveQRCode(String? code) {
    try {
      if (code != null) {
        urDecoder.receive(code);
        if (urDecoder.isComplete()) {
          final result = urDecoder.resolve(target);
          if (!succeed) {
            onSuccess(result);
            succeed = true;
          }
        }
      }
    } catch (e) {
      onFailed("Error when receiving UR $e");
      reset();
    }
  }

  void reset() {
    urDecoder = URDecoder();
    succeed = false;
  }
}

class AnimatedQRScanner extends StatelessWidget {
  final SupportedType target;
  final SuccessCallback onSuccess;
  final FailureCallback onFailed;
  final Widget? overlay;

  const AnimatedQRScanner({Key? key, required this.target, required this.onSuccess, required this.onFailed, this.overlay})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (BuildContext context) => _Cubit(target, onSuccess, onFailed, overlay: overlay),
      child: _AnimatedQRScanner(),
    );
  }
}

class _AnimatedQRScanner extends StatefulWidget {
  @override
  _AnimatedQRScannerState createState() => _AnimatedQRScannerState();
}

class _AnimatedQRScannerState extends State<_AnimatedQRScanner> with WidgetsBindingObserver {
  final MobileScannerController controller = MobileScannerController(detectionSpeed: DetectionSpeed.noDuplicates);
  late StreamSubscription<Object?>? _subscription;
  late final _Cubit _cubit;

  @override
  void initState() {
    WidgetsBinding.instance.addObserver(this);
    _cubit = BlocProvider.of(context);
    _subscription = controller.barcodes.listen(_handleBarcode);
    unawaited(controller.start());
    super.initState();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    unawaited(_subscription?.cancel());
    _subscription = null;
    controller.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    super.didChangeAppLifecycleState(state);

    switch (state) {
      case AppLifecycleState.detached:
      case AppLifecycleState.hidden:
      case AppLifecycleState.paused:
        return;
      case AppLifecycleState.resumed:
        // Restart the scanner when the app is resumed.
        // Don't forget to resume listening to the barcode events.
        _subscription = controller.barcodes.listen(_handleBarcode);
        unawaited(controller.start());
        return;
      case AppLifecycleState.inactive:
        unawaited(_subscription?.cancel());
        _subscription = null;
        unawaited(controller.stop());
        return;
    }
  }

  @override
  Widget build(BuildContext context) {
    return MobileScanner(
      controller: controller,
      overlayBuilder: _cubit.overlay != null ? (context, constraints) => _cubit.overlay! : null,
    );
  }

  void _handleBarcode(BarcodeCapture capture) {
    for (final barcode in capture.barcodes) {
      _cubit.receiveQRCode(barcode.rawValue);
    }
  }
}
