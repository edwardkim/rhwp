# PR #6999 메인테이너 보정·병합 수행계획

- 작성일: 2026-09-11
- 관련 이슈: #6869, #6871
- 선행 검토: [pr_6999_review.md](pr_6999_review.md)
- 메인테이너 결정: 기여자에게 재작업을 요청하는 대신 우리 쪽에서 보정 후 병합한다.
- 현재 단계: S1~S4 및 self-review 완료. 검토 기록 반영·최종 CI 확인 후 merge commit 병합을 승인받음.

## 1. 경로와 고정 기준

기본 경로는 `maintainer_general`을 유지한다. 기존에 읽은 intake/local-validation/visual-fixture/
rework 문서 외에 `collaborator_external_pr.md`를 읽었으며, 그 문서의 **9.3.1 직접 source 보정**과
**9.3.1.4 최신 devel 호환 보정**을 적용한다. collaborator의 squash 예외를 메인테이너 병합에
일반화하지 않는다. 최종 병합 방식은 merge commit이다.

| 항목 | 재확인 값 |
| --- | --- |
| PR | #6999, OPEN |
| source repository / branch | planet6897/rhwp / `fix/6869-hwpx-page-num-pos-collapse` |
| maintainerCanModify | true |
| 원 PR head / fork remote ref / fetched ref | 모두 `d563b48d761a4382106a00fd8daa90d7fb846222` |
| 최신 fetch devel | `6806950b1ab57e6d97978b0986d63c689b12457e` |
| 충돌 | `src/serializer/hwpx/section.rs`, 형광펜 처리와 render_runs 변경 |

원 기여 커밋은 다음 순서와 SHA를 보존한다.

1. `8bf8211df45fb8d4e293c97b89b91d857a20fc2f`: 문단당 쪽번호 위치 하나로 축소.
2. `d563b48d761a4382106a00fd8daa90d7fb846222`: HWPX 출처에도 접힌 슬롯의 축 보정 적용.

새 PR 생성·cherry-pick·contributor rebase/amend/force-push는 하지 않는다.
구현 착수 때 clean한 주 작업공간에서 원 PR head를 가리키는 단일 가시성 브랜치
`review/planet6897-pr6999`를 만들어 이후 작업을 이어간다. 기존 #6999 detached review worktree의
진단 자료와 #6996 검토 자료는 보존한다. 추가 worktree는 만들지 않는다.

## 2. 변경 범위와 보호할 계약

핵심은 `hp:pageNum` 방출 여부의 **문단 호출 스코프**다. 문단 종류별 분기를 추가하지 않는다.

- 각 문단은 독립된 `false` 상태로 시작한다.
- 자식 문단을 저장한 뒤 부모가 이어서 실행될 때 부모의 기존 상태를 정확히 복원한다.
- 빈 문단과 조기 반환에서도 복원 경로를 빠뜨리지 않는다.
- 같은 문단의 중복만 제거하고 다른 문단의 첫 쪽번호는 보존한다.
- 실제 접힌 슬롯만 그 문단의 축에서 차감한다. 자식의 슬롯을 부모에게 계상하지 않는다.
- HWPX에 원래 없던 secd/cold의 이중 차감 방지와 기존 형광펜의 비점유 축 계약을 유지한다.

최소 구현은 `render_runs`를 문단 스코프 진입·복원 wrapper로 두고 기존 본체를 내부 함수로
분리하는 방식이다. 진입 시 부모 상태를 저장하고 false로 설정하며, 본체가 반환하면 부모 상태를
복원한다. 기존 본체의 여러 return은 모두 wrapper로 돌아오므로 개별 분기에 복원 코드를
중복하지 않는다. 정상 반환 경로의 계약이며, panic 복구 같은 별도 기능은 추가하지 않는다.

렌더러·IR 모델·HWP 파서·폰트·CI 정책·기존 baseline은 이번 보정에서 바꾸지 않는다.
한컴이 서로 다른 쪽번호 설정 중 어떤 것을 선택하는지에 대한 별도 정책 확장은 하지 않는다.

## 3. 수행 순서

### S1 — 최신 devel 통합과 충돌 해소

