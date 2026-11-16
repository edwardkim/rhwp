# Print의 확인된 일반 글리프 프로그램과 기하

## 실제 원인과 독립 대조

새 보정본의 중첩 표 Native 전 17쪽은 보호값을 지키지만, Linux의 fresh WASM은
2·10·11·14·15쪽이 정본보다 0.01~0.16%p 낮다. 10·11·14·15쪽의 큰 잔여 성분은
Gulim 본문 글자의 오른쪽 획이다. 2쪽의 19픽셀은 주로 동그라미 두 개다.

제품을 수정하지 않은 exporter 사본의 같은 입력·글꼴 목록·WASM·PDF로 대조했다.
대조군 5쪽 PNG의 SHA는 실제 최종 exporter 출력과 모두 같다. 실제 적재한 동일한
Gulim SFNT의 무힌팅 윤곽만 기존 Canvas 원점·변환에서 재생하면 다음 결과를 얻는다.
위치·크기·전진·글꼴·기준 PDF·판정법은 바꾸지 않았다.

| 쪽 | 원 출력 | 같은 프로그램의 윤곽 재생 | 보호값 |
| --- | ---: | ---: | ---: |
| 2 | 99.96743% | 99.96767% | 99.98% |
| 10 | 98.60465% | 98.82747% | 98.76% |
| 11 | 98.93157% | 99.09788% | 99.05% |
| 14 | 98.47133% | 98.70176% | 98.59% |
| 15 | 99.07188% | 99.19472% | 99.12% |

Gulim 4쪽의 차이는 공급 글꼴 교체 없이 Print의 실제 glyph paint에서 복구된다.
2쪽의 `○`는 `HY Sinmyeongjo`로 요청되지만 공급된 CFF 조각 5개 모두에 그 문자가 없다.
이 별도 fallback도 확인된 실제 프로그램만 소비해야 한다. 이를 아직 해결됐다고 쓰지 않는다.

## 보정할 소비 경계

현재 `print_fallback_glyph`는 원본 TTF 선언의 결측 glyph만 처리한다. 원본 유형이 HFT·
미정의인 경우와 실제 주 프로그램의 일반 glyph는 Canvas의 호스트별 힌팅으로 남는다.
`geometricPrecision` 힌트만으로는 같은 SFNT의 기하를 보장하지 못했다.

실제 FontFace의 로드와 우선순위를 소유한 호스트가 **주 프로그램과 family의 완전성**을
명시적으로 등록한다. Print는 그 주 프로그램이 갖는 glyph만 기존 원점에 무힌팅으로
재생한다. 주 프로그램이 글자를 갖지 않고 같은 family의 뒤 후보가 미확인이라면
기존 Canvas painter에 맡긴다. 모든 CFF 조각이 확인된 family가 글자를 갖지 않을 때만
다음 family로 진행한다. 빈 glyph·미등록 앞 family·가변·기울임·컬러 프로그램·문맥 shaping은
새 기하 경로로 강제하지 않는다. Screen·조판 폭·charX·저장 정보는 유지한다.

Sweep뿐 아니라 Studio의 실제 페이지별 host FontFace 공급도 같은 등록과 수명 경계를 쓴다.
Studio가 정확한 프로그램으로 해소하지 못하는 모호한 family는 추측하지 않는다.
제품의 글꼴·문서·쪽 번호 조건이나 측정 점수별 보정을 추가하지 않는다.

## 검증 계획과 현재 상태

공개 합성 TTF로 주 프로그램·불완전한 앞 후보·완전한 CFF 결측·Screen 보존을 실제
WASM/Canvas에서 검사한다. 기존 fallback·분리 CFF·가짜 굵기 계약도 그대로 실행한다.
관련 공통 코드를 보정한 뒤 새 최적화 WASM·실물 17쪽과 보호 범위를 확인한다.
진단 사본의 향상을 제품 검증 완료나 보호 기준 갱신으로 승격하지 않는다.


## 공급 경계의 사전 검증

- 기존 최적화 WASM에 새 반례를 실행하면 주 TTF 재생과 완전한 CFF 결측 두 검사가 실패하고,
  미확인 동반 face 보존 검사는 통과한다. [수정 전 브라우저 로그](verification-record.md).
- Studio의 실제 descriptor 순서·페이지별 수명·예외 후 해제·제한된 unicode-range를 포함한
  12개 검사가 통과했다. [Studio 검사](verification-record.md).
- 실제 Chrome FontFace 공급 검사 2개와 Studio TypeScript 검사가 통과했다.
  [Chrome 등록 검사](verification-record.md), [TypeScript 검사](verification-record.md).
- wasm32 공통 API·소유 경로의 `cargo check --locked --target-dir target/pr-review
  --target wasm32-unknown-unknown --lib`가 통과했다. [컴파일 로그](verification-record.md).
- 위 결과는 새 최적화 WASM의 실물 전쪽 대조 완료를 뜻하지 않는다. 기존 측정과 보호 하한은
  그대로 유지하고 새 제품 package의 실제 결과를 별도로 확인한다.


### 공통 빈 기호 제외 범위의 보존

실제 Gulim 프로그램에는 cmap에 대응하지만 glyf가 빈 `U+FFFC`가 있어 공통 Rust 검사기가
FontFace의 unicode-range에서 그 기호를 제외한다. 제한이 하나라도 있으면 TTF 전체를
새 경로에서 제외하는 조건은 일반 본문의 획 보정까지 막으므로, 첫 최적화 빌드는 그 확인
직후 중단했다. 중단한 빌드를 완료·통과로 기록하지 않는다.

Canvas의 제한이 같은 프로그램의 공통 `getFontSymbolUnicodeRange` 결과와 일치할 때는
일반 글리프의 주 프로그램을 공급한다. Rust Print도 동일한 `empty_symbol_codepoints`로
제외된 기호를 다시 cmap 소유로 되돌리지 않는다. 임의의 다른 제한 범위는 계속 등록하지
않는다. 같은 프로그램 검사·descriptor 순서·수명 경계를 확인한 [Studio 집중 13건](verification-record.md)과
[TypeScript 검사](verification-record.md)가 통과했다. 최적화 제품의 실물 재검증은 별도 수행한다.


### 실제 CSS 사다리의 이름 해소

최초 최적화 package에서 기존 21개 브라우저 계약은 통과했으나 새 주 프로그램·완전 CFF 결측
두 반례는 그대로 실패했다. 실제 `fillText` descriptor를 수집해, 따옴표가 포함된 CSS 사다리를
내부 비인용 목록 분리기로 읽어 적재 family를 찾지 못함을 확인했다. API 등록 자체와 조판 트리는
정상이었고 실제 윤곽 소비가 누락됐다. 테스트 기대값을 완화하지 않는다.

기존 `text_font_coverage::select`가 사용하는 `svgtypes::parse_font_families`로 같은 CSS 이름·
이스케이프를 해소한다. 미확인 generic과 앞 family를 건너뛰지 않는 소유 조건은 그대로 유지한다.
이 보정은 wasm32 전용 Print 공급 경로에 한정한다. 최적화 빌드와 실제 브라우저·전쪽 결과는
새 package에서 다시 확인한다. [최초 최적화 반례](verification-record.md).
