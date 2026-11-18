# 같은 family/style 분할 프로그램의 CanvasKit 준비 누락

## 위반된 계약과 독립 근거

같은 family/style의 서로 다른 원본 프로그램은 임의의 한 개로 확정하지 않는다. 실제 glyph 후보를 모두 준비한 뒤 기존 coverage·style 선택이 소비해야 한다. 기존 `resolveRendererLocalFontCandidates`가 이 계약을 구현하지만, ordinary text의 준비 수집은 단일 resolver를 호출해 복수 프로그램이 있으면 모두 누락한다.

현재 Mac의 원본 한양신명조는 영문·확장·한글 프로그램이 같은 family/style로 설치되어 있다. 실제 OS Local Font Access에도 등록이 확인됐다. 1쪽 수식 대조군에서 원본 legacy 프로그램을 실제 byte provider에 추가한 뒤에도 한양신명조 준비 부족으로 Canvas2D fallback이 유지된다. host provider가 활성화되면 Local Font Access의 중복 준비를 하지 않는 기존 설계이므로 권한 부여만으로 해결되지 않는다.

합성 metadata 반례는 regular의 Latin/Hangul 두 프로그램과 bold 한 프로그램이다. 변경 전 수집 결과는 bold 하나뿐이고, 기대는 Latin/Hangul/bold 세 개다. 단일 resolver가 모호한 regular를 거절하는 동작과 바이트를 읽지 않는 계약은 그대로 유지한다. `before.log`가 실제 1 FAIL을 보존한다.

## 보정 범위

ordinary text의 준비 수집에 이미 구현된 복수 후보 resolver를 사용한다. family/style·명시 PostScript 이름·중복 식별자 거절·후보 순서·glyph 윤곽·IR·PDF·판정 문턱은 바꾸지 않는다. source backend와 Rust/WASM producer는 바꾸지 않는 Studio 준비 경계의 수정이다.

변경 뒤 metadata 반례와 정상 style 선택 검사를 먼저 확인하고, 원본 legacy 수식 대조군의 실제 CanvasKit과 영향 Studio 소비자를 다시 검증한다. 기존 Native/fresh WASM 전쪽 증거는 의존 source가 동일하다는 증거를 확인한 경우에만 재사용한다. 기존 실패 캡처를 CanvasKit 통과로 바꾸지 않는다.
