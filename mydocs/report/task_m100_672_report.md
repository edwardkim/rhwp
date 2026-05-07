# Task #672 최종 결과 보고서

## 1. 요약

**Issue #672**: TAC 표 비례 축소 시 셀 콘텐츠 클립 — `common.height` vs measured `row_heights` 불일치

**결과**: 본 task 본질 영역 (TAC 표 비례 축소 메커니즘에서 작은 차이 ≤2% 면제) **정정 완료**. 회귀 0.

**잔존 결함 영역 (별개 본질)**: paragraph_layout 줄 위치 vs row_heights 정합 → **Issue #674** 별도 등록.

## 2. 본질 진단 (Stage 1)

### 2.1 TAC 표 비례 축소 발동 영역 sweep (187 fixture)

본 환경의 187 fixture 중 TAC 표 비례 축소 발동 영역 분포:

| 차이 비율 | 발생 케이스 | 영역 분류 |
|-----------|-------------|----------|
| 0~1% | 3 건 | 측정 오차 |
| **1~2%** | **4 건 (포함: 계획서.hwp 1.32% — 본 case)** | **작은 불일치 (정정 후보)** |
| 2~5% | 4 건 | 중간 영역 |
| 5~10% | 1 건 | |
| 10~20% | 6 건 | 의도적 큰 압축 |
| 20%+ | 4 건 | 의도적 큰 압축 |

### 2.2 한컴 권위 영역 분석

- 작은 차이 (1~2%) 영역: 한컴은 비례 축소 안 함 추정 (셀 콘텐츠 측정값과 common.height 의 미세 차이는 측정 오차)
- 큰 차이 (≥5%) 영역: 한컴도 비례 축소 발동 가능성 (TAC 표 본문 압축 의도)

본 case (`samples/계획서.hwp`) 1.32% 차이 → 한컴은 비례 축소 안 함 → 본 환경도 면제 정합.

## 3. 본질 정정 (Stage 2)

### 3.1 정정 위치

`src/renderer/height_measurer.rs:805-822` — 단일 분기 정정.

### 3.2 정정 내용

```rust
const TAC_SHRINK_THRESHOLD_RATIO: f64 = 0.02;
let shrink_threshold = (common_h * TAC_SHRINK_THRESHOLD_RATIO).max(1.0);
let table_height = if table.common.treat_as_char && common_h > 0.0
    && raw_table_height > common_h + shrink_threshold {
    let scale = common_h / raw_table_height;
    for h in &mut row_heights {
        *h *= scale;
    }
    common_h
} else {
    raw_table_height
};
```

### 3.3 면제 영역 (작은 차이)

```
계획서.hwp pi=0    | 1.32% (12.76 px) — 본 case
2010-01-06.hwp     | 1.97% (6.93 px)
hwp-img-001.hwp    | 1.22% (3.76 px)
kps-ai.hwp         | 1.88% (3.73 px)
hwp-3.0-HWPML.hwp  | 0.16% (1.09 px)
synam-001.hwp      | 0.38% / 0.94% (3.76 / 2.99 px)
```

5%+ 차이 (의도적 큰 압축 영역) 는 기존 동작 그대로 → **회귀 위험 좁힘**.

## 4. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 5. 광범위 페이지네이션 회귀 sweep (Stage 3)

`samples/` 폴더 전체 187 fixture 페이지 수 BEFORE/AFTER 비교:

| 영역 | 결과 |
|------|------|
| BEFORE (task671) | 187 fixtures / **2013 pages** |
| AFTER (task672) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

**회귀 0** — 본 task 정정이 다른 fixture 페이지네이션에 영향 없음.

## 6. 시각 판정 게이트웨이

`samples/계획서.hwp` 1페이지 표 (PNG 시각 확인):