착수 직전 source 세 SHA와 devel을 다시 확인한다. 기여자가 새 commit을 올렸으면 기존 head에
계속 보정하지 않고 새 diff부터 확인한다. 같은 가시성 브랜치에서 devel을 merge하고,
형광펜 처리와 PR의 반환값·슬롯 집계를 모두 보존하도록 충돌을 해소한다.
충돌 해소는 별도 merge commit으로 남기고, PR 고유 diff는 `upstream/devel...HEAD`로 확인한다.

### S2 — 문단 스코프 격리와 회귀 테스트

위 wrapper 방식으로 원인을 보정한다. reviewer probe를 정식 `tests/cases/` 회귀 테스트로
정리하며 source 본문에 테스트를 추가하지 않는다.

필수 검증 사례:

1. 부모 쪽번호 → 일반 텍스트 셀의 표 → 부모 쪽번호: 부모 1, 셀 0.
2. 쪽번호가 있는 셀의 표 → 부모 첫 쪽번호: 부모 1, 셀 1.
3. 나란한 본문 문단의 독립성, 빈 자식 문단의 조기 반환, 2단계 중첩.
4. 같은 공용 경로를 쓰는 머리말 또는 글상자에서도 부모 상태 복원.
5. HWP5/HWPX 출처 각각의 접힌 슬롯 textpos 보정, secd/cold 이중 차감 방지.

기존 실패 2건이 통과로 바뀌고 대조군이 계속 통과하는지 확인한다. 합성 IR은 serializer 계약
검증 자료로만 쓰며 한컴 유효성 검증을 받은 원본 샘플이나 시각 정답지로 취급하지 않는다.
보정 code/test는 devel 통합과 분리된 commit으로 남긴다.

### S3 — 관련 범위와 제출 게이트

- #6869·#5943 focused test 및 최신 devel 형광펜 저장 회귀를 실행한다.
- review 검증 환경에서 manifest `--prepare` 후 fmt, native Clippy, WASM32 lib Clippy,
  workspace build, workspace all-target Clippy를 순차 실행한다. 명령의 정본은
  [local_validation 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)이다.
- 새 code head이므로 기존 녹색 CI를 재사용하지 않는다. focused 결과 보고 후 전체 회귀 승인
  게이트에 따라 `--locked` release-test 전체 nextest를 실행한다. 공유 review target은 이동하거나
  삭제하지 않고 Cargo 명령은 순차 실행한다. 파생 suite/manifest는 제출하지 않는다.
- #6871의 다섯째 사례는 확보된 입력과 기존 근거부터 확인한다. 같은 결함인지와 내용 보존을
  확인하지 못했으면 이슈 전체 완료로 선언하지 않는다. 별도 원인까지 이 PR에 확대 구현하지 않는다.
- 작업 문서의 각주 집계 기각 결론·브랜치 이름을 실제 PR 및 확인된 근거와 일치시킨다.
- 실제 샘플로 추가 한컴 검증이 필요하면 기존 자료를 재사용하고 최소 대상부터 수행한다.
  저장 구조 수정에 무관한 renderer 전수 시각 스윕·Native Skia 전체·Studio 빌드는 추가하지 않는다.

### S4 — 원 PR 갱신과 CI

로컬 결과와 원 PR/보정 head를 구분하여 보고하고 원격 push 승인을 받는다.
직전 fork SHA가 시작 source와 같은지 재조회하고 LFS 대상 판독·dry-run 후
기여자 branch에 추가 commit을 정상 push한다. source가 달라졌으면 push를 멈춘다.
새 head의 Full CI와 필요한 CodeQL/required checks를 확인한다. 이번 code 보정을
review-only fast-pass로 처리하지 않는다. 필요한 PR 본문·comment 현행화도 게시 승인 후 수행한다.

### S5 — self-review와 merge commit 병합·후속 정리

원 코드와 메인테이너 보정의 최종 diff를 self-review하고, 최신 head·CI·mergeability 및 병합
승인을 확인한 뒤 **#6999 자체를 merge commit 방식으로 병합**한다.
원 기여 이력·메인테이너 보정·최종 merge SHA를 연결해 기록한다.
이슈는 각 요건의 충족 여부를 확인해 종료하며, #6871의 다섯째 사례가 미확인이라면 자동 종료를
유발하지 않도록 사전에 closing 문구를 현행화한다. 이후 devel 동기화와 이번 작업의 전용
branch/worktree 정리는 `post_merge.md`를 읽고 적용한다. #6996의 미완료 검토는 정리하지 않는다.

