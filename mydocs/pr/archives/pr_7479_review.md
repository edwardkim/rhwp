---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7479 검토 — fix(layout): 빈 host 줄을 문단 기준 그림 띠에 흡수하고 넘치는 그림을 사다리대로 넘긴다 (#7470 1단계)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 통합 코드·직접 시각 검증은 아직 실행 전입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7479, 작성자 planet6897, base `devel`, 정확한 head `1d894b4a05c4e5cf82b428e71ef0ab22de59c1ae`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `61dac2e5f14c206bc3eb293a67879051476c7004` | candidate | fix(layout): 빈 host 줄을 문단 기준 그림 띠에 흡수하고 넘치는 그림을 사다리대로 넘긴다 (#7470) |
| `1d894b4a05c4e5cf82b428e71ef0ab22de59c1ae` | candidate | 증적(#7470): 빈 host 그림 띠 흡수 Visual Sweep 대표 쪽 review·overlay와 한/글 2024 기준 PDF |

## 변경 범위와 검토 계획

- 원 PR 변경 295줄 추가·9줄 삭제·12파일입니다.
- 생산 경로: `src/renderer/layout.rs`, `src/renderer/mod.rs`, `src/renderer/typeset/section/controls.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

체리픽·충돌 보정·최종 검증은 아직 미실행입니다. 분석·코드·결과 보고·commit 순서로 단계별 갱신합니다.
