---
doc_kind: pr_review_impl
title: "PR #6541 planet6897 연속 체리픽 통합 수행계획"
status: archived
pr: 6541
reviewed_at: 2026-09-01
---

# PR #6541 planet6897 연속 체리픽 통합 수행계획

## 1. 목적과 운영 경계

#6541을 `planet6897`의 현재 열린 기여를 검증하는 제한된 rolling integration PR로 사용한다.
원 contributor PR을 개별 merge하지 않고, 원 commit을 작성자·계보를 보존한 채 integration branch에
체리픽한다. 메인테이너·collaborator 보정은 별도 commit으로 분리한다.

- 통합 branch: `review/planet6897-6514-6536-20260831`
- 초기 원격 integration head: `7d2aec2d6949305f4dbd9f3b145cba8b37541aa6`
- 최종 integration head: `88d0924e550041746464627bc7bc32b1a2511177`
- 현재 기준 `upstream/devel`: `336c4526e9cc5047d6dd9906ebc8d0d5ee6f2188`
- 최종 상태: PR #6541 `MERGED`, merge commit `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`
- 원 contributor branch: 수정·rebase·force-push하지 않는다.
- remote push, Ready 전환, review 게시, merge, close는 각각 별도 승인을 받는다.
- 이번 배치 candidate가 고정된 뒤 등록되는 새 PR은 다음 integration PR로 넘긴다.

## 2. 고정 배치와 계보 원장

| 순서 | 원 PR | 접수 head | 현재 integration 상태 | 기능 축 |
|---:|---:|---|---|---|
| 1 | #6514 | `b643b3822edccaa234133fc4cf2701910b090b8f` | `c8708e2d8` 적용 → `ad877288b` 보정 | 자간 fit-test |
| 2 | #6536 | `8e4269db82cae5a45115f332c2fb80a467a45f32` | `b8041f23c` 적용 → `0ff2e25b6`, `7cf17c1ce` 보정 | 쪽-앵커 표 흐름 |
| 3 | #6543 | `a3e1f514b9b7d52902b62f95a21f9b492745f674` | `3955515d3`, `7e903460d` 적용 | 한컴 font face chain |
| 4 | #6546 | `581740ccb1581f6cb9b17bf73ed00d49fd5e6647` | `a1648ea87` 적용 → `cae16410d` 원장 | 문단 내부 vpos 되감김 |
| 5 | #6548 | `578afd06265a664584ab9516af47342ce54ecc26` | `604230770` 적용 → `cae16410d` 원장 | anchor-delay 1 ULP |
| 6 | #6552 | `f7aa7d4c6d5052d4598825ff0f841f7cc919cea2` | `ddb6f43f1` 적용 | 미주 reset 사다리 |

체리픽 직전 `gh pr view`와 `git ls-remote`로 head를 다시 비교한다. 새 head가 생기면 위 접수 SHA의
검증을 재사용하지 않고 diff·CI·patch-id를 갱신한다.

## 3. 현재 blocker와 기존 기록 정정 대상

### #6514

- `#[doc(hidden)] pub`은 문서에서만 숨을 뿐 downstream에 공개된 test-only API다.
- 양수 자간 trim을 실제 glyph ink·공개 조판 경로·오라클 없이 최종 계약으로 단정했다.
- 당시 active 경로의 `pr_6514_review.md`와 기존 통합 기록의 `승인` 표현은 최신 메인테이너 판정과 맞지 않았다.

### #6536

- 원 patch는 1페이지를 복원하지만 `2.` 본문을 표 뒤로 이동시킨다.
- `0ff2e25b6` 보정 뒤 render tree에서도 `연번` 표는 `y=456.2, h=178.4, bottom=634.6`,
  `끝.`은 `y=600.1`이라 표의 세로 범위와 겹친다.
- 현 회귀 테스트의 `body_y < table_y && table_y < ending_y`는 table top만 확인한다.
- 당시 active 경로의 `pr_6536_review.md`와 기존 통합 기록의 “보정 완료”·“수용 가능” 표현은 정정 대상이었다.

기존 review 문서를 먼저 사실과 다르게 유지한 채 merge하지 않는다. 다만 code candidate를 고정하기 전에
여러 번 review-only commit을 추가하지 않고, 현재 Draft comment와 이 계획서로 상태를 표시한 뒤 최종
trailing 기록에서 원 PR별 판정을 한 번에 현행화한다.

## 4. 단계별 수행계획

### Stage 0 — integration 기준선 동기화

1. collaborator 원격 head가 `7d2aec2d6`인지 재확인한다.
2. local integration worktree가 clean인지 확인한다.
3. 최신 `upstream/devel`을 integration branch에 merge한다. rebase하거나 contributor commit을 재작성하지 않는다.
4. conflict, merge tree, `git diff --check`, PR 고유 diff를 확인한다.
5. 이 단계 결과를 commit으로 고정한 뒤 다음 단계 승인을 받는다.

