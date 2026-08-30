---
kind: pr-review
status: maintainer-fix-acceptable
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6445
issue: 6312
author: planet6897
---

# PR #6445 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 불가: 최신 devel과 충돌 |
| 수용 대상 | 메인터너 보정 `de5209d52d20749ec413a996f0c89da0e7af1362`이 포함된 #6481 |
| 현재 상태 | 메인터너 보정 후 수용 가능: 원 head 충돌을 해소한 보정과 N-up 원본 physical-sheet 판정에 차단 finding 없음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6445를 원 head가 아닌 보정된 통합 결과 기준으로 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6445 |
| 원 head | `222c81ef2fbc244087bfe0791959124f95a2930b` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `96df435c74cfcd659074b5074918cc5cfe2de422` |
| 메인터너 보정 | `de5209d52d20749ec413a996f0c89da0e7af1362` |
| 통합 순서 | 3/8 |

## 검토와 충돌 보정

float anchor line에서 저장된 source vertical position을 사용하는 #6312 보정이다. 원 PR CI는 수집 시점에 비성공 check 없이 완료됐지만 최신 devel과 `src/renderer/float_placement.rs`가 충돌했다.

충돌은 현행 `next_plain_text_vpos`와 visible-line helper를 유지하면서 `source_line_seg_vertical_pos`를 우선 사용하는 방식으로 해소했다. 이어 cached typeset 경로의 `next_para_first_stored_vpos`에도 같은 우선순위를 연결해 원 PR 계약이 우회되지 않게 보완했다. fixture baseline 충돌에서는 기존 #6300과 새 #6442 행을 모두 보존했다.

통합 후보에서 `anchor_paragraph_line_advances_the_flow`, `paragraph_error_matches_the_table_error`, 기존 계약인 `issue_6312_visible_tab_host_keeps_its_own_line`이 통과했다. 공통 필수 clippy·build·manifest·format 검증도 통과했다.

원 PR의 시각 자료는 보조 근거로만 사용했다. hidden 원본 `156721992`와 repo의 image-only slice를 구분하고, HWP MCP 2020으로 만든 hidden-original physical sheet 1의 좌측 slot을 통합 code head logical page 1과 직접 비교했다. `pixel_match=85.33588`이며, slice가 BinData 이미지를 placeholder로 치환한 차이는 남지만, 이번 주장인 제목 아래 앵커 문단의 별도 줄과 아래 흐름은 Hancom sheet에서 확인됐다. 대표 증적은 [N-up claim review PNG](../assets/pr_6481_issue6312_nup_claim_review.png)이며, 원본/PDF SHA, physical-slot mapping, 명령과 한계는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 보존했다.

## 현재 결론

**최종 판정: 메인터너 보정 후 수용 가능.** 원 PR head는 최신 devel에 직접 merge할 수 없지만, source-vpos 계약을 현행 float/typeset 경로로 옮긴 `de5209d52d20749ec413a996f0c89da0e7af1362` 보정은 focused 회귀와 N-up 원본 시각 판정을 통과했다. placeholder image 차이는 이번 앵커-line 주장과 분리한다. 원 PR은 직접 merge하지 않고 보정된 #6481 통합 결과로만 수용한다. remote push, merge, #6445 close는 별도 지시가 있을 때만 수행한다.
