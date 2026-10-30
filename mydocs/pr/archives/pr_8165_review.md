# PR #8165 — 머리말·꼬리말 틀과 바탕쪽 상속

## 최종 판정

**승인·머지 완료.** 2026-10-30 메인터너가 알려진 BOX 차이를 인지한 상태에서 원 PR 수용을 명시 승인했다. 앞선 재작업 요청은 이 PR의 머지 조건에서 해제했다. 남은 차이를 해결했다고 판정하거나 일반 시각 검증 규칙을 변경한 것은 아니다.

- 작성자: pblsketch. [원 PR #8165](https://github.com/edwardkim/rhwp/pull/8165).
- 원 head: `0de61c2578063775dde80b6ccdc41e2a8ed8841f`.
- 최신 devel 기준: `ad4bf269ffef5adf2142b33004a9981fad7ce804`.
- merge-tree: 충돌 없음, tree `d3647ea09b6663f53bf91966c4f522a1ead002db`.
- merge SHA: `f820e064a53b99259fa1c3ed17d8df731f72c1d1`, 2026-10-30 02:23:44 UTC.
- reviewer edwardkim을 API로 지정·확인하고 [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8165#pullrequestreview-5707379655)를 게시했다.
- 원 기여 커밋을 보존하는 merge 방식이다. 메인터너 source·test 수정과 fork push는 없었다.

## 범위와 판단 변경

머리말·꼬리말의 용지·쪽 기준 표와 글상자를 해당 틀에서 배치하고, 바탕쪽이 없는 구역에 앞 구역의 바탕쪽을 이어받는 변경이다. 최신 devel에서도 머리말 표가 용지 밖으로 나가고 둘째 구역의 바탕쪽이 빠져 있었다.

처음에는 BOX의 윗변·글자 기준선 약 4px 차이를 이유로 [재작업 요청](https://github.com/edwardkim/rhwp/pull/8165#issuecomment-6512733284)을 게시했다. 하지만 그 차이를 PR이 만든 회귀로 확정하지 않은 채 기여자에게 재작업을 요구했다. 기여자의 원 설명에도 선언 높이 16px와 한컴 출력 약 19.5px 차이가 명시되어 있었다. 사용자는 개선 성과를 수용하고 첫 기여를 머지하도록 지시했다. 이 요청에 따라 알려진 차이의 수용 결정을 기록하고 보류를 해제했다.

## 실제 검증과 한계

| 대상 | base Native | 후보 Native | 후보 fresh WASM | 쪽수 |
| --- | ---: | ---: | ---: | --- |
| 머리말·꼬리말 틀 | 23.30% | 91.11882% | 91.03647% | 1/1 |
| 바탕쪽 p1 | 100% | 100% | 100% | 2/2 |
| 바탕쪽 p2 | 21.35% | 100% | 100% | 2/2 |

[전쪽 TSV·입력/PDF SHA·대표 review/overlay](../assets/pr_8165_maintainer/README.md).
머리말 표 복원, 꼬리말 틀 배치, 둘째 구역 바탕쪽 표시를 Native와 fresh WASM에서 직접 판독했다. 머리말·꼬리말 틀과 바탕쪽 집중 회귀도 통과했다.

위 실행은 누적 후보 `cffa5c8619662945ba82bd1a0798ac3347884425`의 증거다. 이번 승인 뒤 독립 #8165 후보의 새 빌드·전체 회귀를 실행한 것으로 기록하지 않는다. BOX 차이의 근본 원인과 독립 후보 추가 검증을 완료했다고 확대 해석하지 않는다.

원 head의 [Full CI 39877572532](https://github.com/edwardkim/rhwp/actions/runs/39877572532), Native Skia, lint, A/B/C/D, CodeQL 개별 언어, Adapter, Proptest, Render Diff 성공을 재확인했다. code head 변경이 없고 current-base merge가 clean이므로 전체 Cargo/Native 광범위 검증을 중복 실행하지 않았다. merge 직전 head·현재 devel ref·check 상태를 다시 조회하고 `--match-head-commit`으로 원 head를 고정했다.

## 조판 계약과 입력

`header_footer_frame`과 `header_footer_table_frame_origin`이 머리말·꼬리말 틀을 계산하고, `layout_table`의 frame origin 및 `layout_shape`의 기준 영역으로 전달한다. 본문 표·TAC 표·문단 세로 기준 표에는 이 표 원점 경로를 적용하지 않는다. 바탕쪽은 `src/document_core/queries/rendering.rs`의 구역 할당에서 기존 carry를 소비한다. 선언 높이 1200HU 고정 검사 통과만으로 실제 BOX 형상 일치를 증명하지 않았다. 이 남은 차이가 이번 메인터너 수용 결정의 명시적 한계다.

두 공개 원본은 PR head의 samples 경로에 포함되며 입력 hash를 증거 JSON과 대조했다. 독립 한컴 Print와 source 기여자 캡처는 구분한다. 원 PR은 관련 이슈 없음, Refs #6608이며 #6608을 이번 범위에서 닫지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 첫 기여 감사, 머지 SHA, 알려진 BOX 차이의 수용, 실제 검증 범위를 게시한다. 원 PR의 시각 asset은 code merge SHA로 고정하고, 메인터너의 별도 비교 이미지는 운영 기록 commit SHA로 고정한다. 두 출처의 수치를 섞지 않는다. 게시 후 API 본문과 asset blob을 확인한다.

## 후속 처리

maintainer 일반 경로의 운영 기록으로 archive review·시각 asset·오늘할일을 devel에 직접 반영한다. contributor fork branch는 보존한다. 알려진 BOX 차이는 향후 개선 대상으로 남으며 이 작업에서 새 이슈·기능 수정으로 확대하지 않는다.
