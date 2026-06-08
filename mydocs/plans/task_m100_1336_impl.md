# 구현계획서 — 미주 다단 영역 body 초과 정정 (M100 #1336, 스코프 A)

수행계획서: `task_m100_1336.md`

## 결정 스코프

미주(endnote) 다단 영역에서 단 0 과적으로 콘텐츠가 body 하단을 크게 초과(p22 +82px)하는
버그만 수정. 본문 줄간격 드리프트(B)는 제외.

## 단계 구성 (4단계)

### Stage 1 — 근본원인 확정 (조사, 코드 무수정)

- p22 미주 2단의 단 0(35 items)에서 **typeset 누적 높이(used=1010.7)** 와 **layout 실제
  배치 높이(~1174)** 의 ~160px 불일치 지점을 특정.
- 조사 도구: `dump-pages -p 21`, `export-render-tree`, 미주 흐름 코드
  (`typeset.rs` 미주 fit 경로 ~2074~3460, `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX`
  적용처), 미주 area 배치(layout 측).
- 확인 항목:
  1. 미주 단 0 fit 판정이 어떤 높이(height_for_fit/total_height/vpos-span)로 이뤄지나.
  2. layout 이 항목을 배치하는 높이와 왜 어긋나나(미주 vpos-delta 시드, 줄간격 누적).
  3. `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX` 가 과적을 허용하는지.
  4. 단 0 초과 시 단 1/다음 페이지 이월이 왜 발동 안 하나.
- 산출: `working/task_m100_1336_stage1.md` (불일치 지점 + 최소 수정안 + 회귀 위험 평가)

### Stage 2 — 수정

- Stage 1 에서 특정한 지점에 **최소 범위** 수정(미주 단 fit 높이 ↔ layout 높이 정합,
  또는 단 0 over-fill 차단 후 이월).
- 미주 trailing 전면 통일 금지(메모리). 본문/각주 경로 불변.
- 산출: 소스 커밋

### Stage 3 — 검증 + 회귀

- 대상 문서: p22 미주가 body 안에 배치되는지(`LAYOUT_OVERFLOW` 경고 소멸), PDF 와 미주
  영역 정합.
- 회귀: 미주 보유 샘플(`issue_1082_endnote_multicolumn_drift`, exam_*, hwpspec 등) +
  전체 `cargo test --release` + `clippy`.
- 단 구분선(#1335 연계): 캡 없이도 꽉 찬 페이지 구분선 초과 완화되는지 부가 확인
  (단, #1334 미적용 base 이므로 정성 확인).
- 산출: `working/task_m100_1336_stage3.md`

### Stage 4 — 최종 보고

- `report/task_m100_1336_report.md`.

## 검증 기준 (Stage 3 합격선)

1. p22 미주 콘텐츠가 body 하단 안에 배치(`LAYOUT_OVERFLOW` 경고 0).
2. 미주 보유 샘플 회귀 없음, 전체 테스트 통과, clippy 무경고.
3. 미주 영역 PDF 대비 항목 누락/중복 없음.