Rollback: 아직 push하지 않은 integration merge만 `git merge --abort`한다. 완료된 collaborator commit이나
원 contributor history에는 reset·amend를 사용하지 않는다.

### Stage 1 — #6514 blocker 보정

1. 공개 test-only module 없이 실제 공개 조판 경로로 양수·음수 자간 fit을 재현할 수 있는 최소 fixture를
   우선 탐색한다.
2. public behavior로 불가능하면 private 불변식 예외와 의도된 내부 crate 경계를 별도 설계·승인받는다.
3. 실제 줄 나눔, pen advance, glyph ink를 분리해 characterisation과 correctness oracle을 구분한다.
4. focused test와 기존 #5678 잔여 과제를 분리하고 #5678은 자동 close하지 않는다.

Gate Q1: 테스트 경계와 오라클 근거 승인 뒤에만 보정 commit을 만든다.

### Stage 2 — #6536 blocker 보정

1. 회귀 조건을 `body_bottom <= table_top`과 `table_bottom <= ending_top`처럼 비겹침까지 고정한다.
2. `2.` 본문 → `연번` 표 → `끝.` 순서와 각 bbox의 비겹침을 모두 만족하도록 flow 소비를 보정한다.
3. Hancom 2020 기준 PDF physical page 1과 재비교하고 대표 패널을 새로 만든다.
4. #6535의 7건 중 현재 단계가 해결하는 범위를 유지하고, 남은 사례 때문에 이슈는 open으로 둔다.

Gate Q2: 사람이 직접 확인한 순서·비겹침 판정 승인 뒤 다음 PR을 적용한다.

### Stage 3 — #6543 font face chain 적용

1. 원 commit `1749531a9`, 후속 test commit `a3e1f514b` 순서와 patch-id를 확인해 체리픽한다.
2. SVG golden 변경이 font chain 문자열 외 좌표·glyph 배치를 바꾸지 않는지 재확인한다.
3. `HY신명조`, `HY헤드라인M`, `HY그래픽M`의 host face 해석과 fallback 결과를 검증한다.
4. #6514 자간 fixture와 함께 실행해 실제 face metric 변화가 줄 나눔 계약을 흔들지 않는지 교차 확인한다.

### Stage 4 — #6546·#6548 저장 vpos 묶음 적용

1. #6546 `581740ccb`를 먼저 적용하고 focused #6542와 시각 p7을 확인한다.
2. #6548 `578afd062`를 적용한다.
3. 예상된 `tests/fixtures/ir_field_sweep_baseline.tsv` conflict는 양쪽 행을 기계적으로 합치지 않는다.
4. 두 신규 fixture를 포함한 IR sweep dump를 다시 만들고, 실제 비영 발산만 사전순 원장으로 반영한다.
5. `oracle_page_count`, `off_canvas`, `text_overlap`, #2070 315쪽 핀을 누적 head에서 확인한다.

Rollback: 원 PR별 cherry-pick과 integration 전용 baseline 보정을 별도 commit으로 두고, 실패 시 마지막
integration commit만 revert한다. source PR history를 변경하지 않는다.

### Stage 5 — #6552 미주 사다리 적용

1. `f7aa7d4c6`을 적용하고 #6546의 vpos rewind 해석과 충돌하는지 code·fixture로 교차 확인한다.
2. #6545 p23 겹침 금지·저장 진행량 focused test를 실행한다.
3. `3-09월_교육_통합_2022.hwpx` p23을 Hancom 2024 오라클과 비교한다.
4. 형제 문서 `3-10월_교육_통합_2022.hwpx` 18쪽 무회귀를 확인한다.

### Stage 6 — 누적 candidate 전체 검증

동일 worktree와 `target/pr-review`에서 Cargo 명령을 순차 실행한다.

1. integration manifest prepare/check
2. `cargo fmt` 및 native/WASM/workspace all-target Clippy `-D warnings`
3. workspace build
4. 원 PR 6건 focused test와 관련 baseline
5. release-test 전체 nextest
6. Native Skia 3종
7. locked Docker WASM build
8. font face browser 계측과 SVG golden 결정성
9. 문서별 Hancom 시각 검증 및 대표 asset 직접 열람

광범위 CI가 녹색이어도 다음 직접 판정을 생략하지 않는다.

| 원 PR | 직접 확인 대상 |
|---:|---|
| #6514 | 양수·음수 자간의 실제 줄 나눔·ink 경계 |
| #6536 | Hancom 2020 p1 본문·표·종결문 순서와 비겹침 |
| #6543 | 세 face의 실제 host 선택과 glyph/폭 변화 |
| #6546 | Hancom 2022 p7 본문 하한과 페이지 경계 |
| #6548 | 대상 문서 2쪽 복원과 인접 page-count 핀 |
| #6552 | Hancom 2024 p23 미주 수식·후속 문단 비겹침 |