## 4. 승인 요청 및 중단 기준

이번 메인테이너 지시는 **우리 쪽 보정 후 병합 경로 선택**으로 기록했다. 이 상세 계획의
S1~S2 로컬 통합·구현 승인을 받았다. 원격 push/comment 및 최종 병합은 해당 시점의
검증 결과와 함께 확인한다.

다른 source 파일까지 충돌이 확대되거나, 한컴의 쪽번호 선택 정책 변경이 필요하거나,
새 회귀를 baseline 완화로 숨겨야 하는 상황이면 보정을 확대하지 않고 근거를 보고한다.
되돌릴 필요가 있으면 메인테이너 소유 변경만 별도 revert 대상으로 삼고 contributor history와
사용자 WIP를 reset/삭제하지 않는다.

## 5. S1~S2 실행 기록

- 착수 시 source SHA, fork branch, PR head는 모두 기존 `d563b48d761a4382106a00fd8daa90d7fb846222`와
  일치했다. devel은 `6806950b1ab57e6d97978b0986d63c689b12457e`였다.
- 주 작업공간은 `review/planet6897-pr6999`로 전환했다. 로컬 devel 포인터와 #6996 worktree는 보존했다.
- S1 merge commit: `9c94c5fe6`. 예상한 `section.rs` 한 곳만 수동 충돌 해소했다.
  devel의 `PositionedMarkpens`와 PR의 슬롯 집계·반환값을 모두 보존했다.
- 통합 중 첫 부모 대비 `git diff --cached --check`는 devel에서 유입된
  `scripts/renderer_baseline_manifest.json`의 기존 CRLF를 경고했다. 이 파일을 정정 대상으로
  확장하지 않았다. 실제 PR 기준 `git diff --check upstream/devel...HEAD`는 통과했다.
- S2 code/test commit: `2b479c8b31735278afe2452c6740f26fba25eee9`.
  `render_runs` wrapper가 부모 bool을 저장·false로 진입·본체 실행·부모 값 복원을 수행한다.
  문단 종류별 예외는 추가하지 않았다. production 변경은 context 주석 1줄과 section wrapper다.
- 회귀 원본 `tests/cases/issue_6869_page_num_pos_nested_scope.rs`에 7건을 추가했다.
  부모/자식 중복·첫 컨트롤 소실, sibling, 빈 자식, 2단계 중첩, 머리말, 부모/자식 textpos를
  HWP5/HWPX 두 출처에서 검사한다. secd/cold는 기존 #5943 focused test로 함께 확인한다.
- 기존 #6999 검증 worktree를 이 code commit의 detached HEAD로 갱신했다. 원본 probe와
  검토 문서는 보존했다. 이 worktree에서만 manifest를 준비했다(48/48 targets).
- 전체 `cargo fmt --all -- --check`가 통과했다. 신규 focused 7/7 통과(두 출처를 각 test에서 검사),
  build 2분 11초, test 0.010초다. 첫 검토에서 실제 실패한 부모 중복/소실 두 경로도 통과했다.
  suite 배정 Node 계약 검사도 23/23 통과했다. 인접 focused 결과는 검토 보고서에 기록한다.
- S3 전체 검증과 원격 작업은 아직 완료하지 않았다. 원 PR의 현재 head는 변하지 않았다.

## 6. S3 실행 기록 (2026-09-11)

- 검증 code head는 `2b479c8b31735278afe2452c6740f26fba25eee9`로 유지했다.
  review worktree에서 fmt, native Clippy, WASM32 lib Clippy, workspace build,
  workspace all-target Clippy, manifest check를 순서대로 모두 통과했다.
  각 명령은 `--locked`와 고정 공유 target을 사용했다.
- 첫 lint 실행의 도구 출력이 잘려 결과 수신을 잃었으므로, 해당 실행 종료 후 캐시를
  재사용해 같은 묶음을 재확인하고 `output/pr6999-s3/*.log`에 보존했다.
  마지막 로그 표시용 `tail -2`만 다중 파일에서 실패했으며, `tail -n 3`으로 확인했다.
  `set -e` 아래 순차 실행된 검증 명령들은 모두 성공했고, 이 표시 오류와 구분한다.
