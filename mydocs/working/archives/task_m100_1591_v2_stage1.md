# Task #1591 v2 1단계 완료 보고 — 근본 정밀 규명 + 실파일 가드 편입

## 결론 (요약)

**단일 근원 가설 확정**: Class C 3건 모두 "첫 문단(para0)의 hidden 슬롯(secPr/colPr)이
cc 축에는 있으나 방출 슬롯 목록에 없어 **구조적으로 mismatch 경로에 진입**"이 근원이다.
C1(+8)과 C2(fieldEnd 드롭, cc −8)는 mismatch 경로의 서로 다른 두 결함으로 갈라진 증상이다.

## 위치 축 매핑 (원본 XML ↔ IR 대조)

### 36384689 (C1, +8) — para0: cc=33, text_len=0, controls=5

원본 run 구조 / UTF-16 위치 축:

| 위치 | 내용 | run | charPr |
|------|------|-----|--------|
| 0..8 | secPr (SectionDef) | run[0] | 25 |
| 8..16 | ctrl(colPr) (ColumnDef) | run[0] | 25 |
| 16..24 | tbl (Table, 글자처럼) | run[1] | 25 |
| 24..32 | ctrl(pageNum) (PageNumberPos) | run[2] | **10** ← char_shape (24,10) |
| 32 | ctrl(bookmark) (zero-width) + t"" | run[2] | 10 |

cc = 33 = 4슬롯×8 + 1. `inferred_control_slot_count` = (33−1−0)/8 = **4** (물리 축).
방출 슬롯 = SectionDef 제외(템플릿) + 첫 ColumnDef 제외(#1584 흡수) → **4 ≠ slots.len()**
→ mismatch 경로(section.rs:536) → 슬롯을 char-offset 없이 일괄 방출 → pageNum 8유닛이
경계(24) 앞으로 이동 → char_shape (24,10)→(32,10) **+8**. 36385445 동형.

### 36388711 (C2, −16/−8 + cc −8) — para0: cc=82, text_len=49, controls=4

| 위치 | 내용 | run | charPr |
|------|------|-----|--------|
| 0..16 | secPr + ctrl(colPr) | run[0] | 7 |
| 16..24 | ctrl(fieldBegin) (Field) | run[1] | 7 |
| 24..44 | t"【서식 1-1】…양식"(20자) | run[1] | 7 |
| 44..52 | ctrl(fieldEnd) (field_ranges, 컨트롤 아님) | run[1] | 7 |
| 52..81 | t" ‑ ① …보건복지부)"(29자) | run[2] | **8** ← (52,8) |
| 81 | ctrl(bookmark) + 경계 | run[2]/[3] | 9 ← (81,9) |

slot_count = (82−1−49)/8 − field_ranges(1) = **3**; 방출 슬롯 = [Field] (+ColumnDef 여부 무관)
→ mismatch. mismatch 경로의 결함 2종이 함께 발현:

1. **균형 field_ranges 의 fieldEnd 방출 코드 부재** (고아 fieldEnd 만 #1556 에서 복원)
   → fieldEnd 소실 → cc −8, 후속 경계 −8.
2. **북마크 hoist** (비-empty 문단, section.rs:466) → 문단 시작으로 이동 → 순서 파괴.
   경계 −16 = fieldBegin 슬롯 오배치(−8) + fieldEnd 소실(−8) 결합.

## 수정 설계 (Stage 2)

`src/serializer/hwpx/section.rs` 3점 수정:

1. **hidden 슬롯 정합** (핵심): else 분기에서
   `hidden = SectionDef 수 + 억제된 첫 ColumnDef(suppress_first_col 적용 시 1)` 산출.
   메인 경로 진입 조건을 `slot_count == slots.len() + hidden` 으로 확장하고
   `expected_utf16_pos` 초깃값을 `8 × hidden` 으로 설정 — hidden 슬롯(항상 컨트롤
   선두)의 자리를 위치 축에서 건너뛴다. 증거 불일치(합성 IR 등) 시 자연히 기존
   mismatch 경로로 폴백(자가 검증).
2. **북마크 in-order 통일**: 문단 시작 hoist(466) 제거, 메인 경로의 슬롯 방출
   지점(빈 문단 pre-loop·본문 루프·말미 루프)과 finish 직전에
   `emit_inorder_bookmarks` 호출 (mismatch 경로의 #1627 메커니즘 재사용 + 1라운드
   Stage 2 교정 편입). 메인 경로는 현재 북마크를 아예 방출하지 않으므로(hoist
   전담) 이 추가가 없으면 첫 문단 이동 시 북마크가 드롭된다.
3. **mismatch 폴백 보강**: 잔존 mismatch 경로에도 균형 `field_ranges` 의 fieldEnd
   말미 방출 추가 (#1556 고아 처리와 동형 — cc 보존).

### 수기 트레이스 검증

두 파일 모두 수정 후 방출 시퀀스가 원본 run 구조와 **정확히 일치**함을 수기
트레이스로 확인 (36384689: 템플릿[secPr,colPr] → cut16 없음 → tbl → cut24 →
run(25) 닫힘 → pageNum+bookmark → t"" / 36388711: fieldBegin@16 → 텍스트 →
fieldEnd@44 → cut52 → 텍스트+bookmark → cut81 → 빈 run(9)).

## 실파일 가드 편입 (본 단계 산출)

타깃 3건(공개 결재문서, 14~51KB)을 `samples/hwpx/opengov/` 동결 말뭉치에 추가하고
`tests/fixtures/opengov_snapshot.tsv` 에 **현재 결함 상태(IR_DIFF 1)로 동결**:

- Stage 2 수정이 PASS 로 개선하면 게이트가 개선 검출로 실패 → 스냅샷 승격 강제
  (게이트 설계 의도) — 수정 효과가 PR 에서 가시화된다.
- `cargo test --test opengov_corpus_snapshot` 2/2 통과 확인 (20건).

## 방향 결정 근거

- (a) 카운트-슬롯 정합(채택): 메인 경로(위치 정확)를 재사용, 첫 문단 전용 특례가
  아니라 hidden 슬롯 일반 규칙. 진입 조건이 cc 증거와 자가 검증되어 오진입 없음.
- (b) mismatch 경로 위치추정 개선(부분 채택): fieldEnd 방출 부재만 보강(폴백 안전망).
  mismatch 경로에 위치추정을 새로 짓는 것은 메인 경로 중복 구현이라 배제.

## 리스크 재평가

- 첫 문단은 지금까지 **전부 mismatch 경로**였으므로, 이번 수정으로 hidden 정합이
  성립하는 첫 문단 전수가 메인 경로로 이동한다 — 영향 범위가 넓어 Stage 3 통제
  비교(hwpdocs 전수)가 필수 게이트.
- RED 단위 테스트(합성)는 Stage 2 에서 수정과 함께 GREEN 으로 커밋(1라운드
  `task1591_bookmark_not_hoisted_before_slot` `#[ignore]` 해제 포함).
