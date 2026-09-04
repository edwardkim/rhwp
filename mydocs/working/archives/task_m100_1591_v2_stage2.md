# Task #1591 v2 2단계 완료 보고 — hidden 슬롯 정합 + 북마크 in-order + fieldEnd 복원

## 구현 내용 (`src/serializer/hwpx/section.rs`)

### 1. hidden 슬롯 정합 (핵심 — Class C1/C2 공통 근원 교정)

슬롯 선택 else 분기에서 제외되는 hidden 후보(SectionDef·템플릿 흡수 첫 ColumnDef)의
인덱스를 수집하고, **cc 증거가 hidden 의 8유닛 점유를 보여주면
(`slot_count == slots.len() + hidden.len()`) hidden 을 위치 슬롯으로 편입**한다:

- SectionDef: `render_control_slot` 에 방출 arm 이 없어(`_ => {}`) XML 0바이트,
  위치 축만 +8 전진.
- 첫 ColumnDef: consume-once 플래그(`body_coldef_template_pending`)를 유지한 채
  슬롯에 남겨 XML 방출만 1회 억제 — `slot_count == controls.len()` 첫 분기와 동형.
- 결과: 첫 문단이 mismatch 폴백 대신 **메인 경로(UTF-16 위치 축)** 로 진입해 후위
  슬롯(pageNum 등)·fieldEnd 가 char-offset 위치에 방출된다.
- 증거 불일치(합성 IR, hidden 이 cc 를 점유하지 않는 문서)는 종전 동작 그대로
  (자가 검증 — 오진입 없음).

### 2. 북마크 in-order 통일 (1라운드 교정 편입)

- `bm_inorder = text.is_empty() || !slots.is_empty()` — 슬롯 있는 비-empty 문단도
  hoist 대신 컨트롤 순서(in-order) 방출. 비-empty·무슬롯 문단은 종전(문단 시작) 유지.
- 메인 경로에 `emit_inorder_bookmarks` 방출 지점 추가(빈 문단 pre-loop·본문 루프·
  말미 루프 + trailing) — 메인 경로는 종전 hoist 전담이라 방출 지점이 없었고, 이
  추가가 없으면 첫 문단 이동 시 북마크가 드롭된다.
- 1라운드 RED `task1591_bookmark_not_hoisted_before_slot` **`#[ignore]` 해제 → GREEN**.

### 3. mismatch 폴백 보강 (#1593)

잔존 mismatch 경로에 균형 `field_ranges` 의 닫는 fieldEnd 말미 복원 추가
(#1556 고아 처리와 동형) — fieldBegin(슬롯)만 방출되고 fieldEnd 가 소실되던
cc −8 결함의 안전망.

## 신규/전환 테스트

| 테스트 | 종류 | 검증 |
|--------|------|------|
| `task1591v2_first_para_hidden_slot_char_shape_position` | 합성 (36384689 동형) | char_shape 경계 24 보존(+8 회귀 가드) + 컨트롤 순서 |
| `task1593_first_para_same_para_field_end_preserved` | 합성 (36388711 동형) | field_ranges 1/1·cc 35 보존(fieldEnd 드롭 가드) + 경계 −8 가드 |
| `task1591_bookmark_not_hoisted_before_slot` | 1라운드 RED → GREEN | 표 뒤 북마크 순서 보존 |
| opengov 스냅샷 3건 | 실파일 | **IR_DIFF 1 → PASS 0 승격** (36384689·36385445·36388711) |

## 게이트 결과

| 게이트 | 결과 |
|--------|------|
| 타깃 3건 `hwpx-roundtrip` | **전부 PASS (diff=0, r2=0)** — 수용 기준 1·2 충족 ✅ |
| `opengov_corpus_snapshot` (20건) | 통과 (3건 PASS 승격 반영) ✅ |
| `hwpx_roundtrip_baseline` | 통과 (구조 보존 전수) ✅ |
| `cargo test --lib` | 2054+ passed / 0 failed (신규 2건·해제 1건 포함) ✅ |

## 다음 단계

3단계 — hwpdocs 전수 통제 비교 (채택 게이트: 개선−회귀>0, 악화 0):
- BEFORE: stage-1 커밋 worktree 바이너리로 전수 배치
- AFTER: 본 수정 바이너리로 전수 배치
- 파일별 status 대조 → 개선/회귀/악화 집계
