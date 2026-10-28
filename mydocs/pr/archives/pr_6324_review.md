---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6324
---

# PR #6324 review - 셀 float 자기 변위를 흐름 오프셋으로 오인하지 않는다

## 검토 판단

**수용 권고.** 셀 안 그림의 저장 `vertical_pos`가 그림 높이와 세로 오프셋의 합일 때만
자기 변위로 판정하도록 좁힌 수정이다. `microbe_bank_cell_picture.hwpx`의 그림이 셀과
용지 밖으로 더 전진하던 원인을 직접 고정한다.

## 라우팅과 근거

- 원 PR: https://github.com/edwardkim/rhwp/pull/6324
- 작성자 / reviewer: `planet6897` / `jangster77` review request 등록
- source head: `77207f87bcbe39ae908dda6050238c3ba77c14f6`
- 통합 검토 branch: `review/open-nondraft-20260830`
- source CI의 rustfmt 실패는 maintainer commit `9e84ab0a4`로 `table_layout.rs`만 포맷 보정했다.
- focused `issue_6313_cell_picture_own_displacement`: 2/2 통과.

## 시각 증적

- fixture: `samples/issue6313/microbe_bank_cell_picture.hwpx`
- `info --json`: `hancom-office-2020 11.0.0.7571`, 5쪽. 2020 bucket을 선택했다.
- 기준 PDF: `pdf/microbe_bank_cell_picture-2020.pdf`.
  Hancom 2020 MCP 비동기 변환은 5/5쪽, SHA-256
  `42e4116a77413337e76e13328da757c62e36f378e1b29b857692de744a1aaa65`를 확인했다.
- all-page visual sweep: 5/5 완료, 자동 flag 0. 평균 pixel match `88.42851%`.
  글리프 모양 차이는 있으나 그림 placeholder와 표/본문의 흐름 이탈은 보이지 않았다.
- 보관 asset: `mydocs/pr/assets/pr_6324_issue6313_{info,visual_sweep_summary}.json`,
  `mydocs/pr/assets/pr_6324_issue6313_p5_review.png`.

## 공통 검증과 코멘트

- 통합 head에서 fmt, workspace/native/WASM clippy, release-test 전체 8,712/8,712,
  Native Skia, wasm-pack locked, Studio gate를 통과했다.
- 통합 PR merge 후 원 PR에는 2020 bucket, 5쪽 sweep의 flag 0, focused 2/2와 함께 위 대표
  이미지를 merge SHA 고정 raw URL로 남긴다.
