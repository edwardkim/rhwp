// Actual WindowServer/Core Animation capture. No widget/producer screenshot fallback.
import AppKit
import CoreGraphics
import CoreVideo
import Foundation
import ImageIO
import ScreenCaptureKit

@main
struct CaptureMetalWindow {
  // AppKit's documented startup creates NSApp before using its other services.
  // A CLI async entry point does not perform NSApplicationMain's initialization.
  // Prohibited policy prevents finishLaunching from activating this helper.
  @MainActor
  private static func initializeAppKit() throws {
    guard Thread.isMainThread else {
      throw NSError(domain: "rhwp-capture", code: 10,
                    userInfo: [NSLocalizedDescriptionKey: "AppKit capture initialization requires the main thread"])
    }
    let application = NSApplication.shared
    _ = application.setActivationPolicy(.prohibited)
    guard application.activationPolicy() == .prohibited else {
      throw NSError(domain: "rhwp-capture", code: 11,
                    userInfo: [NSLocalizedDescriptionKey: "capture helper could not prohibit foreground activation"])
    }
    application.finishLaunching()
    guard application.activationPolicy() == .prohibited, !application.isActive else {
      throw NSError(domain: "rhwp-capture", code: 12,
                    userInfo: [NSLocalizedDescriptionKey: "capture helper unexpectedly became active"])
    }
  }

  @MainActor
  static func main() async {
    do {
      let args = CommandLine.arguments
      guard args.count == 4, let pid = Int32(args[1]), pid > 0,
            let dpr = Double(args[2]), dpr.isFinite, dpr > 0, dpr <= 16 else {
        throw NSError(domain: "rhwp-capture", code: 1,
                      userInfo: [NSLocalizedDescriptionKey: "expected PID DPR OUTPUT.png"])
      }
      try initializeAppKit()
      guard CGPreflightScreenCaptureAccess() else {
        throw NSError(domain: "rhwp-capture", code: 2,
                      userInfo: [NSLocalizedDescriptionKey: "Screen Recording permission is required; no TCC bypass or mock fallback"])
      }
      let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
      let windows = content.windows.filter {
        $0.owningApplication?.processID == pid && $0.isOnScreen && $0.windowLayer == 0 &&
        $0.frame.width > 0 && $0.frame.height > 0
      }
      guard windows.count == 1 else {
        throw NSError(domain: "rhwp-capture", code: 3,
                      userInfo: [NSLocalizedDescriptionKey: "expected exactly one visible qualification window"])
      }
      let window = windows[0]
      guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else {
        throw NSError(domain: "rhwp-capture", code: 4,
                      userInfo: [NSLocalizedDescriptionKey: "qualification window is not the foreground application"])
      }
      let width = Int(ceil(window.frame.width * dpr)), height = Int(ceil(window.frame.height * dpr))
      guard width > 0, height > 0, width <= 4096, height <= 4096, width * height <= 4_194_304 else {
        throw NSError(domain: "rhwp-capture", code: 5,
                      userInfo: [NSLocalizedDescriptionKey: "capture exceeds the physical pixel bound"])
      }
      let configuration = SCStreamConfiguration()
      configuration.width = width
      configuration.height = height
      configuration.showsCursor = false
      configuration.scalesToFit = false
      configuration.pixelFormat = kCVPixelFormatType_32BGRA
      configuration.colorSpaceName = CGColorSpace.sRGB
      if #available(macOS 15.0, *) { configuration.captureDynamicRange = .SDR }
      let filter = SCContentFilter(desktopIndependentWindow: window)
      let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: configuration)
      guard image.width == width, image.height == height,
            let color = CGColorSpace(name: CGColorSpace.sRGB),
            let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8,
                                    bytesPerRow: width * 4, space: color,
                                    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
        throw NSError(domain: "rhwp-capture", code: 6)
      }
      let output = URL(fileURLWithPath: args[3])
      guard args[3].hasPrefix("/"), !FileManager.default.fileExists(atPath: output.path) else {
        throw NSError(domain: "rhwp-capture", code: 9,
                      userInfo: [NSLocalizedDescriptionKey: "output must be a new absolute path"])
      }
      context.interpolationQuality = .none
      context.setBlendMode(.copy)
      context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
      guard let normalized = context.makeImage(),
            let destination = CGImageDestinationCreateWithURL(output as CFURL, "public.png" as CFString, 1, nil) else {
        throw NSError(domain: "rhwp-capture", code: 7)
      }
      CGImageDestinationAddImage(destination, normalized, nil)
      guard CGImageDestinationFinalize(destination) else { throw NSError(domain: "rhwp-capture", code: 8) }
      try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: output.path)
      let result: [String: Any] = ["pid": pid, "windowId": window.windowID, "width": width, "height": height,
                                  "windowBoundsPoints": [window.frame.origin.x, window.frame.origin.y,
                                                         window.frame.width, window.frame.height],
                                  "deviceScale": dpr, "actualWindowServerCapture": true,
                                  "colorSpace": "srgb", "dynamicRange": "sdr",
                                  "appKitInitialized": true, "captureActivationPolicy": "prohibited"]
      let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
      print(String(decoding: data, as: UTF8.self))
    } catch {
      fputs("Actual macOS compositor capture failed: \(error)\n", stderr)
      exit(2)
    }
  }
}
