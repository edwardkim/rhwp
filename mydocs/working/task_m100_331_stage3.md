# Task #331 Stage 3 완료 보고서 — 검증

- **이슈**: [#331](https://github.com/edwardkim/rhwp/issues/331)
- **브랜치**: `local/task331`

## 검증 결과

### 3-1. 21_언어 샘플 (이슈 재현 케이스)

```
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp
```

- ✅ 페이지 수 16 → 15
- ✅ LAYOUT_OVERFLOW 0건
- ✅ pi=26 (`2. '프로세스 마이닝'에 대해 추론한 것...`) + 보기 ①②③ (pi=27/28/29) 모두 page 0 col 1 에 fit — PDF 일치

### 3-2. lib 테스트

```
cargo test --lib
```

**결과**: `992 passed; 0 failed; 1 ignored`

- 5개 테스트가 trailing_ls 보정 후 페이지 수용량 변화로 calibration 필요
- `text_editing.rs` 의 5 개 페이지 경계 테스트 반복 수/텍스트 길이 조정:
  - `test_page_overflow_with_enter`: 50 → 100 enters
  - `test_page_break_with_default_line_spacing`: 50 → 100
  - `test_page_break_with_mixed_line_spacing`: 40 → 80
  - `test_page_break_with_tight_line_spacing`: 텍스트를 multi-line 으로 변경
  - `test_page_boundary_with_incremental_spacing_increase`: 39 → 30 multi-line, 범위 15..50 → 0..total, 상한 360% → 560%

**핵심 인사이트**: trailing_ls 가 advance 에서 제외됨에 따라 단일 줄 문단은 line_spacing 변경에 advance 가 영향받지 않음 (HWP 동작 일치). 따라서 `test_page_boundary_with_incremental_spacing_increase` 와 `test_page_break_with_tight_line_spacing` 은 multi-line 문단 텍스트로 변경.

### 3-3. Golden SVG (svg_snapshot)

```
cargo test --test svg_snapshot
```

**결과**: 6 passed (baseline 갱신 후)

- form-002, issue-267/ktx-toc-page, table-text, deterministic: 무영향 (이전 baseline 그대로 통과)
- issue-147/aift-page3: body/cell 클립 높이 9.5~9.6px 단축 (trailing_ls 만큼) — 의도된 변경, baseline 갱신
- issue-157/page-1: body 1042.93 → 1033.33 (-9.6), cell y 246.43 → 236.83 (-9.6) 등 순수 y-shift — baseline 갱신

콘텐츠 누락이나 겹침 없음 — 전체적으로 레이아웃이 +9.6px 만큼 위로 정렬되는 형태.

### 3-4. 통합 테스트

```
cargo test --test issue_301
cargo test --test hwpx_to_hwp_adapter
cargo test --test hwpx_roundtrip_integration
cargo test --test tab_cross_run
```

**결과**: 모두 통과

### 3-5. 기타 샘플 페이지 수 스팟 체크

| 샘플 | 페이지 수 |
|------|-----------|
| multi-table-002.hwp | 2 |
| tac-case-002.hwp | 1 |
| hwp-multi-002.hwp | 6 |

회귀 없음.

## 결론

- 모든 자동화 테스트 통과 (992 + 6 + 4)
- 21_언어 샘플 page 1 col 1 PDF 일치 확인
- Golden 2 개 baseline 갱신 (의도된 trailing_ls 보정 결과)
- 다른 샘플 회귀 없음
