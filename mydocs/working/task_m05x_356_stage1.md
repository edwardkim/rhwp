# 단계 1 보고서 — Task #356 재현 및 정량 진단

- **단계**: 1/5 (재현·정량 진단, 코드 무수정)
- **브랜치**: `local/task356`
- **빌드**: `cargo build --release` (clean, 15.94s)

## 1. 본 샘플 페이지 3 재현

**명령**: `./target/release/rhwp dump-pages "samples/2022년 국립국어원 업무계획.hwp" -p 2`

```
=== 페이지 3 (global_idx=2, section=0, page_num=3) ===
  body_area: x=75.6 y=94.5 w=642.5 h=933.5
  단 0 (items=23, used=913.9px, hwp_used≈134.2px, diff=+779.8px)
    Table          pi=20 ci=1  1x1  642.5x45.5px  vpos=0..0 [vpos-reset@line1]
    FullParagraph  pi=21  h=8.0   vpos=4614  "(빈)"
    Table          pi=22 ci=0  vpos=5574
    FullParagraph  pi=23..38 ...  (정상 진행)
    FullParagraph  pi=39  h=41.3  vpos=66281..68681  " ㅇ 국어사전 정보보완심의회…"
    FullParagraph  pi=40  h=28.0  vpos=500           "□ 교육 대상 발굴 및 콘텐츠…"   ← vpos 리셋
    FullParagraph  pi=41  h=46.7  vpos=3560..5812
    PartialParagraph  pi=42  vpos=8564..10816
```

### vpos 권위값 분석

| 항목 | 값 |
|------|-----|
| body_area 높이 | 933.5px ≈ 70,012 HU (1px = 75 HU @96dpi) |
| pi=39 vpos_end | 68,681 HU (남은 공간 1,331 HU = 17.7 px) |
| pi=40 spacing_before | 1,000 HU = 13.3 px |
| pi=40 첫 줄 line_height | ≈ 1,600 HU = 21.3 px |
| 예상 vpos (리셋 없으면) | 71,281 HU > 70,012 → **HWP 가 새 페이지로 보냄** |
| 실제 pi=40 ls.vpos | **500 HU** (= 새 페이지 상단) |

### 페이지네이터의 px 평가 (현재 동작)

- `used = 913.9 px`, `body = 933.5 px`, 잔여 19.6 px → pi=40 (sb=6.7 + line=21.3 = 28.0 px) 시도. 잔여 19.6 < 28.0 이지만…
- `paginate_text_lines()` engine.rs:617~636 의 `effective_trailing` 로직으로 trailing 제외 이후 적합 판정 → pi=40 을 같은 페이지에 강제 배치
- 결과: pi=40, pi=41 (h=46.7), pi=42 (PartialParagraph) 까지 들어가며 SVG 좌표는 body 박스 초과
- LAYOUT_OVERFLOW 경고:
  ```
  LAYOUT_OVERFLOW: page=2, col=0, para=41, y=1085.0, bottom=1028.0, overflow=57.0px
  LAYOUT_OVERFLOW: page=2, col=0, para=42, y=1121.7, bottom=1028.0, overflow=93.7px
  ```

## 2. 동일 패턴 페이지 29 (pi=572 → pi=573 리셋)

```
페이지 29: items=20, used=915.0px, hwp_used≈133.0px, diff=+782.1px
  Table  pi=572  vpos=62012
  FullParagraph  pi=573  vpos=500     ← 리셋 (HWP 가 새 페이지로 보냄)
  FullParagraph  pi=574  vpos=3324..5648
  PartialParagraph  pi=575  vpos=8472..10796
```

LAYOUT_OVERFLOW 경고:
```
page=28, col=0, para=573, overflow=18.1px
page=28, col=0, para=574, overflow=75.7px
page=28, col=0, para=575, overflow=113.4px
```

## 3. 전체 SVG 페이지 수

| 항목 | 값 |
|------|-----|
| 현재 SVG 페이지 수 | **35** (`/tmp/t356_before/*.svg` 카운트) |
| PDF 페이지 수 (이슈 본문) | 37 |
| 차이 | -2 |

vpos 리셋 무시로 페이지당 2개 문단/표가 더 들어가고 있어 누적 페이지 부족. 본 샘플에서 명확히 확인된 리셋 지점은 페이지 3, 페이지 29 두 군데로, fix 시 2 페이지 추가가 정확히 발생 가능 → **PDF 와 일치 (37) 가능성 높음**.

## 4. 회귀 베이스라인 (코드 변경 전)

- `cargo test --release` 전체 통과: 1008 + 14 + 25 + 6 + 1 + 1 PASS, 0 FAIL
- 기준 SVG 디렉터리: `/tmp/t356_before/` (35 파일) — 단계 4 비교용 보존

## 5. 영향 범위 추정

| 영역 | 평가 |
|------|------|
| 본 샘플 (page 3, 29) | **확정 수정 대상** |
| 다른 샘플 — vpos 리셋 빈도 | 단계 4 에서 측정 (form-002, issue-147/157/267, table-text 골든 + 기타) |
| 표 분할 | 본 fix 는 *문단* 진입 직전 분기만 처리. 표 컨트롤은 별도 경로(`process_table_paragraph`) 라 영향 최소 예상 |
| 머리말/꼬리말/각주 | 본문 흐름과 분리되어 있어 영향 없음 (구현 시 재검증) |

## 6. 결론 및 다음 단계

- **확정**: HWP 권위값(LINE_SEG vpos) 기준으로 pi=40 / pi=573 은 새 페이지의 첫 문단. 현재 페이지네이터가 이 신호를 무시하여 본문이 body_area 밖으로 밀려남
- **헬퍼 시그니처 확정**:
  ```rust
  fn detect_inter_paragraph_vpos_reset(prev: &Paragraph, cur: &Paragraph) -> bool
  ```
  조건: 둘 다 line_segs 비어있지 않음 + 같은 column_start + cur.first.vpos < prev.last.vpos_end
- **단계 2 진행 요청**: 헬퍼 + 단위 테스트 추가
