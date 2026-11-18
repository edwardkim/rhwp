Pod::Spec.new do |s|
  s.name             = 'rhwp_native_reference'
  s.version          = '0.1.0'
  s.summary          = 'Bounded experimental rhwp Metal texture producer.'
  s.description      = 'Optional native Ganesh producer with immutable CVPixelBuffer frames and explicit consumer retirement backpressure.'
  s.homepage         = 'https://github.com/edwardkim/rhwp'
  s.license          = { :type => 'MIT', :file => '../../../LICENSE' }
  s.author           = { 'rhwp contributors' => 'https://github.com/edwardkim/rhwp' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.public_header_files = 'Classes/RhwpMetalPlugin.h'
  s.dependency 'FlutterMacOS'
  # FlutterMacOS is a placeholder pod; ObjC++ needs an explicit framework link
  # even when the host also obtains the Flutter engine through SwiftPM.
  s.frameworks       = 'FlutterMacOS', 'Metal', 'CoreVideo', 'CoreGraphics', 'IOSurface', 'QuartzCore'
  s.platform         = :osx, '12.0'
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17',
    'CLANG_ENABLE_OBJC_ARC' => 'YES',
  }
end
