# Task M100 #2193 Stage 4 작업보고서 — pagination 구현 방향 게이트

## 0. 판정 요약

- **Stage 판정**: 완료 — 구현 전용 이슈로 분리할 수 있음
- **기준**: `upstream/devel@62bcae43`
- **production 변경**: 없음
- **기각**: 현재 쪽만 갱신한 뒤 downstream page를 그대로 재사용하는 단순 bounded pagination
- **채택 후보**: 거대 표 continuation을 재개 가능한 상태로 만들고 영향 범위를 순차 재조판하며,
  필요하면 UI thread에 양보하는 chunked 실행 경로
- **안전 원칙**: fragment fingerprint가 완전히 일치하지 않거나 의존성을 증명할 수 없으면 기존
  full pagination으로 fallback
- **다음 게이트**: 아래 실행 이슈 초안을 사용자 승인 후 GitHub에 생성하고 구현 범위를 별도 계획

## 1. 코드 경로 판정

| 관찰 | 코드 근거 | 의미 |
|------|-----------|------|
| deferred flush는 항상 전체 paginate | `src/wasm_api.rs`의 `flush_deferred_pagination` | page tree를 무효화한 뒤 `self.paginate()`를 동기 호출한다. |
| dirty section도 조판은 처음부터 시작 | `src/document_core/queries/rendering.rs`의 `paginate_pass`, `src/renderer/typeset.rs`의 `typeset_section_with_variant` | 선택 측정 캐시는 재사용하지만 새 `TypesetState`로 모든 문단을 0번부터 순회한다. |
| 표 continuation 상태는 loop 지역값 | `src/renderer/typeset.rs`의 `typeset_table_paragraph` | `cursor_row`, `start_cut`, `is_continuation`을 fragment마다 전진시키지만 외부에서 재개할 수 없다. |
| `TableBreakToken`은 선언만 존재 | `src/renderer/typeset.rs`의 `TableBreakToken` | `start_row`와 cell offset 형태의 의도는 있으나 현재 조판 경로에서 생성·소비되지 않는다. |
| 수렴 감지는 full typeset 이후 진단 | `src/document_core/queries/rendering.rs`의 `offset != 0` 분기 | 문단 삽입·삭제에만 실행되고, cell edit의 `offset == 0`에는 적용되지 않으며 계산을 줄이지 않는다. |
| 현재 `PartialTable` match는 cut을 무시 | `src/renderer/pagination.rs`의 `matches_with_offset` | row 범위만 같아도 match하므로 page 재사용의 안전 판정으로 사용할 수 없다. |

`paginate_pass`는 시작 시 render-normalized 파생본도 다시 계산한다. 선택 측정은 변경되지 않은
문단을 재사용할 수 있지만, dirty table 측정과 section 전체 typeset·후처리는 남는다. 따라서 Stage 3의
boundary full flush p50 915.4ms(HWP), 954.0ms(HWPX)는 단순 mutation 비용이 아니라 이 동기 경로의
비용이다.

## 2. 단순 page 수렴 재사용을 기각한 이유

#2214 영구 회귀는 target 표가 115개 `PartialTable` fragment로 모든 쪽에 걸친다는 사실과 fragment
연속성을 고정한다. 44번째 입력 뒤 flush 전후를 비교하면 다음과 같다.

- page 0의 `end_cut=[37]`, cursor와 cell bounds는 동일하다.
- page 1도 기존 fragment와 동일하다.
- page 2부터 page 114까지 113개 continuation cut이 실제로 바뀐다.
- page count는 양쪽 모두 115지만 각 fragment의 `start_cut == 이전 end_cut` 연속성은 flush 결과로
  다시 맞춰야 한다.

따라서 visible page가 정확하다는 이유로 pagination을 완료 처리하거나 page 2 이후를 그대로 복사하면
문서 모델과 downstream fragment가 불일치한다. 일반 문서에서 수렴 최적화가 유효할 수는 있지만,
이 fixture에는 첫 unchanged page 탐색만으로 줄일 수 있는 affected range가 사실상 없다.

