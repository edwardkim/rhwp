---
kind: pr-review
status: approved-with-fidelity-residual
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5842 검토 - renderer bughunt r7

## 판정

- source head `1ee1ba2f10bd242c9fabed60729d32ec40d2f3d4`를 적용했다.
- 셀 다문단 float picture 적층, 퇴화 baseline, 셀 Square-wrap을 보정한다. 기능 회귀는 없다.

## 검증과 시각 증적

- `issue_5833_cell_multi_para_float_pics`, `issue_5825_degenerate_stored_baseline`, `issue_5818_cell_square_logo_text_wrap`은 각각 1/1 통과했고, 통합 전체 nextest 8,109/8,109도 통과했다.
- HWP 2020 기준 `pdf/pr_open_20260821/cell_square_logo_text_wrap-2020.pdf`는 job `65826658-880e-4c19-9a53-a8f2cc7c889a`, validation ok, 1 page, SHA-256 `773e13465c97c66f5b6892b08403ee865762b87dd75c84c21a78728f6daa6023`다.
- p1 pixel match 99.31476%/proxy 9.72222%이며, 사람 대조에서 headline/logo box 기하는 맞고 glyph raster 차이는 남았다. 대표 이미지는 `mydocs/pr/assets/pr_5842_issue5818_review_001.png`다.

## 최종 판단과 GitHub 기록

- **수용**: #5844 전체 CI와 r7의 세 직접 회귀 계약이 성공했다. glyph raster 차이는 기록된 renderer/PDF 차이이며 추가 코드 보정이나 보류 사유는 아니다.
- merge 뒤 원 PR에는 셀 그림 적층·baseline·Square-wrap 수용 근거를 comment로 남기고 close한다.
