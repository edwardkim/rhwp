# Task #688 최종 결과보고서

## 결함 요약

`samples/table-vpos-01.hwpx` 5쪽 마지막 큰 표 (pi=34, "정부혁신 4대 추진전략 / 12대 추진과제") 가 SVG 출력에서 **시각적으로 거의 전부 누락**됨.

PDF 권위본 (`pdf/table-vpos-01-2022.pdf`) 5쪽: 큰 박스 + 헤더 + 4그룹 (참여소통/기본사회/공직혁신/공공 AX) × 3 = 12 추진과제 그리드.

## 근본 원인 (두 곳에 동일 결함)

외부 1×1 표 셀 안에 nested 표 2개 (1×1 헤더 + 11×3 그리드) 가 있는 구조 처리 시:

**Bug A** — `src/renderer/layout/table_layout.rs::layout_table()` 라인 150-168:
**Bug B** — `src/renderer/height_measurer.rs::measure_table_impl()` 라인 457-468:

두 곳 모두 1×1 래퍼 표 unwrap 로직이 다음 형태:
```rust
if let Some(nested) = cell.paragraphs.iter()
    .flat_map(|p| p.controls.iter())
    .find_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
```

`flat_map` 으로 셀 paragraphs 전체를 훑어 첫 nested 표만 가져옴. paragraphs 가 2개 이상이거나 다른 control 이 섞인 경우 **두 번째 paragraph 의 nested 표가 통째 누락**.

### 결과 (수정 전)

- layout_table: 외부 표가 첫 nested 1×1 헤더로 unwrap → 11×3 그리드 누락
- measure_table: 외부 표 측정값이 nested 1×1 헤더 height (57.72px) 로 잡힘 → measured_table 이 작게 잡혀 외부 표 cell-clip 이 57.72px 로 그려짐

두 결함이 결합되어 페이지 5 의 nested 11×3 그리드 (4그룹 + 12 추진과제) 가:
- layout 단계에서 그려지지 않거나
- 그려져도 외부 셀 cell-clip(y=227.17, h=57.72) 밖(y=295~)이라 SVG 클리핑으로 가려짐

## 수정

두 곳 모두 unwrap 조건을 다음 4가지 모두 충족하는 경우로 좁힘:
1. 외부 표 1×1 단일 셀 (현행)
2. **셀 paragraphs 가 정확히 1개**
3. **그 paragraph 의 control 이 정확히 1개의 nested table 만**
4. visible text 없음 (현행)

| Commit | 파일 | 라인 변경 |
|--------|------|----------|
| `4b357394` | `src/renderer/layout/table_layout.rs` | +15 / -7 |
| `7d8cca27` | `src/renderer/height_measurer.rs` | +13 / -9 |

## 검증 결과

### 시각 정합 (DoD 1~4)

`rsvg-convert` 로 SVG → PNG 변환:

| 시점 | 페이지 5 PNG 크기 | nested 11×3 그리드 표시 |
|------|------------------|------------------------|
| 결함 (수정 전) | (PDF 권위본 비교 필요) | ❌ 누락 |
| Stage 1 만 | 26 KB | ❌ 클립으로 가려짐 |
| Stage 1 + Stage 3 fix | **142 KB** | ✅ 완전 표시 |

PDF 권위본과 시각적으로 동등.

### 광범위 회귀 (DoD 5)

`scripts/svg_regression_diff.sh` + `/tmp/regr_full.sh`:

```
TOTAL: pages=1502 same=1499 diff=3 samples=159
diff list:
  exam_social__hwp/exam_social_001.svg     (자연 해소)
  table-vpos-01__hwp/table-vpos-01_005.svg (의도된 수정)
  table-vpos-01__hwpx/table-vpos-01_005.svg (의도된 수정)
```

회귀 0건. exam_social.hwp 페이지 1 의 변경은 동일 결함이 1×1 셀 (paras=3, "뜨거워진 한반도..." 텍스트) 에도 영향 → 수정 후 외부 표 정상 렌더 (cell-clip 통합 + width 411.92 정상화).

### 단위 테스트 (DoD 6)

```
test result: ok. 1119 passed; 0 failed; 1 ignored
... (총 1192+ 테스트 전부 통과)
```

SVG 스냅샷 테스트 6건 포함 모두 통과.

## 보조 관찰 (별개 결함)

본 타스크 수정과 **무관**한 별개 결함:

- **페이지 2 hwp_used diff = -791.9px**
- **페이지 3 hwp_used diff = -1658.3px**

수정 전후 변화 없음. 권위 vpos 와 layout 의 페이지 분할이 어긋남. 별도 후속 이슈 분리 권장.

## 정정 사항

단계 1 완료보고서에 "페이지 1 LAYOUT_OVERFLOW 4.1px 자연 해소" 라 기록되었으나, base ref(`local/devel`) 에서 본래 발생하지 않음을 단계 3 에서 확인. 첫 관찰은 작업 진입 직전 `pr-task677` 브랜치 환경 차이로 발생한 일회성 출력이며 본 수정의 효과가 아니다. 본 보고서에서 정정한다.

## 진행 결과

| 단계 | 핵심 작업 | Commit |
|------|----------|--------|
| 1 | layout_table 1×1 unwrap 조건 정밀화 | `4b357394` |
| 2 | 광범위 회귀 검증 (159 샘플 / 1502 페이지) | `01ecb71a` |
| 3-fix | height_measurer 1×1 unwrap 동일 결함 수정 | `7d8cca27` |
| 3 | 시각 정합 + 보조 관찰 + 정정 + 최종 보고서 | (이 commit) |

## 후속 권장

1. `samples/table-vpos-01.hwpx` 페이지 2~3 hwp_used 큰 diff 별개 결함 — 별도 이슈 분리 후 조사
2. 1×1 래퍼 unwrap 로직이 layout_table 과 height_measurer 두 곳에 중복 존재. 향후 공통 helper 로 추출 검토 (본 타스크 범위 외)

## 참고

- GitHub Issue: #688
- 수행계획서: `mydocs/plans/task_m100_688.md`
- 구현계획서: `mydocs/plans/task_m100_688_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_688_stage{1,2,3}.md`
- 권위 PDF: `pdf/table-vpos-01-2022.pdf` (한글 2022)
