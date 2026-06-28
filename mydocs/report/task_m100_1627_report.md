# 최종 결과보고서 — Task #1627 (HWPX roundtrip: bookmark in-order 방출 + char_shape IR_DIFF 분석)

**제목**: HWPX roundtrip 첫 문단 char_shapes 오프셋 shift (4건) 조사 및 bookmark 위치 보존
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1627 · **브랜치**: `local/task1627` (base: `upstream/devel`)

## 1. 대상
HWPX parse→serialize→reparse char_shapes 오프셋 불일치 4건(18,387 중, 증가에도 불변):
36384689·36385445(+8) · 36388711(−16/−8) · 36399822(표 셀). 전부 **빈-text(객체-only) 문단**.

## 2. 근원 (진단 `examples/diag_1627.rs`)
대표 36384689 p[0](text 비어있음)에서 두 증상 확인:
1. **컨트롤 재정렬**: parse `[SectionDef,ColumnDef,Table,PageNumberPos,Bookmark]` →
   reparse `[…,Bookmark,Table,PageNumberPos]`. 직렬화기가 bookmark 를 문단 시작으로 강제
   (`section.rs`: "IR 에 위치 정보 없음")해 원본 순서 깨짐.
2. **char_shape +8**: parse `(24,10)` → reparse `(32,10)`. slot 카운트 회계 차(parse 3 slot=24
   vs reparse 4 slot=32) — `inferred_control_slot_count` 와 실제 slot 방출의 mismatch("위치
   추정 불가") 경로에서 발생.

## 3. 수정 — bookmark in-order 방출 (저위험, 옵션 2)
빈-text 문단의 bookmark 를 문단 시작 강제 대신 **para.controls 순서대로 slot 사이에 in-order
방출**(zero-width 라 char-position 불변). 비-empty 문단은 종전(문단 시작) 유지 — slot
char-position 정밀 경로 보호.
- `src/serializer/hwpx/section.rs`: slot 수집에 `slot_ctrl_indices` 병행, `emit_inorder_bookmarks`
  헬퍼로 mismatch 경로에서 slot 사이 interleave. empty-text 한정.

**결과**: **컨트롤 순서 round-trip 해소**(parse==reparse 순서 일치). 단위 테스트
`task1627_empty_para_bookmark_serialized_after_preceding_table` 가드.

## 4. char_shape IR_DIFF — 근본 원인 완전 규명 + 보류 (작업지시자 결정)

### 완전 규명된 근본 원인
char_count 는 parse=reparse **동일(33)** — 문자/내용 손실 0. 차이는 char_shape **경계 위치만**
(parse 24 = 3 slot vs reparse 32 = 4 slot, 순수 cosmetic·렌더 무영향).

근원: `is_hwpx_inline_slot`(section.rs:755)이 **SectionDef·ColumnDef 를 slot 에서 제외**하나,
HWPX char_count 는 이들의 `secPr`/`colPr` 를 **8유닛 위치로 집계**한다. 객체-only 첫 문단
(secPr+colPr+표+pagenum)에서 `inferred_control_slot_count`(char_count 기반)=4 ≠
`slots.len()`(inline-only)=2 → **mismatch "위치 추정 불가" 경로** → char_shape 경계 부정확(+8).

### 수정에 필요한 변경 (고위험, 보류 사유)
precise(main) 경로 진입에 SectionDef/ColumnDef 를 위치-점유 slot 으로 라우팅 필요:
- `is_hwpx_inline_slot`/slot 수집에 포함 + `render_control_slot` 에 secPr/colPr arm 추가.
- **그러나 secPr/colPr 는 섹션 템플릿/secPr 작성기로 이미 방출 → 이중 방출 위험.**
- **#1584 의 ColumnDef 템플릿 흡수 억제 로직과 정면 충돌**(첫 ColumnDef 를 slot 에서 빼는
  delicate 처리 존재). 전 코퍼스 HWPX roundtrip(99.98% PASS) 위협.

### 판정 (작업지시자 결정: 보류)
char_count 보존(내용 손실 0)인 cosmetic char_shape 경계 차 4건을 위해 secPr/colPr slot
라우팅 + #1584 재작업의 corpus-wide 고위험 변경은 비례하지 않음. **알려진 한계로 보존.**
근본 원인이 완전 규명됐으므로, 향후 secPr/colPr 의 char-position 모델을 정합하는 저위험
재설계(예: slot 추상화에 zero-emit position-slot 도입) 착수 시 본 분석을 출발점으로 사용.
(IR_DIFF 4건 잔존 = 빈-text 문단 char_shape 경계.)

## 5. 검증
| 게이트 | 결과 |
|--------|------|
| bookmark in-order 단위 테스트 | RED(시작 강제)→**GREEN** |
| `cargo test --lib` | 1975 passed / 0 failed |
| hwpx_roundtrip baseline | 4 passed |
| clippy / fmt | 무경고 / 0 diff |

## 6. 산출물
- 소스: `src/serializer/hwpx/section.rs` (bookmark in-order, empty-text 한정)
- 테스트: `task1627_empty_para_bookmark_serialized_after_preceding_table`
- 진단: `examples/diag_1627.rs`
- 보류: char_shape IR_DIFF (slot 회계 고위험)
