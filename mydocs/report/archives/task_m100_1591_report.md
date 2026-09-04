# Task #1591 — 최종 결과보고서 (불채택 + 근본원인 문서화)

**제목**: HWPX para0 char_shape +8 시프트 (Class C1)
**마일스톤**: M100 (v1.0.0) · **이슈**: edwardkim/rhwp#1591 · **브랜치**: `local/task1591`
**결과**: **불채택**(순효과 0, 채택 게이트 미달). 진짜 근본원인 문서화 + 회귀 repro 보존.

---

## 1. 경위

fidelity 잔여 IR_DIFF Class C(3건). Stage 1 조사에서 para0 북마크 hoisting(section.rs:416-426)을
char_shape +8 의 원인으로 지목 → Stage 2 에서 북마크를 슬롯 시스템에 편입(hoisting 제거).

## 2. Stage 2 결과 — 부분 교정, 게이트 미해소

- **컨트롤 순서는 교정**: rt 가 `[…Table, PageNum, Bookmark(끝)]` 로 정렬(hoisting 제거 성공).
- **그러나 char_shape 는 여전히 +8**(pos 24→32) → 타깃 IR_DIFF 미해소.
- 통제 비교(fidelity12→13, 공통 10150): **개선 0 / 회귀 0 / 순효과 0**.

## 3. 진짜 근본원인 (재규명)

char_shape +8 은 북마크 hoist 와 **독립적인 first-para mismatch-path 위치추정 결함**:

- para0 는 **첫 문단** → #1584 ColumnDef 템플릿 흡수 적용 → 첫 ColumnDef 가 `slots` 에서 제외.
- `inferred_control_slot_count`(=4, cc 기반) ≠ `slots.len()`(=3, ColumnDef 제외) → **mismatch 경로**.
- mismatch 경로는 슬롯의 실제 char-offset 위치를 추정 못 해 char_shape 경계를 +8 오배치.
- 이 +8 은 **#1584 이전·북마크 수정 전후 모두 불변** → 북마크와 무관한 선존 mismatch-path 결함.

→ 북마크 hoist 는 **게이트 비가시의 또 다른(실재) 버그**였고, Class C1 의 char_shape 타깃은
별개의 first-para mismatch-path 위치추정(슬롯카운트 vs 슬롯 불일치) 결함. **F3(#1561)급 난이도.**

## 4. 판단 — 불채택

북마크 슬롯 편입은 올바른 교정(순서 보존·회귀 0)이나 타깃 IR_DIFF 미해소·**순효과 0**으로
채택 기준(순효과>0) 미달. 작업지시자 결정에 따라 **Stage 2 롤백**:

- `section.rs`/`roundtrip.rs` Stage 2 직전 상태로 복원(hoisting 유지).
- RED 테스트 `task1591_bookmark_not_hoisted_before_slot` 는 `#[ignore]`(hoist 버그 repro 보존).

## 5. Class C 분해 (남은 과제)

| 파일 | 근본 | 상태 |
|------|------|------|
| 36384689·36385445 | **first-para mismatch-path char_shape 위치추정** (북마크 아님) | 미해결(F3급) |
| 36388711 | Field ClickHere (−16/−8) | 별개(C2) |

## 6. 권고 (후속)

1. **mismatch-path char_shape 위치추정** — slot_count(cc 기반) vs slots(템플릿 ColumnDef 제외)
   불일치가 first-para 다중컨트롤에서 char_shape 를 오배치. F3 와 동질의 슬롯 위치 정합 영역 →
   별 이슈 + 광역 통제 비교 필수. (난이도 높음, 우선순위 판단 필요.)
2. **북마크 hoist** — 게이트 비가시지만 실재하는 순서 오류. 위 1 과 함께 또는 별도 처리 가능.
3. **C2(36388711 Field)** — 별 이슈로 분리.

## 7. 교훈

Stage 1 조사가 표면 증상(북마크 재배치)을 근본으로 오인. **통제 비교(순효과)가 부분 수정의
게이트 무효과를 정확히 검출** — IR diff 상세 추적만으로는 놓칠 다층 원인을 통제 비교가 가려냄.
무손실 게이트의 채택 기준은 통제 비교 순효과이며, 부분 교정은 채택하지 않는다.
