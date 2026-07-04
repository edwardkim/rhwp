# PR #1690 리뷰 — #1658 round 3 valign over-count 정정

- PR: #1690 `Task #1658: valign over-count 수정 (중첩 표 셀 세로정렬 상단정렬 결함)`
- 작성자: @planet6897
- 기준: `devel`
- 검토 대상 head: `ea89716a016df9dc0e33ebca990064890277f43f` (문서 작성 시점 참고값)
- 처리 방식: #1688 선행 반영 후 통합 cherry-pick PR #1712로 함께 반영
- 통합 merge: #1712, merge commit `b7d76030b5b0a54435e6d1237de976e45ffd3aba`
- 원 PR 후속: #1690 supersede comment 후 close 완료 (`2026-06-30T16:38:32Z`)

## 변경 요약

중첩 표가 있는 inline 셀에서 `total_content_height`가 과대 계산되어 `valign=Center/Bottom` offset이
0에 가까워지고, 결과적으로 내용이 상단 정렬처럼 보이는 결함을 수정한다.

핵심 수정은 `src/renderer/layout/table_layout.rs`의 inline 셀 경로에서 중첩 표 높이를
`text_height += nested_h`로 직접 더하지 않도록 한 것이다. 중첩 표 기여는 마지막 `max(...)` 단계의
`nested_bottom`과 stored vpos가 담당하므로 double-count를 제거한다.

함께 추가된 검증 자산:

- `samples/valign_fixtures/*.hwpx`: @kkyu8925 제보 기반 합성 fixture 4종
- `tools/valign_offset_gate.py`: `CENTERME` baseline 위치 기반 Center/Bottom 정렬 회귀 게이트
- #1658 round 3 계획, stage 문서, 보고서

## 리뷰 중 보정 사항

초기 검토에서 `tools/valign_offset_gate.py`가 fixture 없음, 마커 없음, export 실패 같은 누락 케이스를
success로 처리할 수 있음을 확인했다. 작성자는 `ea89716a`에서 다음을 보정했다.

- `BUG(미수정)`뿐 아니라 `누락`이 1 이상이면 exit 1
- 없는 fixture directory 기준 전부 누락 케이스 exit 1 확인
- 정상 fixture 기준 `BUG=0`, `누락=0` 확인

## 로컬 검증

통합 cherry-pick 브랜치에서 #1688 후 #1690을 연속 적용한 결과 기준:

- 순차 merge simulation: #1688 -> #1690 충돌 없음
- `git diff --check`: 통과
- `python3 -m py_compile tools/clipping_gate.py tools/detect_table_clipping.py tools/valign_offset_gate.py`: 통과
- `CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `CARGO_INCREMENTAL=0 cargo build --release`: 통과
- `CARGO_INCREMENTAL=0 cargo test --release --lib`: 통과 (`2038 passed; 0 failed; 7 ignored`)
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `python3 tools/valign_offset_gate.py --dir samples/valign_fixtures --exe /Users/tsjang/rhwp/target/release/rhwp`: 통과
  - `centered_cell_nested_table`: `FIX(Center)`
  - `cell_vcenter_multi_nested_overcount`: `FIX(Center)`
  - `cell_vbottom_nested_overcount`: `FIX(Bottom)`
  - `cell_vcenter_nested_undercount`: `가드OK`
  - `BUG(미수정)=0 누락=0`
- `python3 tools/clipping_gate.py --check tests/fixtures/clipping_baseline.tsv --exe /Users/tsjang/rhwp/target/release/rhwp`: 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1086 task1086_k_water_rfp_page_count_matches_hancom_pdf`: 통과

원 PR #1690 자체의 GitHub Actions 도 문서 작성 시점 최신 head 기준 모두 통과했다.

## 리스크

#1690은 #1688의 round 2 샘플과 클리핑 게이트를 전제로 검증된다. 따라서 개별 직접 merge보다 #1688을
먼저 반영한 뒤 #1690을 이어 반영하는 순서가 맞다.

`samples/byeolpyo4.hwp`는 #1688에서도 추가되므로 통합 cherry-pick에서는 샘플 blob이 중복되지 않는다.
원 PR #1690의 `aec4ad8a3` 커밋은 통합 브랜치에서 `mydocs/working/task_m100_1658_v2_stage2.md`의
재현 설명 보정으로 적용됐다.

## 최종 판단

수용 권고. #1688 선행 반영 후 #1690을 이어 반영하면 충돌이 없고, 로컬 게이트와 핵심 회귀 테스트도 통과했다.

최종 처리 결과:

- 통합 PR #1712 GitHub Actions 통과 후 merge 완료
- 원 PR #1690 comment: https://github.com/edwardkim/rhwp/pull/1690#issuecomment-4845818372
- 원 PR #1690 close 완료
- #1658은 후속 block-continuation 정합 작업이 남아 있어 open 유지
