# Task #1116 Stage 22 보고서 — PR #1120 SVG 스냅샷 golden 갱신

## 작업 시각

- 2026-05-26 KST

## 사용자 지시

- PR #1120의 남은 CI 실패를 수정.

## 실패 내용

Stage 21 수정 커밋 이후 PR #1120의 `Build & Test`는 기존 `cargo test --lib` 실패 지점을 통과했다.

이후 전체 `cargo test`에서 `tests/svg_snapshot.rs`가 실행되며 다음 7개 스냅샷이 실패했다.

- `form_002_page_0`
- `issue_147_aift_page3`
- `issue_157_page_1`
- `issue_267_ktx_toc_page`
- `issue_617_exam_kor_page5`
- `issue_677_bokhakwonseo_page1`
- `table_text_page_0`

## 원인

#1116의 SVG/browser 폭 고정 변경으로 숫자/Latin 텍스트에 `textLength`와 `lengthAdjust="spacingAndGlyphs"`가 출력된다.

또한 목차 tab leader의 끝 좌표 일부가 텍스트 폭 고정값 기준으로 소폭 조정된다.

이는 렌더링 정책 변경을 반영한 기대 출력 변화이므로, 코드 회귀가 아니라 golden 미갱신이다.

## 수정 내용

다음 golden SVG를 갱신했다.

- `tests/golden_svg/form-002/page-0.svg`
- `tests/golden_svg/issue-147/aift-page3.svg`
- `tests/golden_svg/issue-157/page-1.svg`
- `tests/golden_svg/issue-267/ktx-toc-page.svg`
- `tests/golden_svg/issue-617/exam-kor-page5.svg`
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`
- `tests/golden_svg/table-text/page-0.svg`

테스트가 생성한 `.actual.svg` 임시 파일은 커밋 대상에서 제외하고 삭제했다.

## 검증

```bash
UPDATE_GOLDEN=1 cargo test --test svg_snapshot -- --nocapture
cargo test --test svg_snapshot -- --nocapture
```

결과:

- `cargo test --test svg_snapshot -- --nocapture`: 8 passed, 0 failed.

## 결론

PR #1120의 두 번째 CI 실패는 #1116 SVG 폭 고정 출력 변경에 맞춘 golden 갱신 누락이었다. golden을 갱신한 뒤 스냅샷 테스트가 통과한다.
