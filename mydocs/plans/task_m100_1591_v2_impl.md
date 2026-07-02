# Task #1591 v2 구현계획서 — first-para mismatch-path 슬롯 위치추정 통합 정합

## 전제 (재현 확정)

`local/task1591-v2` 베이스에서 타깃 3건 모두 `rhwp hwpx-roundtrip` IR_DIFF=1 재현:

- 36384689 (강남소방서 화재발생종합보고서) — char_shape +8
- 36385445 (송파소방서 화재발생종합보고서) — char_shape +8 (동일 패턴)
- 36388711 (사회보장제도 신설 협의요청서) — char_shape −16/−8 + cc −8

## 코드 구조 이해 (사전 정독)

`src/serializer/hwpx/section.rs` 의 run 방출 3경로:

1. **fast path** (525): 슬롯·필드 없음 → 단일 run.
2. **mismatch 경로** (536): `inferred_control_slot_count(para) != slots.len()` —
   위치 추정 포기 → 텍스트(경계 분할) 후 **슬롯 일괄 말미 방출**.
   - 고아 fieldEnd 는 말미 일괄 복원(#1556)하지만 **`para.field_ranges` 의 균형
     fieldEnd 는 방출 코드가 없다** → cc −8 드롭 (#1593 의 유력 근원 — Stage 1 검증).
3. **메인 경로** (563+): UTF-16 위치 축(`char_offsets` + expected_utf16_pos)에서
   슬롯/필드/문자/경계를 함께 처리 — 위치 정확.

`inferred_control_slot_count` (775): `max(cc 잉여/8, char_offsets 갭/8) − field_ranges
− orphan_field_ends`. para0 에서 SectionDef/ColumnDef 도 cc 상 8유닛을 점유하므로
카운트에 포함되는 반면, `slots` 는 SectionDef 제외(secPr 템플릿)·첫 ColumnDef
제외(#1584 템플릿 흡수) → **카운트≠슬롯 불일치가 구조적으로 발생**해 첫 문단이
항상 mismatch 경로로 빠진다.

## 단일 근원 가설 (Stage 1 검증 대상)

Class C 3건 모두 "para0 = 첫 문단 → 템플릿 흡수 → mismatch 경로" 진입이 근원:

- +8 시프트(C1): mismatch 경로가 슬롯을 char-offset 위치가 아닌 말미/일괄 위치에
  방출 → 8유닛 슬롯이 char_shape 경계 앞으로 이동.
- fieldEnd 드롭(C2, cc −8): mismatch 경로에 균형 field_ranges 방출 부재.
- 북마크 hoist: 비-empty 문단 북마크의 문단 시작 강제(466) — 1라운드에서 순서
  교정 확인(순효과 0이라 롤백됐으나 실재 버그).

## 단계 구성 (4단계)

### Stage 1 — 근본 정밀 규명 + RED 테스트

- 36384689 para0: cc=33, controls=5(`SectionDef, ColumnDef, Table, PageNumberPos,
  Bookmark`), char_offsets/char_shapes 를 덤프해 원본 XML run 구조
  (`[secPr,ctrl,colPr] / [tbl,tbl,t] / …`)와 슬롯 8유닛 축을 정확히 매핑 —
  slot_count=4 vs slots=3 의 구성 요소를 특정.
- 36388711 para0: mismatch 경로 진입 여부 + field_ranges 방출 부재로 fieldEnd 드롭이
  설명되는지 확정 (단일 근원 가설 검증).
- 수정 방향 확정:
  - **(a) 카운트-슬롯 정합**: 템플릿 흡수분(SectionDef/첫 ColumnDef)을
    `inferred_control_slot_count` 에서도 동일 규칙으로 차감해 mismatch 자체를 해소 →
    첫 문단이 메인 경로(위치 정확)로 진입. 단 cc 축에서 흡수분의 8유닛은 실존하므로
    슬롯 위치 축(expected_utf16_pos)의 시작 오프셋 보정이 필요할 수 있음.
  - **(b) mismatch 경로 개선**: 경로 유지하되 슬롯을 char_offsets 증거 위치에 방출
    + field_ranges fieldEnd 방출 추가.
  - 판단 기준: (a)가 메인 경로 재사용으로 정합성이 높지만 진입 문단 집합이 넓어질
    수 있어 통제 비교 리스크 큼. (b)는 국소적. Stage 1 매핑 결과로 결정.
- RED: 타깃 3건 최소 재현 단위 테스트(fixtures 합성 또는 실파일 스니펫) +
  1라운드 `task1591_bookmark_not_hoisted_before_slot` `#[ignore]` 해제 준비.

### Stage 2 — 구현

- Stage 1 확정 방향 구현 (`section.rs`, 필요 시 `roundtrip.rs`).
- 북마크 hoist 제거 편입: 1라운드 Stage 2 교정(비-empty 문단도 in-order 방출) 재적용.
- same-para fieldEnd: 확정 경로에 field_ranges 방출 추가(위치 보존).
- 게이트: RED→GREEN, `cargo test --test hwpx_roundtrip_baseline` 회귀 0,
  `cargo test --lib` 회귀 0.
- 산출: 소스 커밋 + `task_m100_1591_v2_stage2.md` (Stage 1 보고는 `_stage1.md`).

### Stage 3 — 통제 비교 (채택 게이트)

- hwpdocs 전수 `rhwp hwpx-roundtrip --batch` 수정 전/후 (공통 ~10150):
  - 타깃 3건 IR diff=0
  - **개선 − 회귀 > 0, 악화 0** (1라운드와 동일 채택 기준)
- 악화 ≥1 → 전량 롤백 + 재분석. 광역 회귀 불가피 판명 시 불채택·문서화 종료.
- 전/후 inventory.tsv 는 `output/poc/task1591_v2/` 에 보관, 집계 스크립트는
  `tools/` 에 커밋 (검증 파일 PR 포함 규칙).

### Stage 4 — 가드·최종 보고

- 단위 테스트 편입(합성 fixture 우선, 실파일 인용 불가 시 opengov 동형), snapshot
  영향 확인.
- `task_m100_1591_v2_report.md` + 이슈 #1591·#1593 수용 기준 대조표.

## 리스크

- F3(#1561) 2회 회귀·1라운드 순효과 0 전례 — 슬롯 위치 축은 문단 유형(빈 문단,
  autoNum, 고아 fieldEnd, 셀 subList depth>0)별 특례가 많다. (a) 방향 선택 시
  mismatch→메인 경로 이동 문단 전수의 diff 를 통제 비교로 확인.
- 템플릿 흡수(#1584)·empty-text 북마크(#1627)·고아 fieldEnd(#1556) 게이트 테스트가
  이미 존재 — baseline/lib 로 커버. 특례 간 상호작용이 최대 위험.