- 전체 회귀 명령:

  ```bash
  cargo nextest run --locked --cargo-profile release-test \
    --target-dir /home/edward/mygithub/rhwp-shared-review-target \
    --tests --no-fail-fast
  ```

  호스트 논리 CPU 16개, 메모리 31 GiB 중 가용 약 28 GiB, 다른 Cargo 작업 종료를
  확인하고 기본 동시성을 사용했다. 로그는 `output/pr6999-s3/nextest.log`다.
  최종 결과는 9459/9459 PASS, 46 skipped, 5 slow, exit 0이다. 빌드 3분 41초,
  테스트 390.014초이며 run ID는 `bb15a220-0648-442e-beac-ae676c64d189`다.
  마지막 대형 표 테스트도 166.963초에 통과했다. baseline·기대값은 변경하지 않았다.
- #6871 다섯째 `00961` 원본을 기존 로컬 코퍼스에서 찾았다. XML의 가장 가까운 문단
  소유자 기준으로 첫 문단에 `pageNum` 4개가 존재한다. 현재 보정 바이너리의 x2x 저장은
  4→1개로 줄이고 해당 문단의 마지막 LineSeg textpos를 56→32로 보정했다.
  문단 37개·표 4개·그림 5개와 순서대로 이은 section 텍스트 968자가 보존됐다.
  텍스트 SHA-256은 전후 모두
  `c06083b9106772de2fa5fe1f22b6e4a3508dcfe440bb417332975c5bfd75c1b3`다.
  산출물은 로컬 `output/pr6999-s3/00961-corrected.hwpx`이며 원본을 변경하지 않았다.
- 위 검사는 저장 구조 보존 근거다. 한컴이 원본에서 누락하던 29행을 실제로 복구해서
  읽는지 또는 시각적으로 온전한지는 미검증이다. 따라서 #6871 전체 종료 근거로 확대하지
  않는다. PR 본문의 자동 종료 문구 변경은 게시 승인을 받은 후 처리한다.
- 작업 문서 정정을 별도 `cd1a11dc06eff3f867c921c2a509b078cbf21871`로 커밋했다.
  code head 이후 차이는 `mydocs/working/issue_6869_page_num_pos_collapse.md` 하나다.
  전체 회귀 종료 후 review worktree도 이 문서 head로 맞추고 fmt·세 Clippy·workspace
  build·manifest를 다시 통과시켰다. source/test/Cargo/CI diff 없음도 확인했다.
  문서-only 변경은 local_validation 4.3에 따라 전체 Cargo 회귀를 다시 반복하지 않았다.
- 마지막 실제 Git fetch/FETCH_HEAD/ls-remote의 devel은 모두 `6806950b1`로 기존 기준과
  같았다. `git merge-tree --write-tree HEAD upstream/devel`은 충돌 없이 exit 0이다.
  원 PR head는 여전히 `d563b48d7`, maintainerCanModify=true다. 원격 변경은 하지 않았다.
- 다음 승인 요청은 원 기여자 branch에 정상 push하고, PR 본문의 메인테이너 보정·검증
  결과와 #6871의 미검증 종료 범위를 현행화하는 것이다. 병합은 새 head CI 확인 후 별도 절차다.

## 7. S4 원격 적용 기록 (2026-09-11)

- 메인테이너가 원 기여자 branch push 및 PR 본문 현행화를 승인했다.
- 직전 Git fetch의 devel은 `6806950b1`, PR head와 fork ref는 모두 `d563b48d7`로
  시작점과 같았다. maintainerCanModify=true, merge-tree와 diff check도 통과했다.
- 원 head→제출 head 변경 경로 211개의 filter는 모두 unspecified였고 `git lfs status`에
  업로드 대상이 없었다. devel 통합분이 포함된 이 211개와 실제 PR 고유 6개 파일을 구분한다.
- `GIT_LFS_SKIP_PUSH=1 git push --dry-run` 통과 뒤 동일 옵션으로
  `https://github.com/planet6897/rhwp.git`의 `fix/6869-hwpx-page-num-pos-collapse`에 정상 push했다.
  force-push·새 PR·추가 source 변경은 하지 않았다.
