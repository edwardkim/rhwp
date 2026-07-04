# Task #1658 v3 2단계 완료 보고 — 하단 배타 예약 구현 (#1611/#1624 통합)

## 구현 내용

### 1. 판별 함수 (`src/renderer/float_placement.rs`)

`is_page_bottom_fixed_float(common)` = `!treat_as_char ∧ wrap=TopAndBottom ∧
vert_rel_to=Page ∧ vert_align=Bottom`. 기존 `is_para_topbottom_float` 불변.

### 2. typeset 배타 회계 (`src/renderer/typeset.rs`)

기존 **#1611 블록**(`typeset_block_table` 내 Page·Bottom 처리)을 배타 모델로 재작성
— 별도 신규 경로가 아니라 기존 경로의 소비 모델만 교체해 #1611/#1624 가드를 계승:

- 상태 2종: `current_bottom_fixed_exclusion`(하단 배타 높이, **max-union** — 겹침
  허용이므로 합이 아님) + `bottom_fixed_consumed_flow`(이 페이지에서 하단 틀이
  소비했을 저장-flow 높이 누계). `available_height()` 에서 배타 높이 차감, 페이지
  전환 시 리셋 (각주 예약과 동일 패턴).
- **vpos 동기화 보정** (핵심): 한글 저장 vpos 는 하단 틀도 문서순으로 누적하므로,
  후속 틀의 vpos 동기화(#1611) 시 `bottom_fixed_consumed_flow` 를 차감해 본문
  텍스트 끝 위치를 복원한다. 이 보정이 없으면 둘째 틀의 vpos(선행 틀 소비 포함)가
  배타 경계를 넘겨 spurious 이월(관악 케이스 2쪽 잔존)된다.
- **이월 판정**: `sync_h > available(배타 반영) → advance` — #1611 의 "본문이 배타
  영역을 침범하면 틀을 다음 쪽에 단독 배치" 시멘틱 유지 (36387725 한글 2쪽 정합).
  #1624 plausibility 가드(vpos ≤ cur+block_height 일 때만 동기화)도 유지.
- **소비 롤백**: `place_table_with_text` 후 current_height 를 배치 전으로 복원
  (하단 틀은 flow 를 소비하지 않음) + 배타/소비누계 갱신.

### 3. 렌더 절대 y 복원 (`src/renderer/layout/table_layout.rs`)

1×1 래퍼 unwrap(619~)이 외곽 표의 페이지/용지 앵커 절대배치를 소실시키고 내부 표를
flow 커서에 렌더하던 결함 교정 — 외곽이 Page/Paper 앵커 자리차지면
`compute_table_y_position` 으로 절대 y 를 계산해 내부 표 시작점으로 사용.

## 검증 (대표 케이스)

| 케이스 | 수정 전 | 수정 후 | 한글 |
|--------|--------:|--------:|------|
| 관악 36389312 (하단 틀 2개) | 2쪽 | **1쪽** | 1쪽 ✅ |
| 디지털도시국 36398366 (PC셧다운, RCA 원본) | 2쪽 | **1쪽** | 1쪽 ✅ |
| 36387725 (#1611 픽스처, 본문이 배타 침범) | 2쪽 | **2쪽** | 2쪽 ✅ (이월 유지) |
| 관악 하단 렌더 | p2 상단(59~271pt) | **p1 하단부** | 하단 ✅ |

### 1차 시도 교훈 (본 라운드 내)

단순 배타 분기(별도 경로, vpos 동기화 미계승)는 36387725 를 2→1쪽으로 **악화**시켰다
(#1611 테스트 RED). 원인: 한글의 이월 판정은 저장 vpos 기준 본문 끝(640.7px, rhwp
flowed 578px 대비 +62px 드리프트)을 쓰는데 배타 분기가 이를 우회. → #1611 블록에
통합 재작성으로 해소 (일치 74→75).

## 게이트 (2단계 시점)

| 게이트 | 결과 |
|--------|------|
| 통제셋 92 | **일치 73→75(+2), over 5→3, under 14 불변, 악화 0** |
| 신규 가드 3건 (`issue_1658_page_bottom_fixed_exclusion`) | 통과 |
| `issue_1611_footer_page_bottom_pagination` | 통과 (유지) |
| opengov 스냅샷 (대표 2건 편입, 22건) | 통과 |

산출: opengov 편입 2건(공개 결재문서 36389312·36398366, 36~38KB) + 가드 테스트.
