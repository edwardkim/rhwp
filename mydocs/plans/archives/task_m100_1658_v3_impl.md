# Task #1658 v3 구현계획서 — 페이지 하단 고정 표 "하단 예약" 배치

## 코드 구조 이해 (사전 정독)

- 판별 지점: `src/renderer/float_placement.rs:15 is_para_topbottom_float()` —
  `VertRelTo::Para` 한정. `VertRelTo::Page` 하단 고정 표는 여기서 제외되어
  `typeset_table_paragraph`(typeset.rs:10329)의 일반 flow 배치로 누적.
- 예약 선례: `available_height()`(typeset.rs:1266)는 이미 **각주 예약**
  (`current_footnote_height` + safety margin) 차감 구조 — 하단 예약도 동일 패턴으로
  차감 항목을 추가하고 페이지 전환 시 리셋하면 회계가 성립한다.
- 렌더 측: layout(table_layout.rs:2141 계열)은 `VertRelTo::Page` 를 절대배치 처리 —
  단일 표 위치는 정상. **다수 스택**(아래→위)의 y 산정이 미지 영역(Stage 1 확인).
- 속성: `CommonObjAttr.vert_align`(model/shape.rs:73) = Bottom 판별.

## 단계 구성 (4단계)

### Stage 1 — 케이스 집합·베이스라인 고정 + 스택 렌더 실태 확인

- 대표 over 케이스 dump 고정: PC 셧다운(36378545 등 계열), 관악 화재발생
  알림(36389312), #1653 RCA 원본(디지털도시국 PC/관악) — 하단 고정 표 속성
  (`vert_rel_to/vert_align/wrap/tac`)·높이·개수 표.
- **한글 하단 스택 실태 확인**: 대표 케이스의 한글 편집기/PDF 기준으로 2개 표가
  같은 페이지 하단에 어떻게 쌓이는지(간격·순서) 확정 — 렌더 스택 y 산정의 정답 근거.
- 코퍼스 스캔(간이): 통제셋 92 + 대형 오라클 452 중 "vert=Page·valign=Bottom 표
  ≥1 보유" 문서 수와 "≥2 보유" 문서 수 정량화 — 영향 반경과 회귀 표면 파악.
- 베이스라인 고정: 통제셋 92 현행 분포(통합 베이스 일치 73 / under 14 / over 5)의
  파일별 스냅샷 저장 (Stage 3 무회귀 대조용).
- 산출: `mydocs/working/task_m100_1658_v3_stage1.md`.

### Stage 2 — 하단 예약 모델 구현

- **판별 함수** (float_placement.rs): `is_page_bottom_fixed_float(common)` =
  `!treat_as_char ∧ wrap=TopAndBottom ∧ vert_rel_to=Page ∧ vert_align=Bottom`.
  (기존 `is_para_topbottom_float` 불변 — 신규 함수 추가로 격리.)
- **typeset 회계** (typeset.rs):
  - 상태 추가: `current_bottom_reserved: f64` + 하단 스택 큐(페이지별
    `(para_idx, ctrl_idx, height)` 목록). `available_height()` 에서 차감,
    페이지 전환 리셋 지점(각주 리셋과 동일)에서 초기화.
  - `typeset_table_paragraph` 진입에서 판별 시: 표 높이(외곽 마진 포함)를 예약하고
    PageItem 에 하단 스택 표식으로 등록, `current_height` 미누적.
    잔여 공간 부족 시 다음 페이지로 이월 예약.
  - Stage 2b 교훈 반영: 판별은 4속성 교집합만, TAC(treat_as_char)·vert=Para·
    valign≠Bottom 은 전부 기존 경로 유지.
- **렌더 스택 배치**: PageItem 하단 표식 → 렌더 y = 본문 하단 − (스택 누적 + 표높이).
  layout 의 기존 Page 절대배치와 이중 적용되지 않도록 단일 소유 확정 (Stage 1 실태에 맞춤).
- 가드 테스트: 대표 케이스 페이지 수(2→1) + 하단 스택 y 순서 + 1개-표 문서 무변화.
- 산출: 소스 커밋 + `_stage2.md`.

### Stage 3 — 통제 검증 (채택 게이트)

| 게이트 | 기준 |
|--------|------|
| render_page_gate 소형(92) | **일치·under 무회귀(파일별 대조)** + over 5 중 페이지 앵커 계열 해소 |
| render_page_gate 대형(452) | 443+ 무회귀 |
| clipping_gate (92) | 회귀 0 |
| valign_offset_gate | fixture 4종 유지 |
| 페이지네이션 샘플 | byeolpyo 4/26, giant 42, scattered 53(#1748 Δbot −4) |
| `cargo test --release` 전체 | 실질 실패 0 (svg CRLF 노이즈 제외) |

- Stage 2b 재발 감시: 파일별 대조에서 일치→under 전이가 1건이라도 나오면 판별/예약
  로직 재검토 (전량 롤백 옵션 유지).
- 산출: `_stage3.md`.

### Stage 4 — 최종 보고

- `task_m100_1658_v3_report.md`: 결과 + #1658 잔여 지도 갱신 (패턴 A 는
  razor-thin(#1759)·saved-vpos(#1772) 프로그램과 범위 중복 정리).
- 재현 샘플: 대표 케이스 중 공개 결재문서를 `samples/hwpx/opengov/` 또는
  `samples/task1658/` 에 편입 (PR 재현 파일 규칙).

## 리스크

- **판별 과대 → under 회귀** (Stage 2b 75→55 전례): 4속성 교집합 + 파일별 무회귀
  대조로 방어.
- 하단 예약 vs 각주 예약 동시 발생 페이지: available_height 차감 순서·중복 확인,
  opengov footer 가드(#1611/#1624)로 방어.
- 예약 표가 페이지를 넘겨야 하는 경우(잔여 공간 < 표높이)의 anchor 문단과의 순서
  — 이월 시 앵커 뒤 본문이 먼저 채워지는 한글 관례(#1753 prefill 계열)와의 상호작용은
  범위 밖으로 격리(발견 시 문서화).
