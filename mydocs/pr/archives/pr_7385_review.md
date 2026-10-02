---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7385 검토 — 수정(layout): 용지 기준 개체를 앵커 줄이 있는 쪽에 등록한다 (#5941)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7385, 작성자 planet6897, base `devel`, 정확한 head `a20cf583c76fdb038ccea6b87429a8f67b5bc832`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `8523553bf464611ba271c5d8ba2951259f09348e` | candidate | 수정(layout): 용지 기준 개체를 앵커 줄이 있는 쪽에 등록한다 (#5941) |
| `1118c7a1d914b511f53a272d668e7e800a3fe215` | candidate | 문서(pr): #5941 Visual Sweep 증적을 보존한다 |
| `a20cf583c76fdb038ccea6b87429a8f67b5bc832` | merge excluded | Merge branch 'devel' into fix/5941-paper-anchored-float-owner-page |

## 변경 범위와 검토 계획

- 원 PR 변경 173줄 추가·1줄 삭제·7파일입니다.
- 생산 경로: `src/renderer/pagination.rs`, `src/renderer/typeset/section/controls.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

### 메인터너 보정: 보류 입력의 검사 재등록 방지

- 분석: 통합 전체 nextest에서 새 검사 `non_tac_float_stays_on_its_anchor_page_so_the_text_does_not_overflow`는 입력 누락으로 실패했습니다. 원본은 없어진 것이 아니라 기존 #7382 보정66에서 `mydocs/pr/assets/issue7445/`로 이동·보존된 자료입니다. 정상 PDF302쪽과 다른 잠정304쪽·하단100px 허용·큰 노드500px 존재는 정상 소유 쪽의 독립 승인 근거가 아닙니다.
- 수정: 이 보류 입력을 다시 회귀 대상으로 삼은 새 검사만 제거합니다. HWP/PDF 증적과 제품의 비-TAC 앵커 줄 라우팅 변경, 기존 정상 대조군은 유지합니다. 정식 승인까지 원본과 실제 소유 쪽에 대한 독립 검증이 필요합니다.
- 결과: 원본 파일을 되돌려 samples 자동 수집에 넣거나 검사 기대를 현재 값으로 바꾸지 않았습니다. 전체 검증 코드 head의 최초 실패 기록은 그대로 보존하며 후속 후보에서 대조군을 재실행합니다. 현재 판정은 보류입니다.

### 누적 후보 검증 상태 갱신 (2026-10-02)

- GitHub를 다시 조회한 결과 선택 당시의 원 PR head와 CI green 상태가 유지됩니다. #7435는 현재도 non-green이며 선택에 포함하지 않았습니다. 누적 후보는 `review/planet6897-green-20261002`, source 고유 커밋63개입니다.
- 최초 전체 nextest는 10,283개 중10,250PASS/33FAIL/50SKIP, exit100으로 완료했습니다. 그 뒤 PR별 보정과 focused 재검증으로 최초 실패28개를 처리했으며 5개가 남았습니다. 전체 재실행 통과로 바꾸어 보고하지 않습니다.
- 각주 빈 번호·합성 사다리·다열 표 대조군·중첩 표 후속 원점의 잔존 5개와 whole fixture 시각 보류를 [현재 검증 기록](../assets/planet6897_green_20261002/review_progress.json)에 기록했습니다. 원 PR별 기존 분석·커밋 출처는 위 내용을 유지합니다.
- **현재 통합 승인/머지 보류**입니다. 원 PR의 green CI는 누적 후보의 실패 또는 미완료 Native/fresh WASM 시각 검증을 대체하지 않습니다. 새 통합 PR 생성·push·머지는 하지 않았습니다.
