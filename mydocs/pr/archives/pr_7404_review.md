---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7404 검토 — 수정(저장): 재래핑된 표의 저장 프레임을 실제 조판 컷으로 되쓴다 (#7114)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 통합 코드·직접 시각 검증은 아직 실행 전입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7404, 작성자 planet6897, base `devel`, 정확한 head `13cf22339f5125df51e78b361656293876313846`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `c952ef71e8da558ba70244363f9161a0f56ee543` | candidate | 수정(저장): 재래핑된 표의 저장 프레임을 실제 조판 컷으로 되쓴다 (#7114) |
| `4d80267e7ded2cd5408bcf3a61c4a3a3b56cfcf7` | candidate | 수정(저장): 중첩 표 안의 쪽 경계를 host 줄로 옮겨 적지 않는다 (#7114) |
| `13cf22339f5125df51e78b361656293876313846` | merge excluded | Merge branch 'devel' into fix/7114-save-side-frame-normalize |

## 변경 범위와 검토 계획

- 원 PR 변경 564줄 추가·26줄 삭제·9파일입니다.
- 생산 경로: `src/document_core/commands/document.rs`, `src/renderer/layout/table_layout.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

체리픽·충돌 보정·최종 검증은 아직 미실행입니다. 분석·코드·결과 보고·commit 순서로 단계별 갱신합니다.
