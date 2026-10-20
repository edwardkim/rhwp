# #7998 stage37 — 범위 종료·최신 devel 통합과 최종 검증

2026-10-20 작업지시자가 남은 구현을 후속 이슈로 분리하고 P5로 마무리하도록 지시했다.
완료 범위는 현재까지 구현한 의미 표/문단·편집/저장 계약이다. 모든 PDF 작성 구조의 복원이나
후속 #8106·#8107·#8108의 해결을 주장하지 않는다. 이 문서는 진행 중 기록이다.

## 고정 기준과 범위

- 직전 로컬 구현: `77f46b19e` (stage36, 동일 바이트 후보 정식 등록 포함).
- fetch한 원격 devel: `5e2b9bfbf`.
- root devel은 사용자 untracked 피드백 두 파일을 보존한 채 해당 remote로 fast-forward했다.
- 남은 B24 내용 성장 격자 → #8106, 보고서6쪽 glyph 폭 → #8107, 본문1/4쪽 자간 → #8108.
- stage36의158입력169쪽/각 backend, Print32저장본·Studio15경로는 병합 전 증거다.
  병합 후 제품이 바뀐 경로의 검증을 대신하지 않는다. 같은 입력/PDF는 재사용한다.

## 병합 보정

16개 충돌의 양쪽 구현을 대조했다. 제품 경로는 PDF source stroke의 path 투영과 새 metafile
image payload를 함께 유지하고, 명시적 physical dash와 새 HWP dash 표현을 함께 소비한다.
글자 위치는 최신 devel의 언어별 percent 속성으로 통일해 pixel offset과 이중 적용하지 않으며,
paint bounds/fast-lane도 같은 위치를 고려한다. 공통 TAC 셀 projection·source row 소유를 보존한다.
독립 Base14 AFM과 새 한컴 font overlays를 함께 유지한다. CFB의 border/text-clip 보존 stream과
HWP3 재정렬 stream은 공존한다. Layer minor27 충돌은 두 계약을 합친28로 구분한다.

최신 devel의 PDFium 기준 rasterizer를 채택한다. 과거 Poppler TSV는 그대로 보존하고 새 최종
시각 검증과 혼동하지 않는다. stroke 상태가 같은 fill을 바꾸지 않는 독립 계약은 유지한다.
상세 충돌 목록은 ignored `output/pr-review/issue7998/final-integration/merge-resolution.json`이다.

디스크 여유7GiB로 필수 빌드가 어려워, 실행 중 Cargo/rustc/wasm 도구가 없음을 확인한 뒤
24시간 이상 미사용한 debug incremental12개만 정리해12.36GiB를 확보했다. 공유 target의
실행 파일·deps·fingerprint·다른 증거는 유지했다. 경로/시간/전후 여유는 같은 폴더
`incremental-cleanup.json`에 기록했다.

## 최종 게이트

진행 순서는 병합 제품의 컴파일·집중/시각 방향 확인 → 필수 전체 Rust/lint/Native Skia·
Worker/package·실제 Studio와 전쪽 fresh WASM → 기록/본문 준비다. 후속 구현을 시작하지 않는다.
아직 이 단계의 통과·PR 준비 완료를 선언하지 않는다. 원격 push·PR 생성·merge는 별도 절차다.

## 첫 통합본의 완료 범위와 실패

첫 통합 제품 `aa73b27d7`의 Native/fresh Docker WASM을 동결했다. Native158입력169쪽은
글꼴 미달6쪽(최저89.54211%)을 제외하고90% 이상, WASM은169쪽 모두90% 이상
(최저90.70673%)이다. 모두 누락·쪽수 차이는 없다. 이 결과를 뒤의 폭 격자 보정 제품의
증거로 재사용하지 않는다. 원본·기준 PDF와 실제 코드/입력이 같은 증거만 재사용한다.

실물A는 현재 IR의15표161셀·5324문자, 실물B는1표60셀·267문자다. 양 저장 형식에서
표/셀의 중첩 순서와 보이는 원문 순서·쪽수12/1이 유지됐다. 이것은 현재 IR 왕복 검사이며
작성 당시 표·셀 수의 정답을 입증한 것은 아니다. 실제 Studio15경로의 삽입·직접 삭제·
Snapshot undo/redo·양 저장은 최종 Native와 바이트가 같다. Worker7개123 assertion도
통과했다. 첫 실행의 Chrome 경로 누락은 환경 실패로 구별해 올바른 경로로 실행했다.