또한 현재 `matches_with_offset`은 `PartialTable`의 `is_continuation`, `start_cut`, `end_cut`,
`is_block_split`을 비교하지 않는다. 이 match를 그대로 조기 종료 조건으로 승격하는 것은 correctness
gate가 될 수 없다.

## 3. 대안 비교

| 대안 | 기대 효과 | 위험·한계 | 판정 |
|------|-----------|-----------|------|
| unchanged page 이후 기존 결과 복사 | 일반적인 국소 편집은 빠를 수 있음 | 이번 fixture의 113쪽 cut 변경을 놓치며 현 fingerprint도 불충분 | 단독 해법 기각 |
| affected fragment부터 continuation 재개 | section 앞부분의 정규화·조판 반복을 제거하고 affected page당 필요한 표 split만 계산 | footnote, header/footer, master, page numbering과 후속 flow 상태 보존이 필요 | 1차 구현 후보 |
| visible page 우선 + 단일 idle full flush | 첫 화면 반응은 빨라짐 | 약 0.9초 작업을 미룰 뿐 main thread freeze는 남음 | 단독 해법 기각 |
| continuation 재개 + chunk/yield | 즉시 visible state를 유지하면서 긴 propagation을 frame 사이에 분산 | pending 상태·취소·revision 일관성 계약이 필요 | 2차 구현 후보 |
| worker에서 full pagination | main thread 정지를 제거 | WASM document ownership과 대규모 상태 전달 재설계 필요 | 장기 대안 |

우선 구현은 **동일 section의 단일 table continuation edit**로 좁혀 재개 가능한 split 상태를 만들고,
동기 전체 완료가 충분히 짧아지는지 측정한다. affected range 자체가 긴 fixture이므로 목표를 달성하지
못하면 같은 상태 기계를 chunk scheduler에 연결한다. 두 단계 모두 불확실한 구조에서는 기존 full
pagination을 유지한다.

## 4. 안전한 구현 경계

### 4.1 dirty descriptor

deferred cell mutation이 최소한 다음 정보를 보존해야 한다.

- section, host paragraph, table control, cell과 cell paragraph 식별자
- 기존 fragment 중 target cell을 포함하는 최초 affected page/column과 table continuation 위치
- 편집 revision 및 기존 pagination revision
- row/column/span, section/page/column layout의 구조 변화 여부

이는 pagination 완료 플래그가 아니라 `TableContinuationDirty`와 같은 영향 범위 기술자다.

### 4.2 재개 상태

현재 `typeset_table_paragraph` loop의 `cursor_row`, `start_cut`, `start_cut_is_block`,
`is_continuation`과 `TypesetState`의 page/column flow 의존값을 명시적인 resume state로 만든다.
미사용 `TableBreakToken`을 그대로 노출하기보다 실제 fragment cut 표현(`Vec<usize>`)과 column/page
state를 포함하는 새 내부 타입으로 검증해야 한다.

재계산한 fragment는 아래 전체 fingerprint를 이전 결과와 비교한다.

- para/control, start/end row
- `is_continuation`, `start_cut`, `end_cut`, `is_block_split`
- column geometry와 used height
- page number, active header/footer/master, footnote/endnote 및 후속 flow에 영향을 주는 상태

이 fingerprint와 다음 resume state가 모두 같을 때만 downstream convergence를 인정한다.

### 4.3 보수적 fallback

초기 fast path는 아래 조건에서만 허용하고 하나라도 증명되지 않으면 full pagination한다.

- text-only cell edit이며 row/column/span/control 구조가 변하지 않음
- section/page/column definition이 변하지 않음
- target continuation chain에 각주·미주나 pagination 종속 floating object가 없음
- header/footer/master/new-page-number/page-hide 경계가 영향 범위에 없음
- fragment 연속성, page count와 global page offset을 완전히 검증할 수 있음
- 새 edit가 진행 중 작업의 revision과 충돌하지 않음

