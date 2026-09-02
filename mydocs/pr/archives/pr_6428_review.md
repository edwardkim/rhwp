---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6428
author: kevin9327
---

# PR #6428 review - 머리말 셀 TAC 도형의 저장 줄 폭

## 검토 대상

- 원 PR head: `80f5ef26240926840d6580631c1ba8eb7c421b63`, 통합 적용 최종 commit `f6d37697`.
- 검증 base: `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`; PR 직전
  `upstream/devel@cfa4ccacab63b470771720ebed33503cdd62adb6`로 충돌 없이 rebase했다.
- 2026-08-31 재조회에서 Open/non-draft, requested reviewer 없음이다. source head의 Build & Test, Lint,
  Native Skia, Archive A-D와 adapter/proptest는 성공했다.
- 바탕쪽 표 오른쪽 셀의 TAC 사각형만 저장 `segment_width`를 사용하고, 부동 그림과 일반 셀 줄 폭 경로는
  변경하지 않는다. source PR이 CI에서 발견한 과도한 "모든 셀 TAC" 적용 회귀도 최종 head에서 좁혀졌다.

## 시각 증적

- 입력: `samples/exam_kor.hwp`, `lastSavedWith.product=hancom-office-2022`이므로 2020 bucket을 사용했다.
- 기준 PDF: `pdf/exam_kor-hwp-2020.pdf`, A4 20쪽.
- 통합 head에서 p6을 `rsvg` sweep으로 확인했다. flagged=0, pixel match 89.05746%, visual-accuracy proxy
  12.56379%다. `exam_kor_odd_header_box_follows_stored_line_width`는 한/글 기준 x=924.0px 쪽으로
  이동했고 이전 rhwp x=926.1px보다 가까운지 직접 잠근다.
- 대표 `mydocs/pr/assets/pr_6428_issue6353_p6_review.png`를 열어 머리말 우측 "홀수형" 박스가 두 기준에서
  같은 머리 띠 위치에 존재함을 확인했다. 임시 output은
  `output/visual_sweep_kevin9327_20260831/pr6428_issue6353/pr6428-issue6353/review/review_006.png`다.
  proxy는 `rsvg`의 폰트 raster 차이에 민감한 보조값이다.

## 통합 검증과 판단

- fmt, native/WASM clippy, workspace build, all-target clippy, manifest, Rust unit tier check가 통과했고,
  release-test 전체 nextest는 `8870 passed, 46 skipped` (450.949초, exit 0)였다.
- rebase는 충돌 없이 적용됐으며 추가 로컬 회귀는 수행하지 않았다. 최종 PR head의 CI 통과를 merge 조건으로 둔다.

**수용 권고.** 변경은 저장 폭이 권위인 셀 TAC 오른쪽 정렬로 국한되고, 실제 HWP p6 sweep과 위치 회귀가
의도한 +1.88px 보정을 뒷받침한다.

## Merge 후 contributor PR comment 계획

- p6의 flagged=0, pixel/proxy 수치, 자동 지표의 한계와 사람의 위치 확인을 기록한다.
- devel 반영 asset을 API 재조회한 뒤 다음 merge-SHA 고정 image를 `--body-file`로 게시한다.

  ```markdown
  ![PR #6428 p6 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6428_issue6353_p6_review.png)
  ```
