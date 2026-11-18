# 분할 원본 프로그램의 준비 결과와 CanvasKit 소비 정합

앞 단계에서 같은 family/style의 분할 파일들을 준비 목록에 보존한 뒤 실제 바이트가 모두 읽히는 것을 확인했다. 그러나 `resolvePreparedTypeface`는 다시 단일 resolver를 사용해 준비 결과를 찾지 못했다. 실제 1쪽 수식 대조군은 한양신명조 준비 실패를 보고했고 Canvas2D로 fallback했다. 이를 CanvasKit 통과로 수용하지 않는다.

관계형 반례는 실제 renderer의 `prepareHostFonts`를 거쳐 Latin·Hangul·명시 fallback 프로그램을 준비한다. 원본 family가 두 프로그램이라는 이유로 미준비로 판정되던 조건에서 보정 전 3 PASS·1 FAIL을 확인했다. 보정은 준비를 마친 같은 family/style 후보들을 기존 순서로 조회하며, 일반 글리프·겹친 글자·문단 shaping·가용성 판단이 같은 후보들을 소비하게 한다. 실제 glyph coverage에서 앞 원본이 가진 문자를 유지하고 결측만 뒤 분할 프로그램·선언 fallback으로 넘긴다.

명시 PostScript 한 프로그램 선택, 중복 PostScript 모호성 거절, 없는 family 미준비 판정, 기존 물리 스타일 선택은 유지한다. 새 반례는 OS 글꼴이나 실물 좌표에 의존하지 않는다. 관련 집중 38 PASS, 최소 글꼴 전체 Studio 2,811 PASS·0 FAIL, TypeScript·Vite build PASS다. 실제 CanvasKit과 대표 Studio 소비자의 보정 후 캡처는 이 커밋을 기준으로 별도 완료해야 한다.

Rust 조판·윤곽·WASM producer, 독립 PDF·판정 문턱과 renderer guard는 바꾸지 않는다. 최종 Rust·정본 Visual Sweep 재사용은 변경 전후 의존 Git 객체와 실행 파일 SHA를 확인한 별도 감사에 근거한다.