동일 편집 저장본30개 중24개는 stage36에서 인쇄한 입력과 바이트가 같아 그대로 재사용했다.
긴 문단의 새6저장본만 새 한컴2020 Print4작업+2동일 입력 alias로 검증했다. 같은169쪽
집합에서 해당6개만 교체하고152개는 실행 파일/입력/PDF의 지문 cache로 재사용했다.

Helvetica 목록6건은 입력에 프로그램이 없는 Helvetica가 선언돼 있고, 실제 Native 관측은
시스템 Noto Sans KR, 한컴 Print의 내장 face는 Haansoft Batang이다. Arial/HANBatang/Batang
공급 디렉터리의 실제 파일을 확인했다. 이전/현재의 문단·글줄·글자 bbox/소속은 모두 같았다.
정식 일반 PNG 모드의 `font_mismatch_exception`6건을 확보했다. raw미달을 그대로 유지하며
일반통과나 자동승인으로 바꾸지 않는다. B08 후속은 #8109, 실물B의 가는 선/작은 글자
농도 차이는 #8110으로 분리했다. 후자는 원인 확정이 아닌 후속 조사이며 devel 대비 하락0건을
주장하지 않는다. 원본·식별 가능한 이미지는 로컬에만 유지한다.

Rust release 빌드는 성공했다(19분30초). lib test 생성자3곳의 새 모델 필드 누락은
`#[cfg(test)]` 영역에서 false/None으로 보완했다. 제품은 바뀌지 않아 앞선 빌드·시각 결과를
반복하지 않았다. 이후 release lib는3800 PASS/2 FAIL/12 ignored였다. 실패는 편집 때
`content_for_style`이 새 devel의4 HWPUNIT 줄 폭 격자를 누락한 것이다.

## 폭 격자 보정 — 병합 회귀만 수정

원본 소유자 폭5002 HU와 원시 칸 폭4998 HU를 구분하는 합성 입력이다. 저장 줄 폭은
독립 한컴 줄 메트릭의4 HU 규칙(#7412:56문서/19837행)에 따라 각각5000/4996이 된다.
수동 합성 입력의 가정을 실제 한컴 저장본과 혼동하지 않는다. 기대값을 느슨하게 하지 않고
`ParagraphBox::content_for_style`의 전체 내용 폭을 먼저 스냅한 뒤 원래 문단 여백을 뺀다.
원점/양 끝을 각각 스냅하거나 출력을 clamp하지 않는다.

실제 소비는 `commands/text_editing.rs`의 flat/nested 편집과 `commands/document.rs`의 초기/
on-demand 재조판 → `ParagraphBox::effective/frame` → `composer::reflow_line_segs`의
column_start/segment_width 발행이다. paint의 `layout/paragraph_layout.rs`·`shape_layout.rs`는
같은4 HU 기준을 여백보다 먼저 적용하는 `body_for_style`의 frame을 소비한다. 행/쪽 분할,
이어받기 높이·glyph 크기·장평/자간은 바꾸지 않는다.

첫 focused편집2건은 통과했다. 빠른 전체 lib에서 초기/on-demand2건이 반대 기대값5002±1을
고정하고 있음을 확인했다. 소유자 폭5002 검사는 유지하고, 두 저장 줄 기대값만 같은 독립
4 HU 규칙의5000으로 엄격하게 맞춘다. 원시 칸 폭·중복 패딩을 허용하는 범위 완화가 아니다.
보정 제품의 focused/전체 lib와 영향 시각 검증을 먼저 확인한 뒤 최종 전체 게이트를 마친다.
이 기록은 아직 최종 통과 판정이 아니다. B24·glyph/자간·글꼴/농도 후속 구현을 시작하지 않는다.

폭 격자 일관화 후 빠른 전체 lib는 root 3802 PASS/12 ignored, 내부 crate 182 PASS로
실패가 없다. 물리 소유자 폭과 저장 줄 폭을 구분해 초기/편집 양쪽을 확인했다.
최종 Native/fresh WASM은 이 제품으로 다시 동결하고, 같은 입력·독립 PDF를 재사용한다.
