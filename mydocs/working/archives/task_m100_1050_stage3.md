# Task #1050 Stage 3 보고서 — 회귀 가드 + sweep + WASM + 시각 판정 게이트

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050)
- 단계: Stage 3
- 일시: 2026-05-21

## 1. 결과 요약

회귀 가드 4 추가 (4/4 통과) + 9 fixture 광범위 sweep diff=0 (직접 export 회귀 부재) + WASM Docker 빌드 (4.90 MB) + rhwp-studio/public 동기화 + 한컴 시각 판정용 산출물 준비 완료.

## 2. 회귀 가드 추가

`tests/issue_1050_footnote_serialize.rs` (88 라인, 4 tests):

| 테스트 | 검증 항목 |
|--------|----------|
| `issue_1050_hwpx_to_hwp_textbox_footnote_roundtrip` | HWPX → HWP 저장 후 재로드 시 글상자 안 각주 본문 "글상자 내부 각주" + 본문 각주 회귀 부재 |
| `issue_1050_hwp_roundtrip_footnote_preserved` | HWP → HWP 라운드트립 시 footnote IR 보존 (footnote-01.hwp ≥5 footnote + paragraphs 정합) |
| `issue_1050_hwpx_footnote_attrs_mapped` | HWPX `suffixChar` / `instId` → `after_decoration_letter` / `instance_id` 매핑 |
| `issue_1050_ctrl_footnote_size_20` | 저장본 CTRL_FOOTNOTE size = 20 (한컴 정답지 정합) |

결과: **4/4 passed**.

## 3. 광범위 sweep (9 fixtures, 143 SVG each)

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|-------------------|
| samples/hwpx/footnote-tbox-01.hwpx | 1 | 0 |
| samples/footnote-tbox-01.hwp | 1 | 0 |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

```
diff -rq output/poc/issue_1050/sweep-before/ output/poc/issue_1050/sweep-after/ = 0
```

→ **전체 143 SVG 완전 동일** (직접 export 회귀 부재 정량 입증). Task #1050 본질은 parser + serializer 영역, 직접 export 무영향 expected — 확인.

## 4. CI 패턴 검증

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo test --release --test issue_1050_footnote_serialize | **4/4 passed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

## 5. WASM 빌드 + 동기화

| 항목 | 결과 |
|------|------|
| Docker WASM 빌드 | OK (`pkg/rhwp_bg.wasm` 4.90 MB) |
| rhwp-studio 동기화 | OK (`public/rhwp_bg.wasm` + `rhwp.js` 247 KB) |

## 6. 한컴 시각 판정용 산출물

`output/poc/issue_1050/footnote-tbox-01-stage3-final.hwp` (14 KB) — Stage 2 정정 후 HWPX → HWP 저장본.

작업지시자 판정 항목:
1. 한컴 한글 2020 로 위 파일 열기 → **각주 출력 정합** 확인 (Stage 0 의 "각주 출력 비정상" 회귀 부재)
2. 페이지 하단 각주 영역에:
   - "1) 글상자 내부 각주"
   - "2) 일반 문단내 각주"
3. 한컴 PDF 정답지 (`pdf-large/hwpx/footnote-tbox-01.pdf`) 와 비교
4. rhwp-studio HWPX 직접 로드 → 동일 각주 표시 (회귀 부재)

판정 통과 후 Stage 4 (no-ff merge + close + orders + archives) 진행.

## 7. 잔여 / 후속

- 미주 (Endnote) 보유 sample 의 한컴 호환 직접 검증 — 본 task 에서는 Footnote 와 동일 본질 추정으로 패턴 적용. 별도 sample 확보 시 후속 검증.
- 한컴 footnote payload 추가 의미 (numberShape per-footnote 다른 값) — 본 task 는 default 0 사용. FootnoteShape 와 일관성 후속 검토.
- 글상자 LIST_HEADER (`BodyText.Section0#18`) 의 size=33 vs 20 차이 — 본 task 범위 외 (글상자 영역, 별도 task)