chunk 실행을 추가한다면 중간 상태를 “pagination complete”로 공개하지 않는다. exact cursor/page-tree
query는 현재 visible fragment와 pending revision을 구분해야 하며, 저장·인쇄·전체 페이지 질의는 작업을
완료시키거나 기존 full fallback을 사용해야 한다.

## 5. 전용 실행 이슈 초안

### 제목

`perf: 거대 셀 table continuation pagination을 resumable/chunked 경로로 분리`

### 본문

#### 배경

#2193 계측에서 115쪽 거대 셀 문서의 첫 flow boundary 입력은 mutation p50 약 0.2ms에 비해
동기 full pagination p50이 HWP 915.4ms, HWPX 954.0ms였고 input-to-2-rAF는 약 1.0~1.1초였다.
native full flush도 약 1.14초다. #2214 correctness pin에 따르면 page 0 cursor는 flush 전에도 exact지만
page 2~114의 113개 continuation cut은 flush 후 실제로 재정렬된다.

#### 목표

1. dirty table fragment부터 continuation split을 재개해 section 처음부터의 조판을 피한다.
2. affected chain이 길 때 작업을 안전하게 chunk하고 UI thread에 양보할 수 있는 상태 계약을 만든다.
3. unsupported 문서 구조는 기존 full pagination으로 fallback한다.
4. 기존 #2214 정확성 핀과 #2193 native/browser profile로 전후 효과를 검증한다.

#### 작업 단계

1. render normalization, selective table measurement, table continuation loop와 pagination 후처리에 내부
   subphase timer를 추가해 full flush 비용을 분해한다.
2. deferred cell edit에서 section/host/table/cell/revision과 최초 affected fragment를 담는 dirty
   descriptor를 만든다.
3. table split cursor와 필요한 page/column flow state를 resume token으로 만들고 affected fragment부터
   재계산한다.
4. 완전한 `PartialTable` fingerprint와 downstream 상태가 일치할 때만 수렴·재사용한다.
5. 동기 목표를 충족하지 못하면 같은 resume state를 frame/idle chunk scheduler에 연결하고 취소·revision
   교체 규칙을 추가한다.
6. 각 단계에서 unsupported dependency와 검증 실패는 full pagination으로 fallback한다.

#### 완료 조건

- HWP/HWPX 115쪽 fixture에서 44번째 입력 후 page 0~114 fragment 연속성, 115쪽, model/tree/layout,
  exact cursor/caret가 full pagination oracle과 동일하다.
- `PartialTable` fingerprint는 cut, continuation과 block split 상태를 포함한다.
- 저장·인쇄·전체 페이지 질의가 pending pagination을 완료하거나 안전하게 fallback한다.
- stable 입력의 flush 0회 계약과 기존 #2214 native/browser 회귀가 유지된다.
- 같은 #2193 profile로 before/after raw JSON과 p50/p95를 기록한다.
- 목표 latency는 subphase 계측 후 별도 수치 gate로 확정하며, 단순히 full flush를 idle로 미룬 결과를
  완료로 간주하지 않는다.

#### 관계

- parent tracking: #2193
- correctness prerequisite/regression oracle: #2214 / PR #2241
- selection rect #2215 / PR #2401, revision cache #2308, table border click #2400과는 별도 범위

## 6. 다음 작업

1. 사용자 승인 후 위 초안으로 전용 GitHub 실행 이슈를 생성하고 #2193에 연결한다.
2. 현재 계측 브랜치를 push하고 계측·Stage 4 기록 PR을 생성한다.
3. 실행 이슈에서 먼저 subphase 계측을 구현한 뒤 resume state의 최소 범위를 다시 승인받는다.

이번 Stage에서는 production paginator, 공개 WASM API, GitHub issue/comment를 변경하지 않았다.
