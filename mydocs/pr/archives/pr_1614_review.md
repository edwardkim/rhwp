# PR #1614 처리 보고서 — footer(발신명의) Page+Bottom page-fit 정합 (-1쪽 갭 요인 B, cherry-pick 통합)

- PR: https://github.com/edwardkim/rhwp/pull/1614
- 제목: `Task #1611: footer(발신명의) Page+Bottom page-fit 정합 (−1쪽 갭 요인 B)`
- 작성자: planet6897 (collaborator)
- 연결: Closes #1611 (=Task #1600 -1쪽 갭 요인 B), #1610(#1608) 위 스택
- base ← head: `devel` ← `planet6897:pr/devel-1611-squash`
- 처리일: 2026-06-28

## 1. 처리 결정

**cherry-pick 통합 후 PR close.** -1쪽 갭 시리즈의 요인 B — `VertRelTo::Page`+`valign=Bottom`+
`TopAndBottom` 비-TAC footer(발신명의)가 page-fit ~60px 과소로 본문 페이지에 흡수되던 문제를
정정한다. #1610 머지 후 본 PR 이 `render_minus1_page_gap.md` add/add 충돌로 CONFLICTING
(소스 무충돌)이 되어, #1611 신규 단일 커밋(`900c8ed0`)만 통합한다.

## 2. 충돌

#1611 신규 커밋은 단일 커밋이며 #1610/#1609 스택 중복 없음. 충돌은
`mydocs/tech/render_minus1_page_gap.md` add/add 1건뿐. PR판이 devel판의 **완전한 상위집합**
(devel 고유 0줄, PR 추가 13줄 = 요인 B 해소 섹션) → PR판 채택으로 무손실 해소. 소스
(typeset/parser/test)는 충돌 없이 적용.

## 3. 통합 내용 (devel 위 cherry-pick 1커밋, 작성자 보존)

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | generic fit 직전 `is_page_bottom_topbottom_block` 분기 추가 (+47줄). 4조건 가드(`!TAC && TopAndBottom && Page && Bottom`)로 좁게 한정. ① `current_height`를 stored vpos(`line_segs[0].vertical_pos`)로 동기화 ② `block_height = max(table_total, 선언높이 common.height)`로 fit → 초과 시 블록 통째 다음 쪽 단독 배치 |
| `src/parser/hwpx/{mod,header}.rs` | #1608 스택 잔재(요인 A) — 이미 devel 반영분과 동일, no-op |
| `tests/issue_1611_footer_page_bottom_pagination.rs` | footer 2쪽 분리 가드 (신규) |
| `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx` | fixture (46KB) |
| `tests/fixtures/render_page_controlset.tsv` | 통제셋 갱신 |
| 다수 `mydocs/` 문서 | 계획/단계/최종 보고서 (스택 누적) |

핵심: Paper-앵커(절대좌표, 페이지네이션 불참, typeset.rs 위 10440줄)와 달리 Page-앵커는
페이지네이션에 참여하므로 한컴처럼 stored vpos 위치 + **선언 높이**로 fit 판정해야 한다.
페이지네이터 effective_height(셀 내용 측정치, 302.3px)가 선언(351.4px)보다 작아 fit 이
과소되던 것을 선언 높이로 예약·판정해 정정.

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass (21m42s) |
| 충돌 시뮬레이션 | 문서 1건(상위집합 해소) |
| 신규 `issue_1611_footer_page_bottom_pagination` | 통과 |
| `issue_1608` / `hwpx_roundtrip_baseline` / `visual_roundtrip_baseline` | 4 / 1 / 3 passed |
| 전체 `cargo test --tests` | **FAILED 0건** (160 결과 / 누적 2636 passed) |
| fmt --check | clean |
| clippy | PR 변경 무경고 (잔존 1건은 `tests/issue_1585_...` 무관 기존 코드) |

## 5. 시각 판정 주의

통제셋 일치 66→72(net +6, −1쪽 21→12: 9해소 3회귀)는 컨트리뷰터 한글 오라클
(`render_page_gate.py`, 로컬 root) 측정이다. 로컬에선 baseline 게이트로 회귀 없음만 확인했고,
페이지 정합 권위는 작업지시자 환경(`feedback_self_verification_not_hancom`). PR 이 회귀 3건
(한컴이 1쪽 유지하는 footer 를 과도하게 민 over-correction, net +6 우세로 수용)을 정직하게 명시.

## 6. 후속

- 요인 A(#1610)·요인 B(본 PR) 처리로 -1쪽 갭 21→12. 잔존 12건은 over-correction 회귀 3건 포함
  — 추가 정합 시 별도 조사.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1614_review.md`
