# Task #896 최종 결과 보고서 — sample16 페이지 18 추가 시각 정합

**이슈**: [edwardkim/rhwp#896](https://github.com/edwardkim/rhwp/issues/896)
**브랜치**: `local/task896` (base: `local/task894 @ ce8d3ce`)
**선행 task**: #894 (PR #897), #877 (PR #890)
**기간**: 2026-05-14

## 1. 개요

Task #894 의 Stage 4 분리 task. sample16 페이지 18 의 추가 시각 정합 처리.

## 2. 최종 결과

### 2.1 성과 요약

| Stage | 항목 | 결과 |
|-------|------|------|
| 1+2 | paragraph 398/399 ◦ 잘못 추가 → dash skip | ✅ 한컴 정합 |
| 3+4 | WMF font name 인코딩 + Korean fallback chain | ✅ 한글 글리프 표시 |
| 5/6 | HWP5/HWPX 페이지 수 inflate (skip — CLI 정합 / WASM 환경 별도) | ⏭️ skip |
| **7** | **HWPX `<hp:pageBorderFill>` 파싱 — upstream 4beb6b07 (Task #888) 가 동일 영역 별도 처리 → 본 task 에서 drop** | ⏭️ skip |
| 9 | WMF text positioning (한컴 사적 unit scale) → 별도 task #902 분리 | ⏭️ #902 |

## 3. Stage 별 결과

### 3.1 Stage 1+2 — paragraph 398/399 ◦ 정합

#### Fix

`src/parser/hwp3/mod.rs` 의 `apply_bullet_fixup_single` 에 dash skip 추가:

```rust
let first_non_space = para.text.chars().find(|c| *c != ' ').unwrap_or(' ');
if first_non_space == '-' { return; }
```

#### 결과

| paragraph | 이전 | 이후 |
|-----------|------|------|
| 397 | ` ◦  공사 주요업무...` | 유지 |
| 398 | ` ◦    - 하드웨어...` | `    - 하드웨어...` ✅ |
| 399 | ` ◦    - ORACLE...` | `    - ORACLE...` ✅ |

PDF 정합 분석: 한컴 viewer 도 paragraph 398/399 의 ◦ 표시 안 함 (sub-item dash marker).

상세: [`mydocs/working/task_m100_896_stage1.md`](../working/task_m100_896_stage1.md)

### 3.2 Stage 3+4 — WMF font encoding ⭐

#### Fix 1 — `font.rs` charset 기반 primary

```rust
if charset != CharacterSet::ANSI_CHARSET {
    (as_charset, as_latin1)  // multi-byte → as_charset primary
} else {
    (as_latin1, as_charset)
}
```

#### Fix 2 — `util.rs` 한국어 fallback chain

```rust
if has_korean {
    font_family.extend([
        "Apple SD Gothic Neo", "Malgun Gothic", "Nanum Gothic",
        "Noto Sans CJK KR", "sans-serif",
    ]);
}
```

#### 결과

| 항목 | 이전 | 이후 |
|------|------|------|
| SVG font-family primary | `'±¼¸²Ã¼'` (깨진) | `'굴림체'` ✅ |
| 시스템 fallback chain | 없음 | Apple SD / Malgun / Nanum / Noto CJK |
| WMF 그림 안 한글 시각 | 모자이크 깨짐 | **정상 표시** ✅ |

상세: [`mydocs/working/task_m100_896_stage3.md`](../working/task_m100_896_stage3.md)

### 3.3 Stage 7 — HWPX 외곽선 (drop)

본 task 진행 중 HWPX `<hp:pageBorderFill>` 파싱 추가 시도했으나 upstream/devel 의 신 commit `4beb6b07` (Task #888 hwpx hwp save) 가 동일 영역 별도로 처리. 중복 → 본 task 에서 drop.

PR #904 conflict 발생 → close → 본 PR 재구성 (Stage 7 commit 제외).

### 3.4 Stage 9 — WMF text positioning (별도 task #902 분리)

WMF binary 분석 결과 한컴 사적 WMF unit scale (SETWINDOWEXT(56,72) + SETVIEWPORTEXT 미호출) 의 비표준 비례 → Task #860 의 viewBox 자동 확장 영향. Fix 영역 매우 큼 + 회귀 위험 매우 높음.

→ **별도 task [#902](https://github.com/edwardkim/rhwp/issues/902)** 분리.

## 4. 검증

### 4.1 cargo test

```
cargo test --release --all-targets: 1398 passed, 0 failed
```

### 4.2 sample 페이지 수 회귀

- HWP3 sample 6종 (sample/4/5/10/13/14/16) 동일
- HWPX sample 동일

### 4.3 시각 정합

- sample16 paragraph 398/399 ◦ 표시 제거 ✅ (한컴 정합)
- paragraph 394 WMF 그림 안 한글 글리프 정상 표시 ✅

## 5. 변경 파일

### 5.1 소스

- `src/parser/hwp3/mod.rs` (+8) — Stage 1+2
- `src/wmf/parser/objects/graphics/font.rs` (+11) — Stage 3+4
- `src/wmf/converter/svg/util.rs` (+18, -2) — Stage 3+4

### 5.2 문서

- `mydocs/plans/task_m100_896.md`
- `mydocs/plans/task_m100_896_impl.md`
- `mydocs/working/task_m100_896_stage1.md`
- `mydocs/working/task_m100_896_stage3.md`
- `mydocs/report/task_m100_896_report.md` (본 파일)

## 6. 분리 task

| Issue | 내용 |
|-------|------|
| [#902](https://github.com/edwardkim/rhwp/issues/902) | WMF unit scale 정합 — 한컴 사적 WMF SetWindowExt 비표준 비례 |

## 7. 결론

본 task #896 의 핵심 성과:
- ✅ paragraph 398/399 ◦ 정합
- ✅ WMF 한글 글리프 표시
- ⏭️ HWPX 외곽선 — upstream 4beb6b07 가 별도 처리 (drop)
- ⏭️ WMF text positioning → #902 분리

cargo test 1398 passed + sample 회귀 없음.
