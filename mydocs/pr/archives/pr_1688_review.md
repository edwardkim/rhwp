# PR #1688 리뷰 — #1658 round 2 pagination cut 정합

- PR: #1688 `Task #1658 (round 2): continuation ≤3 cut 정합 + COM 행높이/클리핑 시각 게이트 인프라`
- 작성자: @planet6897
- 기준: `devel`
- 검토 대상 head: `59232b9efe573ecc39d1e7da3cf316315d263a33` (문서 작성 시점 참고값)
- 처리 방식: 원 PR 직접 merge 대신 #1690과 함께 통합 cherry-pick PR #1712로 반영
- 통합 merge: #1712, merge commit `b7d76030b5b0a54435e6d1237de976e45ffd3aba`
- 원 PR 후속: #1688 supersede comment 후 close 완료 (`2026-06-30T16:38:32Z`)

## 변경 요약

`advance_row_cut`의 `tiny_fragment_waste` 흡수 임계를 continuation 조각에서는 `<=3`, fresh 조각에서는
기존처럼 `<=2`로 분리한다. 거대 셀 분할 시 한글 break보다 1~3줄 이르게 capacity-break가 발생해
reset 직전 orphan 페이지가 생기는 케이스를 완화한다.

함께 추가된 검증 자산:

- `tools/hangul_row_heights.py`: 한글 COM 기반 행높이 측정 도구
- `tools/detect_table_clipping.py`: 본문 클리핑 검출 경로 강건화
- `tools/clipping_gate.py`: controlset 클리핑 회귀 게이트
- `tests/fixtures/clipping_baseline.tsv`: controlset 92문서 baseline
- `samples/byeolpyo1.hwp`, `samples/byeolpyo4.hwp`
- #1658 round 2 보고서와 기술 문서

## 리뷰 중 보정 사항

초기 검토에서 `tools/clipping_gate.py`가 렌더 실패 또는 fixture 누락을 success로 처리할 수 있음을 확인했다.
작성자는 `59232b9e`에서 다음을 보정했다.

- `ERR/없음` 카운트가 1 이상이면 `--check`, `--save` 모두 exit 1
- baseline에 있으나 측정되지 않은 key가 있으면 exit 1
- 요약 출력에 `ERR/없음`, `baseline누락` 표시

또한 `byeolpyo4.hwp`의 현 head 재현값이 과거 문서의 27쪽/23.5px가 아니라 26쪽/6클립/90.7px임을
확인했고, `mydocs/tech/task_m100_1658_giantcell_residual.md` 상단에 현행화 안내가 추가됐다.

## 로컬 검증

통합 cherry-pick 브랜치에서 #1688 후 #1690을 연속 적용한 결과 기준:

- 순차 merge simulation: #1688 -> #1690 충돌 없음
- `git diff --check`: 통과
- `python3 -m py_compile tools/clipping_gate.py tools/detect_table_clipping.py tools/valign_offset_gate.py`: 통과
- `CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `CARGO_INCREMENTAL=0 cargo build --release`: 통과
- `CARGO_INCREMENTAL=0 cargo test --release --lib`: 통과 (`2038 passed; 0 failed; 7 ignored`)
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `python3 tools/clipping_gate.py --check tests/fixtures/clipping_baseline.tsv --exe /Users/tsjang/rhwp/target/release/rhwp`: 통과
  - `문서=92 개선=0 회귀=0 ERR/없음=0 baseline누락=0`
- `rhwp info samples/byeolpyo4.hwp`: 26쪽
- `python3 tools/detect_table_clipping.py samples/byeolpyo1.hwp samples/byeolpyo4.hwp --exe /Users/tsjang/rhwp/target/release/rhwp`
  - `byeolpyo4.hwp`: `CLIP 6/26p max_overflow=90.7px`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1086 task1086_k_water_rfp_page_count_matches_hancom_pdf`: 통과

원 PR #1688 자체의 GitHub Actions 도 문서 작성 시점 최신 head 기준 모두 통과했다.

## 리스크

`byeolpyo4.hwp` 클리핑 자체는 이 PR의 해결 범위가 아니다. round 4 시도는 `issue_1086` 페이지 수 회귀를
유발해 PR에서 제거됐고, block-continuation 측정 불일치 정합은 후속 전용 작업으로 남는다.

`tools/clipping_gate.py`는 이제 누락과 렌더 실패를 실패로 처리하므로 회귀 차단 게이트로 사용할 수 있다.

## 최종 판단

수용 권고. 단독 merge보다 #1690과 함께 통합 cherry-pick PR 로 반영하는 편이 좋다. #1690이 round 2
인프라와 샘플을 전제로 하며, 두 PR의 최종 검증도 연속 적용 결과로 수행했다.

최종 처리 결과:

- 통합 PR #1712 GitHub Actions 통과 후 merge 완료
- 원 PR #1688 comment: https://github.com/edwardkim/rhwp/pull/1688#issuecomment-4845818362
- 원 PR #1688 close 완료
- #1658은 후속 block-continuation 정합 작업이 남아 있어 open 유지