### Stage 7 — candidate freeze와 review 기록

1. 각 원 PR의 최종 source SHA → integration commit SHA → 보정 SHA를 원장에 확정한다.
2. `pr_6514_review.md`, `pr_6536_review.md`의 오래된 판정을 고치고 #6543/#6546/#6548/#6552 기록을 추가한다.
3. 실제 검증 결과와 시각 asset을 하나의 trailing review commit으로 반영한다.
4. code candidate 이후에는 허용된 review-only 범위만 추가한다.

### Stage 7 실행 결과 — local candidate

- code candidate: `ddb6f43f1d606918886fa6881af06e3c89183dc0`
- 통합 기준: `upstream/devel@336c4526e9cc5047d6dd9906ebc8d0d5ee6f2188`
- 원 PR 여섯 개의 원격 head는 접수 SHA에서 변하지 않았고 확인 시점의 GitHub checks는 모두 green이다.
- mandatory Rust lint bundle과 unit-tier gate를 통과했다.
- release-test 전체 nextest: `8,914 passed`, `46 skipped`, 실패 0, 374.273초
- Native Skia: lib 전체와 placeholder 2/2, direct PDF 4/4 통과
- locked Docker WASM: wasm-pack 0.15.0, 최적화 포함 6분 52초, `/app/pkg` 생성 성공
- Chrome 151 host font 계측에서 세 face 모두 접미사 제거명만 fallback 기준선과 다른 glyph/폭을 냈다.
- 문서별 직접 판정과 잔여 범위는 각 검토 기록에 고정했다:
  [#6514](pr_6514_review.md), [#6536](pr_6536_review.md), [#6543](pr_6543_review.md),
  [#6546](pr_6546_review.md), [#6548](pr_6548_review.md), [#6552](pr_6552_review.md).

이 문단은 Stage 7 완료 시점의 기록이다. 당시에는 원격 push·Draft 해제·review 게시·merge를 아직
수행하지 않았고, trailing review commit 이후의 SHA를 최종 local candidate로 동결해 Q7 원격 승인을
별도로 받았다.

### Stage 8 — 원격 통합

1. 작업지시자 push 승인 뒤 collaborator 원격 head를 다시 fetch한다.
2. fast-forward 가능한지와 예상 밖 commit이 없는지 확인한 뒤 push한다. force-push하지 않는다.
3. 최신 #6541 Full CI와 mergeability를 확인한다.
4. 최종 review와 작업지시자 승인 뒤 Draft를 해제한다.
5. Ready 상태 최신 head의 required check를 다시 통과한 뒤 정상 merge commit 방식으로 병합한다.

### Stage 8 실행 결과 — merged

- 최종 head `88d0924e550041746464627bc7bc32b1a2511177`의 메인테이너 self-review를 게시하고
  Draft를 해제했다.
- code candidate의 full CI 이후 trailing review-only head는 fast-pass로 성공 11건, 정책상 skip
  20건, 실패·대기 0건을 확인했다.
- PR #6541은 2026-09-01에 정상 2-parent merge commit
  `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`로 `devel`에 병합됐다.

### Stage 9 — 원 PR·이슈 후속 처리

1. #6541 merge SHA와 `devel` 반영을 확인한다.
2. 각 원 PR에 source SHA → integration SHA → merge SHA 계보와 검증 결과를 게시한다.
3. 원 PR은 개별 merge하지 않고 integration 완료 사유로 close한다.
4. 이슈는 실제 해결 범위로 별도 판단한다. 부분 해결 이슈는 open으로 유지하거나 후속 이슈를 만든 뒤
   closure 관계를 명시한다.
5. integration worktree·review target·agent 생성 임시 산출물은 종료 게이트에서 정리한다.

Stage 9의 원 PR·이슈 comment/close와 local cleanup은 이 archive 기록이 `devel`에 반영된 뒤 별도 승인
범위에서 수행한다. 이 문서의 merge 완료 기록 자체는 그 원격 조치를 수행했다는 뜻이 아니다.

## 5. 승인 게이트

| Gate | 승인 대상 |
|---|---|
| Q0 | 최신 `devel` integration merge와 conflict 결과 |
| Q1 | #6514 테스트 경계·오라클 보정안 |
| Q2 | #6536 bbox 비겹침 구현과 시각 판정 |
| Q3 | #6543 적용·수평 조판 결과 |
| Q4 | #6546/#6548 conflict 해소와 IR 원장 결과 |
| Q5 | #6552 적용·미주 오라클 결과 |
| Q6 | 전체 누적 검증·candidate freeze |
| Q7 | remote push와 최신 Full CI |
| Q8 | Ready 전환·최종 self-review·merge |
| Q9 | 원 PR close·issue 후속·cleanup |

이 계획 승인만으로 다음 Gate의 GitHub mutation이나 merge를 자동 승인한 것으로 해석하지 않는다.
