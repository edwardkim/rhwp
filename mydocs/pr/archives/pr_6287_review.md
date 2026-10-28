---
kind: pr-review
status: accepted-with-residual-note
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 18:45 KST
pr: 6287
issue: 6204
author: planet6897
---

# PR #6287 review - 개체 이동 후 배제 밴드 저장 LINE_SEG를 다시 새긴다

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6287
- 작성자: `planet6897`
- reviewer: REST API로 `jangster77` review request 등록 완료
- 원 PR head: `63b477a75bbd59c01d1714e1d8932d29ec107477`
- 통합 검토 브랜치: `review/planet6897-open-ci-20260828`
- cherry-pick 결과:
  - `4c5b1e24b` 제품/회귀 수정
  - `acae5b323` rustfmt 보정
- 기준: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- 상태: non-draft, mergeable, 실패·진행 check 0건
- PR comments/review comments: 0건

## 검토 판단

**수용 권고.** 그림 위치/크기/어울림 기하가 바뀌었는데 저장 `LINE_SEG`가 기존 배제 밴드 기준으로 남아
저장 후 재오픈까지 잘못된 줄 흐름이 굳는 결함을, 기존 텍스트 편집의 picture-band 재투영 경로로 연결한다.
제품 변경은 “원본 샘플 그대로의 fidelity”가 아니라 “개체 속성 변경 후 stale lineSeg 갱신”이므로, edited
output 기준 증적으로 판단한다.

## 증적과 검증

- 원본 fixture: `samples/issue6204/square_picture_band_host.hwp`
- 원본 `rhwp info --json`: `mydocs/pr/assets/pr_6287_issue6204_info.json`
  - `format=hwp5`, `lastSavedWith=hancom-office-2020 11.0.0.2129`, `pageCount=1`
  - 저장 제품이 2020이므로 MCP `engine 2020`, suffix `-2020.pdf` 기준 적용
- 원본 기준 PDF:
  `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6287_issue6204_square_picture_band_host-2020.pdf`
- edited output 생성:

```text
target/pr-review/release-test/rhwp edit set-picture \
  samples/issue6204/square_picture_band_host.hwp \
  --section 0 --para 0 --ctrl 1 --props '{"horzOffset":5000}' \
  -o output/visual_sweep_planet6897_open_ci_20260828/edit_outputs/pr6287_issue6204_square_picture_moved.hwp \
  --verify --json
```

- CLI 검증 JSON: `mydocs/pr/assets/pr_6287_issue6204_set_picture_cli.json`
  - `verify.identical=true`, `verify.diffCount=0`
- edited output 보존:
  - `mydocs/pr/assets/pr_6287_issue6204_square_picture_moved.hwp`
  - `mydocs/pr/assets/pr_6287_issue6204_square_picture_moved_sha256.txt`
  - `mydocs/pr/assets/pr_6287_issue6204_moved_info.json`
- edited 기준 PDF:
  `pdf/pr_planet6897_open_ci_20260828/by_saved_version/pr6287_issue6204_square_picture_moved-2020.pdf`
- visual sweep:
  - 원본 p1: pixel `96.65243%`, visual proxy `7.29836%`, flagged page `1`
    (`square_wrap_text_overlap`, `content_bottom_drift`)
  - edited p1: pixel `96.59400%`, visual proxy `7.55510%`, flagged page `0`
  - 대표 edited asset:
    `mydocs/pr/assets/pr_6287_issue6204_moved_p1_visual_review.png`,
    `mydocs/pr/assets/pr_6287_issue6204_moved_visual_sweep_summary.json`
- focused test:
  - `issue_6204_object_move_invalidates_linesegs`: 1 pass
- 공통 로컬 검증:
  - fmt, suite manifest, unit-tier, CI 범위 clippy/check/WASM check, native-skia lib 통과

## 코멘트 처리

merge 후 원 PR/issue에는 다음을 남긴다.

- 원본 샘플 자체의 visual sweep은 기존 문서 fidelity 차이로 `square_wrap_text_overlap` 후보가 남았지만,
  이번 PR의 주장인 “그림 이동 뒤 stale LINE_SEG 갱신”은 edited output 기준으로 검증했다.
- `set-picture --verify`가 diff 0으로 통과했고, edited output의 visual sweep p1은 자동 flag 0건이다.
- focused 회귀는 저장 후 재로드에도 갱신된 `lineSeg.segment_width`가 유지됨을 검증한다.
- 대표 이미지는 merge SHA 고정 raw URL로 `pr_6287_issue6204_moved_p1_visual_review.png`를 첨부하고,
  원본 p1의 잔여 자동 flag는 이번 수용 판단과 분리한다고 명시한다.

## 후속

원본 문서의 낮은 visual proxy와 overlap 후보는 별도 fidelity 개선축으로 남길 수 있으나, #6204의
개체 이동 후 저장 사다리 갱신 결함을 막는 데는 차단 사유가 아니다.
