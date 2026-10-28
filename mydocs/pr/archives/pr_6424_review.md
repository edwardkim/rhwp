---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6424
author: kevin9327
---

# PR #6424 review - 같은 vertpos 표 조각의 높이 이중 계상 방지

## Metadata

- 원 PR: [#6424](https://github.com/edwardkim/rhwp/pull/6424), source head
  `43d1a8d03f25370697140fa1dfa122f07ddb3e36`.
- 작성자: `kevin9327`; external collaborator reviewer `jangster77` 요청 완료.
- 고정 시점에 Open, non-draft, CI green인 head를 latest `upstream/devel` 위에 conflict 없이
  적용했다.

## 변경과 검토

- 같은 stored `vertpos`를 가진 모든 조각을 무조건 한 줄로 보던 경로를, `column_start`가
  다른 가로 조각에만 적용하도록 좁힌다.
- 이로써 동일 vertical position이지만 같은 column 안에서 이어지는 조각의 cell height가
  중복 계상되는 것을 막는다.

## 시각 증적

- 원본 기준은 `/home/tsjang/Downloads/korea_downloads/산림청/156518878_(국립자연휴양림 보도자료) 국립자연휴양림, 몰카로부터 안전하게 이용하세요!.hwpx`다.
  `rhwp info --json`의 저장 제품은 Hancom Office 2020이므로 Hancom 2020 PDF를 사용했다.
- 기준 PDF: `pdf/156518878-2020.pdf`, SHA-256
  `343d893234f83d631bd1d9921d6fa99df6e40f7c20062a68d84a1ee95208221b`, A4 1쪽.
- Visual Sweep은 실제 한 쪽을 끝까지 비교했고 flag는 없었다. pixel match는 91.32844%,
  visual-accuracy proxy는 18.76937%다. 후자는 글리프 raster 차이를 크게 받는 보조 지표이며
  fidelity 합격률이 아니다.
- 사람 확인에서 제목, 본문 bullet, 표/텍스트 band의 순서와 간격에 겹침이나 잘림이 없었다.
  보존 asset: `mydocs/pr/assets/pr_6424_156518878_review.png`,
  `mydocs/pr/assets/pr_6424_156518878_visual_sweep_summary.json`.

## 댓글 계획과 권고

- merge 뒤 source PR에 Visual Sweep guide 링크, 1쪽 완료/무flag, 위 지표와 사람 확인의 한계를
  함께 기록한다.
- image comment는 devel에 asset 존재를 API로 재확인한 뒤 `--body-file`로 게시하며, 다음 raw
  URL 형식을 사용한다.

  ```markdown
  ![PR #6424 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6424_156518878_review.png)
  ```

**수용.** 통합 branch full nextest `8772 passed, 43 skipped` (430.908초, exit 0)와
위 1쪽 시각 증적을 근거로, `column_start`를 포함한 정확한 동치 조건 변경을 수용한다.
