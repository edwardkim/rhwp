# Task #1050 Stage 5 보고서 — footnote-01.hwpx 추가 sample 검증 + 작업지시자 시각 판정 통과

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050)
- 단계: Stage 5 (한컴 시각 판정 게이트)
- 일시: 2026-05-21

## 1. 작업지시자 시각 판정 결과 — 통과 ✓

### 1.1 footnote-tbox-01.hwpx
> "이제 잘 열리고 정확하게 한컴편집기에서 조판됩니다."

- 한컴 한글 2020 정상 열기 + 각주 영역 정상 조판 (separator + "1)" "2)" prefix + 본문)
- 각주 추가/삭제 동작 정상 (이전 결함: 본문 다단계 목록 1.1.1.1.1. 표시 → 해결)

### 1.2 footnote-01.hwpx (작업지시자 추가 sample)
> "성공입니다."

- 9 개 각주 보유 fixture
- 한컴 한글 2020 정상 조판

## 2. 정량 입증 결과 (footnote-01.hwpx)

### 2.1 hwp5-inventory-diff

| 영역 | 차이 |
|------|------|
| **CTRL_FOOTNOTE size** | 정답지 정합 (모든 9 개) |
| **footnote LIST_HEADER** | 정답지 정합 |
| **footnote 안 PARA_TEXT** | 정답지 정합 (size_changed 0) |
| 잔여 차이 | FOOTNOTE_SHAPE tuple=2 (endnote shape) 1건 — footnote 외 영역 |

### 2.2 자기 라운드트립 SVG (6 페이지)

| 페이지 | 각주 마크 수 |
|--------|------------|
| 1 | 2 |
| 2 | 2 |
| 3 | 5 |
| 4-6 | 0 (각주 본문 영역) |

→ 총 9 개 각주 정상 표시. 본문 paragraph 회귀 부재.

## 3. 광범위 sweep — 10 fixture

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|------|
| samples/hwpx/footnote-tbox-01.hwpx | 1 | **1** (의도) |
| samples/footnote-tbox-01.hwp | 1 | 0 |
| **samples/hwpx/footnote-01.hwpx** | 6 | **3** (의도) |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

```
diff -rq output/poc/issue_1050/sweep-before-v2/ output/poc/issue_1050/sweep-after-v2/ = 4
```

→ **HWPX 출처 2 fixture (footnote-tbox-01 + footnote-01) 만 변동 = 의도된 본질 정정**.
HWP 출처 8 fixture 회귀 부재.

## 4. 회귀 가드 확장 — 7/7 통과

`tests/issue_1050_footnote_serialize.rs`:

| 테스트 | 검증 |
|--------|------|
| issue_1050_hwpx_to_hwp_textbox_footnote_roundtrip | 글상자 안 footnote SVG 표시 |
| issue_1050_hwp_roundtrip_footnote_preserved | HWP 라운드트립 회귀 부재 |
| issue_1050_hwpx_footnote_attrs_mapped | suffixChar / instId 매핑 |
| issue_1050_ctrl_footnote_size_20 | CTRL_FOOTNOTE size=20 |
| **issue_1050_hwpx_footnote_shape_contract** | FootnoteShape 16 필드 정합 |
| **issue_1050_footnote_paragraph_char_offsets** | 각주 안 paragraph char_offsets [0, 8, 9, ...] |
| **issue_1050_footnote_01_hwpx_roundtrip** | footnote-01.hwpx 9 footnote 라운드트립 |

## 5. CI 패턴 (재검증)

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo test --release --test issue_1050_footnote_serialize | **7/7 passed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 | OK (`pkg/rhwp_bg.wasm` **4.91 MB**) |
| rhwp-studio 동기화 | OK |

## 6. 다음 단계

Stage 6 — commit + merge + push + close #1050 + orders + archives.