- fork ref, PR head, local HEAD 모두 `cd1a11dc06eff3f867c921c2a509b078cbf21871`로 확인했다.
  `upstream/pr6999-head`도 재조회해 갱신했으며 기본 작업트리는 clean이다.
- PR 본문은 기존 기여자 조사 기록을 보존하고 상단에 메인테이너 보정·검증을 추가했다.
  #6871 자동 종료 문구와 '함께 닫습니다' 제목을 정정했다. #6869 연결은 유지했다.
  갱신 전·후 사본은 review worktree의 `output/pr6999-s3/pr6999-body-{before,after}.md`다.
- 로컬 gh의 `pr edit`가 Projects classic GraphQL 오류로 실패해, `gh api --method PATCH
  repos/edwardkim/rhwp/pulls/6999 -F body=@<파일>`로 같은 승인 작업을 수행했다.
  직전 본문 불변 확인 후 게시했으며, 게시 뒤 API 본문이 로컬 사본과 일치하고 BOM·치환 문자 및
  #6871 closing 문구가 없음을 확인했다. 별도 comment는 게시하지 않았다.
- 새 head 실행: CI [34564676991](https://github.com/edwardkim/rhwp/actions/runs/34564676991),
  CodeQL [34564676946](https://github.com/edwardkim/rhwp/actions/runs/34564676946),
  Adapter [34564676945](https://github.com/edwardkim/rhwp/actions/runs/34564676945),
  Proptest [34564676966](https://github.com/edwardkim/rhwp/actions/runs/34564676966)는 실행 중이다.
  Controller [34564675484](https://github.com/edwardkim/rhwp/actions/runs/34564675484)는 성공했다.
  CI에서 lint 및 archive A/B/C/D 실제 실행을 확인했다. review-only fast-pass가 아니다.
  무관한 WASM package·Native Skia·Frontend·promotion job은 skip됐으며 전체 통과로 기록하지 않는다.
- S4는 적용 완료·검증 진행 상태다. 새 head CI 완료 후 self-review와 병합 승인 절차로 넘어간다.
  review 문서 2개는 기존처럼 로컬에 보존하며 새 code head CI 성공 전 trailing commit으로 push하지 않는다.

## 8. S5 self-review 기록 (2026-09-11 14:35 KST)

- 현재 head `cd1a11dc0`의 CI·CodeQL·Adapter·Proptest 완료/success를 확인했다.
  CI lint·A/B/C/D archive builder·test worker·Build & Test가 실제 통과했다.
  `CI Impact Policy`도 success, 최신 GitHub 상태는 MERGEABLE/CLEAN이다.
- CodeQL Rust 분석은 통과했다. GHAS 종합은 JS/TS·Python 비교 설정 미수집 경고로 neutral이며
  annotations 0이다. 두 언어는 workflow의 선택 범위 밖이라 분석을 skip했다.
  상세 구분과 check ID는 review 문서에 기록했다. CodeQL 경고 없음으로 쓰지 않는다.
- 원 PR/보정 diff·회귀 범위·문단 호출 스코프·슬롯 차감·이슈 종료 범위를 재검토했고
  추가 차단 결함은 발견하지 못했다. 최종 검토 판정은 **현재 보정 head 승인**이다.
- 코드를 추가 변경하지 않았으므로 녹색 exact head의 전체 CI와 기존 로컬 검증을 재사용했다.
  문서만 수정했으며 추가 원격 게시나 GitHub approve/병합/issue close는 수행하지 않았다.
- 다음 승인은 검토 기록 반영 후 최종 head CI를 확인하고 #6999를 merge commit 방식으로
  병합하는 절차다. #6871은 미검증 사례가 있어 OPEN 유지한다.

## 9. 검토 기록 반영 및 병합 승인

메인테이너가 검토 기록 반영 후 최종 CI를 확인하고 merge commit으로 병합하는 절차를 승인했다.
`cd1a11dc0`의 녹색 Full CI 뒤에 review 문서 2개만 단일 부모 commit으로 잇는다.
코드·테스트·baseline·샘플은 추가 변경하지 않는다. source fork SHA와 최신 devel을 재확인하고,
문서 링크·diff·merge simulation·LFS 판독 후 정상 push한다. 최신 trailing head의 required check가
성공하기 전에는 병합하지 않는다. #6871은 종료하지 않으며, 병합 후 정리 절차는 별도로 보고한다.
