# 최종 결과 보고서 — Task #554

> **이슈**: [#554](https://github.com/edwardkim/rhwp/issues/554)
> **마일스톤**: M100 (v1.0.0 조판 엔진)
> **브랜치**: `local/task554`
> **작성일**: 2026-05-03

---

## 1. 작업 개요

**문제**: HWP3 → HWP5/HWPX 변환본을 rhwp 가 파싱하면 페이지 수가 한컴 정답 대비 +1 ~ +4 증가했다.

**본질** (Stage 1 진단):
- HWP3 파서는 `src/parser/hwp3/mod.rs:1833` 에서 `margin_bottom -= 1600 HU` 보정으로 한글97 의 "마지막 줄 tolerance" 동작을 모방
- HWP5/HWPX 파서/페이지네이션에는 이 보정이 누락 → 한컴이 변환 시 같은 동작을 적용하지 못함
- 한컴이 HWPX XML 에 `bottom="4252"` (15.0mm) 그대로 인코딩하지만 한글97 은 실효 9.4mm 처럼 동작

**해결**: 변환본 식별 휴리스틱 + 조건부 `margin_bottom -= 1600 HU` 보정.

## 2. 진단 결과 (Stage 1)

### 2.1 방안 비교

| 방안 | 평가 |
|------|------|
| A — 단순 -1600 적용 (HWP5/HWPX 모든 fixture) | 광범위 회귀 (-1/-1/-5) |
| B — 공통 페이지네이션 tolerance | typeset.rs 핵심 수정, Task #546 패턴 위험 |
| **C — 한컴 변환본 식별 + 조건부 보정** | 채택 |
| D — HWP3 보정 제거 + 재평가 | 페이지 폭증 |

### 2.2 식별 휴리스틱

| 포맷 | 휴리스틱 | 검증 |
|------|---------|------|
| HWPX | `<hh:head version="1.4">` | 6/6 fixture 100% |
| HWP5 | `PS/Para < 0.05 AND CS/Para < 0.15 AND Para > 50` | 27/27 fixture 100% |

## 3. 구현 (Stage 2)

### 3.1 변경 파일

| 파일 | 변경 | 설명 |
|------|------|------|
| `src/parser/hwpx/header.rs` | +28 LOC | `parse_hwpx_hwpml_version()` 신규 |
| `src/parser/hwpx/mod.rs` | +13 LOC | hwpml="1.4" 식별 + post-process 보정 |
| `src/parser/mod.rs` | +37 LOC | `apply_hwp3_origin_fixup()` (PS/CS 휴리스틱) + 두 진입점 호출 |
| `src/main.rs` | +21 LOC | `info` 명령에 Origin 추정 정보 추가 |
| `tests/issue_554.rs` | +120 LOC | 회귀 테스트 12건 |

### 3.2 핵심 로직

**HWPX** (`src/parser/hwpx/mod.rs`):

```rust
let hwpml_version = header::parse_hwpx_hwpml_version(&header_xml);
let is_hwp3_origin = hwpml_version.as_deref() == Some("1.4");
// ... 섹션 파싱 후
if is_hwp3_origin {
    for section in sections.iter_mut() {
        section.section_def.page_def.margin_bottom =
            section.section_def.page_def.margin_bottom.saturating_sub(1600);
    }
}
```

**HWP5** (`src/parser/mod.rs`):

```rust
fn apply_hwp3_origin_fixup(doc: &mut Document) {
    let total_paragraphs: usize = doc.sections.iter()
        .map(|s| s.paragraphs.len())
        .sum();
    if total_paragraphs <= 50 {
        return;
    }
    let ps_ratio = doc.doc_info.para_shapes.len() as f64 / total_paragraphs as f64;
    let cs_ratio = doc.doc_info.char_shapes.len() as f64 / total_paragraphs as f64;
    if ps_ratio < 0.05 && cs_ratio < 0.15 {
        for section in doc.sections.iter_mut() {
            section.section_def.page_def.margin_bottom =
                section.section_def.page_def.margin_bottom.saturating_sub(1600);
        }
    }
}
```

## 4. 검증 결과

### 4.1 자동 테스트

| 검사 | 결과 |
|------|------|
| `cargo test --lib` | **1113 passed** (회귀 0) |
| `cargo test --test issue_554` | **12 passed** (신규) |
| `cargo test --test svg_snapshot` | 6/6 passed |
| `cargo test --test issue_546` | passed (Task #546 정합) |
| `cargo test --test issue_418/501/505/530` | 모두 passed |
| `cargo clippy --lib -- -D warnings` | **0건** |

### 4.2 페이지 수 정합 (한컴 정답 비교)

| 파일 | 한컴 정답 | 변경 전 | 변경 후 | 평가 |
|------|----------|---------|---------|------|
| hwp3-sample.hwp (HWP3) | 16 | 16 | 16 | ✅ 회귀 0 |
| hwp3-sample-hwp5.hwp (HWP5) | 16 | 17 | 15 | ⚠️ -1 잔존 |
| hwp3-sample-hwpx.hwpx (HWPX) | 16 | 17 | 15 | ⚠️ -1 잔존 |
| hwp3-sample4.hwp (HWP3) | 36 | 39 | 39 | ⚠️ HWP3 자체 회귀 (Task #554 범위 밖) |
| hwp3-sample4-hwp5.hwp (HWP5) | 36 | 38 | **36** | ✅ 정답 |
| hwp3-sample5.hwp (HWP3) | 64 | 64 | 64 | ✅ 회귀 0 |
| hwp3-sample5-hwp5.hwp (HWP5) | 64 | 68 | **64** | ✅ 정답 |
| hwp3-sample5-hwpx.hwpx (HWPX) | 64 | 68 | **64** | ✅ 정답 |
| hwp-3.0-HWPML.hwp | 122 | 122 | 122 | ✅ 휴리스틱 false positive 회피 |

### 4.3 광범위 회귀 0

80+ fixture 광범위 sweep 결과 변화 없음. 특히:
- exam_science.hwp: 4 페이지 (Task #546 정합 유지)
- 2022년 국립국어원 업무계획.hwp: 40 페이지 (방안 A 단순 적용 시 -5 회귀였던 케이스)
- exam_kor.hwp: 20 페이지 (PS/Para=0.076 임계값 근처에서 안전 분리)

## 5. 잔존 사항

### 5.1 의도된 trade-off (별도 task 권고)

**hwp3-sample 변환본 -1 over-correct**:
- 정답 16 vs 결과 15 (HWP5/HWPX 모두)
- **본질적 한계**: 페이지 break 알고리즘이 줄 단위로 결정되어 단일 -1600 HU 보정이 모든 페이지에서 동일 줄 수를 흡수. sample/sample4/sample5 가 -1/-2/-4 만큼 줄어들어야 하는데 단일 보정은 모두 동일 줄 수
- Stage 2-2 보정값 sweep (-400 ~ -1600) 결과 모두 동일 효과 입증
- **해결 방향**: typeset.rs 페이지 break 알고리즘 정밀화 (마지막 페이지에서만 잔여 줄 흡수). Task #546 패턴 위험 高 → 신중한 별도 task 필요

### 5.2 Task #554 범위 밖

**hwp3-sample4.hwp HWP3 자체 회귀**:
- HWP3 39 페이지 vs 정답 36 페이지 (+3)
- HWP3 파서 또는 페이지네이션 별도 결함
- 본 task 와 무관, 별도 issue 등록 권고

## 6. 사용자 인터페이스 개선

`rhwp info <파일.hwp>` 명령에 Origin 추정 정보 추가:

```
페이지 수: 36
ParaShape: 24 (PS/문단 = 0.019)
CharShape: 21 (CS/문단 = 0.016)
Origin 추정: HWP3 변환본 추정 (margin_bottom -1600 HU 보정 적용)
```

## 7. 작업 단계 요약

| Stage | 산출물 | 상태 |
|-------|--------|------|
| 0 | 사전 진단 (read-only) | 완료 |
| 0.5 | 수행계획서 (`task_m100_554.md`) | 완료, 승인 |
| 1 | 광범위 진단 보고서 (`stage1.md`) | 완료, 승인 |
| 2-0 | 구현계획서 (`task_m100_554_impl.md`) | 완료, 승인 |
| 2-1 | HWPX 휴리스틱 + 보정 (`stage2.md`) | 완료, 승인 |
| 2-2 | HWP5 휴리스틱 + 보정 (`stage3.md`) | 완료, 승인 |
| 2-3 | 회귀 검증 + tests/issue_554.rs (`stage4.md`) | 완료, 승인 |
| 2-4 | 정리 + 최종 보고서 (본 문서) | 완료 |

## 8. 후속 권고

1. **별도 task A**: typeset.rs 페이지 break 알고리즘에 한글97 마지막 줄 tolerance 동작 정밀 구현 → hwp3-sample 변환본 -1 over-correct 정정
2. **별도 task B**: hwp3-sample4.hwp HWP3 자체 회귀 (+3 페이지) 별도 issue 등록
3. **휴리스틱 정합 추적**: 향후 새 HWPX 변환본이 hwpml 1.4 가 아닌 다른 버전으로 들어올 가능성 → 신규 fixture 추가 시 회귀 테스트로 자동 추적

## 9. 작업지시자 승인 요청

본 최종 보고서 + Task #554 작업 전체에 대해 다음을 요청합니다:

1. **Task #554 완료 승인**
2. **GitHub Issue #554 close 승인** (CLAUDE.md: 이슈 클로즈는 작업지시자 승인 후)
3. **PR 생성 진행** (`local/task554` → `upstream/devel`):
   - PR #553 (Task #511) 머지 후 진행 (sample fixture 의존)
   - 또는 PR #553 의존성 정리 후 PR 생성
4. **잔존 사항 별도 task 등록 진행** (hwp3-sample -1 over-correct, hwp3-sample4 HWP3 회귀)