| 영역 | BEFORE (task671) | AFTER (task672) |
|------|-----------------|-----------------|
| 셀 [21] r=5 row_heights[5] | 66.88 (비례 축소) | **67.76 (측정값 보존)** ✅ |
| layout cell_h | 66.88 | **67.76** ✅ |
| SVG 줄 baseline 그려짐 | 3줄 (그러나 마지막 줄 시각 클립) | **3줄 (정상 그려짐)** ✅ |
| 다른 셀 영역 회귀 | — | 0 ✅ |

본 task 의 본질 영역 (TAC 표 비례 축소 면제) **정정 완료**.

### 잔존 결함 (별도 Issue)

본 task 정정 후에도 PNG 변환에서 셀 [21] / [52] 마지막 줄/paragraph 미표시:

- 진단: paragraph_layout 줄 layout 위치 (`col_area.y + line_idx * line_height`) 가 row_heights 와 무관 → cell BoundingBox 확장만으로 시각 클립 해결 안 됨
- descent 여유 추가 시도: `max_fs * 0.5` 에서 마지막 줄 일부 보임 (행 침범 시각 회귀)

→ paragraph_layout 줄 위치 결정 로직 자체의 본질 영역으로 분리:

**Issue #674**: paragraph_layout 줄 위치 vs row_heights 정합 — line_segs 부재 paragraph 마지막 줄 시각 클립 ([링크](https://github.com/edwardkim/rhwp/issues/674))

`feedback_hancom_compat_specific_over_general` + 회귀 위험 좁힘 영역 정합.

## 7. 코드 변경 사항 정리

### 단일 분기 정정

`src/renderer/height_measurer.rs:805-822`:
- TAC 표 비례 축소 임계값 가드 추가 (`TAC_SHRINK_THRESHOLD_RATIO = 0.02`)
- 작은 차이 (≤2%) 면제 → row_heights 측정값 보존
- 큰 차이 (≥2%) 비례 축소 발동 (기존 동작)

### 회귀 위험 영역 좁힘 원칙

- 단일 분기 정정 — 다른 영역 무영향
- 임계값 명시 — 사용자 의도 영역 (큰 압축) 보존
- 광범위 sweep 회귀 0 입증
- `feedback_rule_not_heuristic` + `feedback_hancom_compat_specific_over_general` 정합

## 8. 권위 자료

- `samples/계획서.hwp` — 본 task 권위 재현 영역 (Task #671 에서 git tracked 등록 영역 공유)

## 9. 진단 도구

- `examples/inspect_task672.rs` — TAC 표 비례 축소 발동 영역 sweep 도구 (187 fixture 분포 분석)

## 10. 최종 산출물

| 영역 | 파일 |
|------|------|
| 코드 정정 | `src/renderer/height_measurer.rs` (단일 분기) |
| 진단 도구 | `examples/inspect_task672.rs` |
| 수행계획서 | `mydocs/plans/task_m100_672.md` |
| 구현계획서 | `mydocs/plans/task_m100_672_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_672_stage1.md`, `_stage2.md`, `_stage3.md` |
| 최종 보고서 | `mydocs/report/task_m100_672_report.md` (본 문서) |

## 11. 의존성

- **선행 의존**: Task #671 정정 코드 (`local/task671` 브랜치) — 본 task 는 그 위에서 분기
- **후행 의존**: Issue #674 (잔존 결함, 별개 본질)

## 12. 정합 패턴 정리

- `feedback_rule_not_heuristic`: 임계값 본질 영역 명시 (사용자 의도 영역 보존)
- `feedback_hancom_compat_specific_over_general`: 작은 차이만 정정 (한컴 정합)
- `feedback_visual_judgment_authority`: 시각 판정 결과 → 잔존 결함 분리 결정
- `feedback_close_issue_verify_merged`: Issue close 시 정정 코드 devel 머지 검증
- `project_dtp_identity`: 조판 엔진 정합성 강화

## 13. 후속 영역

1. **Issue #674 정정**: paragraph_layout 줄 위치 vs row_heights 정합 (별도 task)
2. 후속 task 영역 — 한컴 권위 동작 추가 분석 (TAC 표 비례 축소 임계값의 정확한 한컴 동작 매핑)
