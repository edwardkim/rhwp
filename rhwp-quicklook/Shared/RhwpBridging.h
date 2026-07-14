// [Task #2267] Swift ← C ABI 브릿징.
//
// RhwpNative.xcframework 는 정적 라이브러리 + C 헤더만 담고 modulemap 이 없으므로,
// bridging header 로 노출한다. 헤더 경로는 HEADER_SEARCH_PATHS 로 지정한다.

#import "rhwp_native_ffi.h"
