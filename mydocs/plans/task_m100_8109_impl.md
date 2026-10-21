# #8109 구현계획 — 원본 font identity와 저장·출력 face 연결

- 날짜: 2026-10-21 (Asia/Seoul)
- 기준: `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`, 브랜치 `task_m100_8109`.
- 상위: [수행계획](task_m100_8109.md). 상태: 2026-10-21 진행 승인,
  [1단계 진단 완료](../working/task_m100_8109_stage1.md) 뒤 2단계 승인·구현 및 집중 검증 완료.
  [2단계 보고](../working/task_m100_8109_stage2.md)를 제출하고 최종 3단계 승인 대기.

1단계에서 명시적 Arial 대체의 독립 한컴 Print를 확인했다. 2단계는 원본 이름을 유지한
PDF 대체 정책과 HWP 대체 필드의 공통 모델 정규화를 구현한다. HWPX→HWP→HWPX에서
대체가 사라지는 실제 재현과 프로그램 유무의 적용 경계는 단계 보고서의 최소 수정 계약을 따른다.

## 확인한 소비 경계와 조사할 변경 지점

| 경로 | 현재 확인한 사실 | 구현 전 확정할 계약 |
| --- | --- | --- |
| `src/parser/pdf/lowering.rs`의 font 등록 | `span.font_name`에서 family를 정규화하고 공통 font/CharShape를 생성한다 | source 선언·프로그램 유무와 대체 identity를 구별하고 저장할 정보의 실제 생산 지점을 확인한다 |
| `src/parser/pdf/structured_lowering.rs`의 glyph 측정 | 공통 `Styles`·`GlyphWidths`를 소비하며 Unknown과 선언 글꼴을 구별한다 | face 변경이 source nominal advance와 측정/실제 줄 구성에 미치는 영향을 확인한다. 기존 장평/자간을 score 보정으로 바꾸지 않는다 |
| `src/renderer/font_metrics_data.rs`, `font_metrics_core14.rs` | 독립 Adobe AFM 자료를 사용한다 | 메트릭 선택과 실제 glyph 선택의 차이를 기록하고, 실제 프로그램 없이 glyph identity를 주장하지 않는다 |
| `src/renderer/font_environment.rs` 및 실제 Native/Web paint font 선택 | `FontEnvironment`는 layout·paint용 session 선언이며 document IR에 저장하지 않는다는 계약이다 | session 치환만으로 HWP/HWPX/한컴 저장 일관성을 해결했다고 판단하지 않는다. 최종 face 선택·덮어쓰기·fallback을 실제 호출 끝까지 추적한다 |
| `src/serializer/header.rs`, `char_shape.rs`, `hwpx/header.rs`와 대응 parser | 공통 글꼴·글자모양의 양 저장 경로다 | 실제로 표현 가능한 대체/임베딩 필드와 한컴의 소비를 사양·정상 생성본·Print로 확인한다 |

변경 전에 한 값의 `source face → IR font/CharShape → 측정 → paint의 실제 face → 저장 필드 →
재열기/한컴 실제 face`를 코드 위치와 관측값으로 잇는다. 정확한 font fallback backend와
저장 필드의 최종 변경 여부는 1단계 실행 결과로 정하며, 위 파일을 일괄 수정하지 않는다.

## 구현 방향의 결정 기준

1. source glyph program이 실제 있는 경우의 기존 선택과 없는 Base14 선언의 경계를 나눈다.
   family 이름·AFM 폭만으로 프로그램 존재를 판정하지 않는다.
2. 프로그램 없는 경우에 사용할 명시적 대체 family와 저장 표현을 확인한다. 원본 face와 대체
   출처를 보존하면서 해당 출력 환경과 한컴에서 재현 가능한 최소 정책을 선택한다.
   특정 PC의 전역 font alias를 바꾸거나 상용 글꼴을 저장소에 추가하지 않는다.
3. 같은 결정이 무편집 저장·직접 편집·재열기 뒤 재편집의 실제 측정/paint에서 소비되는지 확인한다.
   보이는 글꼴만 바꾸고 메트릭이 다르게 적용되거나, 저장 때만 다른 face가 생기는 경계를 보정한다.
4. 먼저 목록 6경로와 적용/비적용 대조군의 독립 출력으로 방향을 확인한다. 구현이 바뀌면 해당
   경계·Native/fresh WASM 출력을 다시 산출한다. 안정화 후 최종 전체 검증으로 넓힌다.

## 검증과 증거

- 원본 `body-list.pdf` 및 기존 HWP/HWPX 저장본·Print의 바이트를 공개 `inputs.tsv`와 대조한다.
  최신 base를 실제 재현하고 이전 89%대 기록과 분리한다. 입력이 같은 독립 PDF는 재사용한다.
- 바뀐 동일 편집 저장본은 저장 제품 버전에 맞는 한컴 Print로 검증한다. 새 입력을 과거 PDF에
  대조하지 않는다. 원래/대체 face, 프로그램 hash·공급 경로·실제 선택 trace를 증거에 연결한다.
- source·무편집·X 삽입·직접 역삭제·Snapshot undo/redo·양 저장/재열기/재편집을 실제 Native와
  Studio WASM에서 실행한다. 목록 2줄·들여쓰기·뒤 문단·원문 소속/순서를 관계로 검사한다.
- 전쪽 Native/fresh WASM TSV를 먼저 산출하고, 같은 조건의 base/candidate·독립 Print에서
  glyph 모양·농도·글자 사이·기준선·줄바꿈을 확대 대조한다. 전쪽 90%·쪽수 일치와
  base 대비 0.1%p 하락 규칙을 모두 만족시킨다.
- 시각 선행 조건 이후에만 `tests/cases/`의 기존 자동 수집 경로로 회귀를 등록한다.
  수정 전 의도한 원인으로 실패했는지 실행한다. 진단 helper나 글자 존재만으로 최종 출력을
  검증했다고 쓰지 않는다. px 고정은 canonical의 페이지별 양 backend 98% 조건을 따른다.
- 제출 전 [local_validation 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
  전체 Rust lint·회귀·Native Skia·최적화 fresh WASM·실제 브라우저와 해당 시각 절차를 완료한다.
  `FontEnvironment` 또는 저장 모델의 변경 범위에 따라 public API·양 serializer 계약도 확인한다.
- 원문 TSV·대표 비교/overlay·실행 로그는 `output/pr-review/issue8109/`에 구분해 보존한다.
  제출 증거만 안정 경로에 보존하고 source/head·입력/Print SHA·명령·잔여 차이를 기존 보고에 잇는다.

전쪽 기준 미달·확인된 결함·필수 증거 부족은 보정·재검증으로 해소한다. 과거 #7998의
글꼴 예외와 CI 통과를 #8109의 해결 근거로 바꾸지 않는다.
