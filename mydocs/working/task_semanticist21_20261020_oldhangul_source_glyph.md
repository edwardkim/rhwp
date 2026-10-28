# 옛한글 원본 글꼴의 가시 글리프와 강조점 장평 경계 보정

## 독립 근거와 적용 범위

`d4ad938b3c`의 Native040/fresh WASM015 강조점 검증은 각각 64조건 중 63조건이
90% 이상이다. 옛한글·결합 문자 입력은 87.99371% / 73.64128%로 미달했다.
장평 50/150 조건의 성조 강조점도 숫자 통과와 별개로 가로 원점 차이가 남았다.
이 단계는 이 두 결함을 공통 글리프 선택과 강조점 기하에서 보정한다.

- #2099 → 원 해결 PR #2128은 Hanyang PUA를 표준 자소로 확장해 공개 글꼴에서도
  읽을 수 있게 했다. 원문·표시 문자열·캐럿 주소의 확장은 유지한다.
- 실제 공급 HCR Batang은 F53A에 가시 윤곽선을 갖지만 같은 표준 자소를 GSUB하면
  빈 글리프가 된다. 별도 한컴 Print에서 원 PUA와 대응 표준 자소 모두 F53A로
  인쇄됨을 확인했다. 등록 글꼴 바이트의 cmap·가시 윤곽선으로만 호환 후보를 선택한다.
- 성조 강조점의 첫 원점은 36pt 글자에서 장평 50/100/150 모두 약 32.0226pt다.
  글리프의 가로 배율과 글자 원점으로부터의 앵커 이동량을 구분한다.

근거는 ignored `output/pr-review/semanticist21-20261019/`의
`issue2099-old-hangul-history-189.json`, `pr2128-old-hangul-resolution-190.json`,
`old-hangul-print-source-proof-187.json`, `old-hangul-harfbuzz-probe-181.json`,
`emphasis-tone-ratio-print-188.json`에 보존한다. 글꼴 프로그램은 커밋하지 않는다.

## 생산과 실제 소비

표준 자소→유일한 PUA 후보는 기존 공개 매핑의 역조회다. 모호한 역매핑은 제외한다.
Native는 실제 선택 face의 glyph/path를 확인하고, Canvas2D/SVG는 호스트가 적재한
프로그램의 cmap 범위를 소비한다. CanvasKit은 paint 후보와 실제 선택 font의 glyph
coverage를 대조한 로컬 재생 뷰를 만든다. 원 paint op의 텍스트·위치·문자 주소와
편집 정보는 변경하지 않는다. 가시 PUA가 없는 글꼴은 표준 자소 경로를 유지한다.

Native의 nominal 재생과 complex shaping 선택, Canvas2D의 본문·효과 재생,
SVG의 일반·그림자 출력, CanvasKit의 positioned 재생을 같은 조건으로 확인한다.
글꼴 등록은 새 API로 분리해 기존 HFT 전용 등록 API 계약을 유지한다.

## 검증 순서와 판정

먼저 형식·TypeScript와 Native041을 빌드하고 기존 64조건 및 별도 옛한글 대조군을
같은 원본·독립 Print로 재검증한다. fresh WASM016도 같은 소스로 빌드해 비교한다.
숫자와 직접 확대 판독을 모두 충족한 뒤 정식 회귀를 추가한다. 이전 출력의 점수를
현재 소스의 완료 근거로 재사용하지 않는다. #8064 기능 보존과 나머지 원 PR 검토도
계속하며 이 단계만으로 통합 완료를 선언하지 않는다.

## 동결 단계 `5689bbf34b`의 실행 결과

- Native041의 기존 64조건과 옛한글 대조군 2조건은 66/66 수치 통과했다.
  옛한글·결합 문자, 성조 장평 50/150 및 두 source 글꼴 대조군은 각각 raw 100%다.
  `oldhangul-199-source-glyph-fixed-native/`와 `oldhangul-native041-panels-201/`에
  전쪽 TSV와 직접 판독한 review·overlay를 보존했다.
- 최적화 fresh WASM016은 같은 커밋으로 빌드했다. 66조건 중 63조건은 수치 통과했다.
  `preserve-042/054/099`는 새 보조 글리프 정보 등록의 32MiB 상한 때문에
  `HCR Batang ExtB` 적재가 실패했다. 시각 점수 미달과 실행 실패를 구분하며, 이 세 조건은
  통과가 아니다. `oldhangul-215-source-glyph-fixed-wasm/`에 실패와 성공 기록을 함께 보존했다.
- WASM016의 옛한글·결합 문자와 장평 50/150 및 두 source 글꼴 대조군도 raw 100%다.
  `oldhangul-wasm016-panels-225/`에서 실제 review·overlay를 확인했다. 굵은 성조의
  위치 차이는 94.94930%의 수치 통과로 면제하지 않는다.
- 독립 현대 자소 반례는 Native 83.04260/80.43863% → 94.31094/94.29863%,
  WASM016 94.36356/94.71017%다. 낱 중성·종성의 0폭과 분해 음절의 폭 차이가 남아
  완료·정식 렌더링 회귀 추가로 판정하지 않았다. source cmap/hmtx와 Print의 원점이
  같은 0폭을 입증한다(`oldhangul-modern-font-advances-218.json`).

대량 WASM220은 동일한 글꼴 적재 오류가 반복돼 소유 프로세스 그룹만 종료했다.
`pr8064-wasm-220-interrupted-222.json`에 PID·실제 명령·중단 사유와 결과를 보존했다.
새 등록 처리는 정보 수집 상한·미지원 프로그램을 선택적 미검증으로 처리하고 동일 family의
과거 coverage를 재사용하지 않게 한다. 실제 FontFace 적재를 이 보조 API 때문에 실패시키지
않는다. 자원 상한 회귀 1건을 Native Skia `release-test` lib에서 실행해 통과했다
(3,871건은 선택 필터로 제외). 실행 로그는 `logs/optional-font-coverage-regression-223.log`다.
이 focused 검사는 새 WASM의 실제 대용량 글꼴 적재·재출력을 대신하지 않는다.

후속 batch는 낱 자소 0폭·현대 분해 음절, 실제 문자열 끝점에 맞춘 브라우저 성조 정렬,
Print와 같은 합성 기울임을 함께 보정한다. 굵은체의 빈 자소 전진은 일반체와 다르며
고정 반 em 이동으로 상쇄할 수 없다는 실제 Canvas 측정은
`emphasis-canvas-tone-bold-probe-226.json`에 보존했다. #7694의 새 작성자 보정 3커밋도
다음 소스 단계에 누적 적용한다. 새 두 HWP의 독립 Print는 이미 확보했다.
