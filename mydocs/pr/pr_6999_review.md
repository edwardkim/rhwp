# PR #6999 검토

## 접수와 범위

| 항목 | 확인값 (2026-09-11, 병합 전 재확인 필요) |
| --- | --- |
| PR | [#6999](https://github.com/edwardkim/rhwp/pull/6999) |
| 작성자 | planet6897, 기존 contributor |
| 제목 | 수정(hwpx): 쪽번호 위치 컨트롤을 문단당 하나로 접고 축에서 뺀다 |
| 관련 이슈 | #6869, #6871 (모두 OPEN) |
| base | devel |
| 검토 head | `d563b48d761a4382106a00fd8daa90d7fb846222` |
| fetch한 devel | `6806950b1ab57e6d97978b0986d63c689b12457e` |
| 규모 | 2 commits, 5 files, +370/-19 |
| 상태 | OPEN, non-draft, CONFLICTING / DIRTY |
| reviewer | edwardkim 지정 후 API 응답 확인 |
| 기존 triage | author assignee, v1.0.0 milestone, bug/rust/hwpx/serialization/test 유지 |
| 검토 위치 | `/home/edward/mygithub/rhwp-review-6999`, detached HEAD |

기본 작업트리의 clean devel과 #6996 리뷰 워크트리는 보존했다.

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md,
           rework_and_exceptions.md (current-base conflict 확인 후 추가)
loaded documents: AGENTS.md, CLAUDE.md, mydocs/manual/codex/MEMORY.md,
  mydocs/README.md, mydocs/manual/README.md, codex/docs_and_git_workflow.md,
  pr_review_workflow.md, pr_review/README.md, 위 기본/보조 문서,
  dev_environment_guide.md
```

## 의도와 긍정적인 부분

쪽번호 위치 컨트롤의 중복 방출을 줄이면서, 제거한 8 UTF-16 유닛 슬롯만큼 저장된
LineSeg `textpos`도 보정하려는 수정이다. HWP5 전용으로 원래 방출하지 않는 슬롯과,
HWPX 출처에도 실제 존재했다가 이번에 제거하는 슬롯을 분리한 방향은 타당하다.

작성자는 한컴 2024에서 #6869 대상 문서의 44쪽 복구 및 #6871 네 문서의 복구를 보고했다.
이것은 작성자 실측 보고로 확인했으며 reviewer가 같은 한컴 변환을 재실행한 결과가 아니다.

## 발견 사항

### F1 — 문단별 중복 제거 상태가 중첩 문단과 공유된다

`src/serializer/hwpx/context.rs:126`에 추가한 `para_page_num_pos_emitted`는 공유 context의
단일 bool이다. `section.rs:1389`는 `render_runs` 진입마다 false로 초기화한다.
하지만 같은 context로 표 셀의 `render_paragraph_parts`를 재귀 호출한다
(`table.rs:376`, `section.rs`의 Table dispatch). 재귀 전후 부모 상태를 보존하지 않는다.

따라서 부모에 `쪽번호 → 표(일반 텍스트 셀) → 쪽번호`가 있으면 셀 문단이 상태를 false로
되돌려 부모의 두 번째 쪽번호도 방출하게 된다. 반대로 셀 문단이 쪽번호를 방출하고 끝나면
부모의 첫 쪽번호를 이미 방출한 것으로 오인할 수 있다. 후자는
`render_control_slot_tracked`의 collapsed-slot 판정에도 전달된다.

이 검토의 기준은 PR이 선언한 “각 문단별 하나, 다른 문단과 독립”이라는 저장 계약이다.
reviewer가 만든 IR probe는 해당 내부 계약을 검사하며 한컴 유효성 검증을 받은 샘플 문서가 아니다.
한컴 중단이나 페이지 소실까지 재현했다고 확대하지 않는다.

직렬화 성공 뒤 XML을 파싱하여 각 `pageNum`의 가장 가까운 `hp:p` 조상별로 개수를 셌다.
최종 probe run `4368d5e8-b9d5-4397-840f-a57717addb14`에서 다음 결과를 직접 확인했다.

| 구성 | 기대 [부모, 셀] | 실제 | 판정 |
| --- | --- | --- | --- |
| 부모 쪽번호 → 일반 텍스트 셀의 표 → 부모 쪽번호 | [1, 0] | [2, 0] | FAIL: 중복 방출 |
| 쪽번호를 가진 셀의 표 → 부모 첫 쪽번호 | [1, 1] | [0, 1] | FAIL: 부모 컨트롤 소실 |
| 별도 두 본문 문단의 쪽번호 (대조군) | [1, 1] | [1, 1] | PASS |

두 실패 모두 XML 직렬화 오류가 아닌 개수 단언 실패다. 공유 bool은 이번 PR에서 추가됐으므로
신규 상태 관리 결함으로 귀속된다. 별도 baseline 바이너리의 동일 probe 실행은 하지 않았다.

보완 방향: 문단 로컬 상태로 소유권을 옮기거나, 모든 재귀·조기 반환 경로에서 부모 상태를
보존·복원해야 한다. 표 외에도 글상자·각주·머리말 등 같은 호출 경로에 적용되는지 확인한다.
문단 형식이나 샘플 이름별 조건으로 회피할 문제가 아니다.

### F2 — 최신 devel과 코드 충돌, 최신 head의 전체 CI 증거 없음

```bash
git merge-tree --write-tree upstream/devel upstream/pr6999-head
```

결과는 conflict(exit 1), 파일은 `src/serializer/hwpx/section.rs`다. devel의 형광펜 처리
(`15e674ad2`, `4fc08b5e9`)와 PR의 `render_runs` 변경이 충돌한다.
충돌을 포함한 simulation tree는 `eb899a2274ea22be3ab478af1f19ed6d21731c23`이며
이 tree를 빌드 가능 후보로 사용하지 않았다. 실제 checkout에서 merge/rebase도 수행하지 않았다.

현재 head에는 CI Impact Policy Controller run `34549867907` 성공과
`CI Impact Policy` PENDING이 보였다. Full CI 성공
`34534899202`, CodeQL 성공 `34534899178`은 첫 commit `8bf8211df45fb8d4e293c97b89b91d857a20fc2f`의
결과다. x2x 축 보정이 추가된 현재 head의 통과로 대체할 수 없다.

### F3 — 이슈 종료 범위와 문서 현행화

- #6871의 [추가 댓글](https://github.com/edwardkim/rhwp/issues/6871#issuecomment-5579297425)은
  다섯째 사례 `00961.x2h`를 기록했다. PR의 복구 표는 최초 네 사례만 다루므로,
  자동 종료 전에 다섯째 사례의 결과 또는 별도 귀속을 확인해야 한다.
- PR 본문은 각주 손실이 집계 오류로 기각됐다고 정정했지만,
  `mydocs/working/issue_6869_page_num_pos_collapse.md:184`에는 여전히 각주 손실이 남았다고
  적혀 있다. 같은 문서의 작업 브랜치도 현재 PR과 다르다. 최종 결론으로 정리할 필요가 있다.

## 실행 검증

- `git diff --check upstream/devel...HEAD`: 통과.
- 제출 focused test: `issue_6869_page_num_pos_collapse` 2/2 PASS (171 filtered skips,
  build 5.05초, test 0.007초), `issue_5943_hwpx_lineseg_axis_rebase` 4/4 PASS
  (179 filtered skips, build 5.24초, test 0.008초). 모두 `run-rust-test.mjs`와 동일한
  `--cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target`을 사용했다.
- reviewer-only probe: 3건 실행, 1 PASS / 2 FAIL / 195 filtered skips, exit 100.
  최종 재빌드 6.20초, test 0.009초. 첫 cold compile은 2분 14초였다.
- 진단 준비 중 reviewer fixture의 style 등록 누락과 수정 뒤 harness 미갱신으로 각각
  직렬화 실패 및 0 tests(exit 4)가 한 번씩 있었다. 둘은 PR 결함 증적에서 제외했다.
  style 등록과 `--prepare` 재실행 후 위 개수 단언으로 재확인했다.
- 실행 명령:

  ```bash
  node scripts/rust-test-suite-manifest.mjs --prepare
  node scripts/run-rust-test.mjs pr_6999_reviewer_scope_probe -- \
    --cargo-profile release-test \
    --target-dir /home/edward/mygithub/rhwp-shared-review-target --no-fail-fast
  ```
- source·기존 test·baseline·Cargo를 보정하지 않았다. probe는 reviewer 소유 로컬 진단 자료다.
  실행 후 원본은 `output/pr6999-reviewer-scope-probe.rs`에 보존하고 테스트 수집 경로에서
  제거했다. 재현 시 해당 파일을 `tests/cases/pr_6999_reviewer_scope_probe.rs`로 복사한 후
  위 `--prepare`와 focused 명령을 순서대로 실행한다. 원 contributor PR에는 포함하지 않는다.
  probe SHA-256: `e0271bddf1e9f9473c8d04889b63b0dfc589540728654ec1b2f2e31ebb810819`.
- probe를 제외한 manifest 재준비와 `--check`: 1243 sources, 48/48 integration targets 통과.
  `git diff --exit-code HEAD -- src tests Cargo.toml Cargo.lock`도 통과했다.
- 실행 환경은 WSL2 Linux, nextest 0.9.137이다. 저장소 권고 0.9.140 미만 및
  `report-skipped` 설정 미지원 경고가 있었으나 테스트 실행/결과 수는 직접 확인했다.
- 신규/변경 sample 없음: 별도 신규 sample 보안 검사 입력 대상 없음.
- 전체 nextest, 세 Clippy, Native Skia, Docker WASM, MCP, visual sweep은 이번 접수에서
  실행하지 않았다. 코드 blocker와 충돌이 있는 후보에 광범위 검증을 추가하지 않았으며,
  이전 head의 CI 재사용으로 전체 검증 통과를 선언하지 않는다.

## 시각 검증 경계 및 Merge 후 contributor PR comment 계획

serializer 구조 보존 변경이므로 renderer 자체의 visual sweep 점수로 합격 여부를 판단하지 않는다.
현재는 한컴 복구 실측 보고만 읽었으며 직접 변환·시각 대조는 미실행이다. 없는 PDF/PNG나
시각 통과 수치를 기록하지 않는다. 병합 comment는 아직 계획·게시하지 않는다.
보완 후보의 저장 계약과 최신 CI를 재검증한 뒤, 필요한 원본·한컴 출력 증적을 재사용하거나
보완한다. 시각 대조를 수행하면 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.

## 최종 판정

### 승인된 S1~S2 보정 실행 (2026-09-11)

- 주 작업공간 브랜치: `review/planet6897-pr6999`.
- devel 통합: `9c94c5fe6`; 원 기여 두 commit과 devel 양쪽 이력을 보존했다.
- 보정 code head: `2b479c8b31735278afe2452c6740f26fba25eee9`.
- 구현: `render_runs`가 부모의 `para_page_num_pos_emitted`를 저장하고 false로 시작한 뒤,
  기존 본체를 `render_runs_in_paragraph_scope`로 호출하고 부모 상태를 복원한다.
  빈 문단 등 본체의 조기 반환도 같은 복원 경로를 거친다. 종류별 분기나 parser/layout 변경 없음.
- 신규 원본: `tests/cases/issue_6869_page_num_pos_nested_scope.rs`.
  SHA-256 `d9acd1002523b23f95d48581d33ef5e5bc52c18d7f70ee5c8cc6d1f0ff79cf1a`.
- `run-rust-test.mjs issue_6869_page_num_pos_nested_scope`에 기존과 같은 locked
  release-test/shared target을 사용했다. 7/7 PASS, 185 filtered skips, build 2분 11초,
  test 0.010초, nextest run `6c09eaf1-cb8f-491a-869d-e140e4547bcd`다.
  각 test에서 HWP5/HWPX 두 출처를 검사한다. 원 probe의 두 실패 경로, sibling, 빈 자식,
  2단계 중첩, 머리말, 부모/자식 textpos 소유권을 확인했다.
- `cargo fmt --all -- --check` 통과. `node --test scripts/tests/rust-test-suite-manifest.test.mjs`
  23/23 통과(3.024초), 실제 PR 범위 `git diff --check upstream/devel...HEAD` 통과.
- 같은 보정 head에서 인접 focused도 모두 통과했다. #6869 기존 2/2(201 filtered skips,
  build 5.08초/test 0.007초), #5943 축 보정 4/4(187 skips, 5.53초/0.008초),
  #6956 형광펜 왕복 3/3(186 skips, 5.39초/0.014초)이다. 신규 7건과 합해 **16/16 PASS**.
  `node scripts/rust-test-suite-manifest.mjs --check`는 1254 sources, 48/48 targets로 통과했다.
- devel 유입 파일의 기존 CRLF 경고는 PR 고유 diff에 포함되지 않았고 임의 정정하지 않았다.
- 원 contributor head의 과거 판정과 보정 후보를 구분한다. 아래 보류 판정은 아직 전체 제출
  게이트·이슈 종료 범위 검토가 남았기 때문에 유지하며 F1의 재현 두 건은 보정 후보에서 해소됐다.
  원격 push/comment/merge는 하지 않았다.

### S3 제출 검증 완료 (2026-09-11)

- code head `2b479c8b3`: fmt·native Clippy·WASM32 lib Clippy·workspace build·workspace
  all-target Clippy·manifest 모두 PASS. 검증은 review worktree와 기존 고정 공유 target에서 순차 수행했다.
- 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir
  /home/edward/mygithub/rhwp-shared-review-target --tests --no-fail-fast`: **9459/9459 PASS,
  46 skipped, 5 slow, exit 0**. 빌드 3분 41초, 테스트 390.014초다.
  run `bb15a220-0648-442e-beac-ae676c64d189`, 로그 `output/pr6999-s3/nextest.log`.
  마지막 `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup`도 166.963초에 통과했다.
  기존 nextest 권고 버전·미지원 JUnit 설정 경고는 유지됐고 테스트 실패가 아니다.
- 최종 문서 head `cd1a11dc06eff3f867c921c2a509b078cbf21871`는 작업 문서 1개만 정정했다.
  code head와 source·test·Cargo·CI가 동일하고, 이 head에서도 전체 lint 묶음·manifest를
  다시 통과했다. 문서-only 증분에는 전체 Cargo 회귀를 중복 실행하지 않았다.
- 실제 devel Git ref는 fetch와 ls-remote 모두 `6806950b1`로 기준과 같다. merge-tree exit 0,
  실제 PR diff check PASS. 원격 contributor head는 여전히 `d563b48d7`이며 push하지 않았다.
- F3 문서 불일치(브랜치명·각주 집계)는 정정했다. HWPX 형광펜 왕복 #6956과 HWP5 파싱
  #7000도 별개로 명시했다. #7000 해결이나 추가 한컴 검증을 수행했다고 쓰지 않았다.
- #6871 다섯째 원본의 첫 문단 `pageNum` 4→1, 마지막 textpos 56→32,
  section 텍스트 968자·문단 37개·표 4개·그림 5개 보존을 로컬 x2x에서 확인했다.
  원본과 산출의 순서대로 이은 텍스트 SHA-256은 모두
  `c06083b9106772de2fa5fe1f22b6e4a3508dcfe440bb417332975c5bfd75c1b3`다.
  한컴에서 본문 누락이 복구되는지는 아직 미검증이므로 #6871 전체 종료는 보류한다.

**승인** — 보정된 현재 head `cd1a11dc06eff3f867c921c2a509b078cbf21871`의 판정이다.
원 기여자 head `d563b48d7`만으로 수용한다는 뜻은 아니다. 아래 S5 self-review를 최종 근거로 삼는다.

2026-09-11 메인테이너가 기여자 재작업 요청 대신 우리 쪽 보정 후 병합 경로를 선택했다.
[보정 수행계획](pr_6999_review_impl.md)에 따라 로컬 코드 보정·제출 검증을 완료했다.
F1과 충돌은 해소됐고 S4 push를 완료했다. 새 head CI와 #6871 자동 종료 범위 정정은
아래 S5에서 확인했다. 실제 병합은 메인테이너의 별도 승인을 받는다.

### S4 push·본문 현행화 (2026-09-11)

승인 후 기존 contributor branch에 `cd1a11dc06eff3f867c921c2a509b078cbf21871`를 정상 push했다.
원 contributor 이력은 유지했고 원격 ref·PR head 일치를 확인했다. LFS 사전 판독은 대상 없음,
dry-run 및 실제 push 모두 성공이다. 원 PR 고유 diff는 6개 파일이며 devel 유입분을 보정 규모에 합산하지 않는다.

PR 본문에 보정·검증을 추가하고 #6871 자동 종료를 제거했다. 원본 기여자 실측과 메인테이너
로컬 구조 검증을 구분했다. API 재조회로 본문 일치·한글 보존·BOM 없음과 closing 문구 제거를 확인했다.

현재 새 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34564676991),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34564676946), Adapter, Proptest는 실행 중이다.
Controller는 성공했고 CI의 lint·archive A/B/C/D가 실제 실행되고 있어 review-only fast-pass가 아니다.
무관한 WASM package·Native Skia·Frontend·promotion job의 skip을 필수 검증 통과로 세지 않는다.

남은 조건은 새 head CI 성공·self-review 및 병합 승인이다. #6871 전체 종료는 여전히 보류한다.
별도 comment, GitHub approve, merge, issue close는 수행하지 않았다.

### S5 self-review 완료 (2026-09-11 14:35 KST 기준)

- 승인된 self-review 대상은 `cd1a11dc06eff3f867c921c2a509b078cbf21871`이다.
  main worktree·review worktree·fetch한 PR head가 일치하고 devel은 `6806950b1`로 유지됐다.
  merge-tree exit 0, diff check PASS, GitHub OPEN / MERGEABLE / CLEAN을 확인했다.
- PR 고유 6개 파일과 원 contributor 변경·메인테이너 보정을 다시 읽었다.
  부모 상태 저장→자식 false 진입→부모 복원은 정상/조기 반환을 같은 경계로 감싼다.
  자식의 슬롯은 그 문단 로컬 vector에만 기록되며 `secd`/`cold`와 실제 접은 중복 슬롯의
  출처별 차감 계약은 분리돼 있다. #5943 기대값 변경은 contributor의 중복 축소 계약에 따른
  변경이며, 메인테이너가 실패를 숨기려고 baseline을 완화한 변경은 없다.
- 신규 7건은 HWP5/HWPX 두 출처에서 부모/자식·sibling·빈 문단·2단 중첩·머리말·축 소유권을
  검사한다. source 본문 테스트·파생 suite/manifest·CI 변경은 PR 고유 diff에 없다.
  serializer 상태·저장 축 검토에서 추가 차단 결함은 발견하지 못했다.
- 정확한 현재 head의 CI `34564676991`에서 lint, archive A/B/C/D 빌드 및 네 테스트 worker,
  `Build & Test`가 모두 success다. Adapter `34564676945`, Proptest `34564676966`,
  CodeQL workflow `34564676946`도 success다. `CI Impact Policy` commit status는 success이며
  게시 근거는 [run 34565757351](https://github.com/edwardkim/rhwp/actions/runs/34565757351)다.
- CodeQL은 결과를 구분한다. Rust의 `Perform CodeQL Analysis`는 실제 실행·success다.
  JS/TS와 Python은 `Skip unselected language`가 실행되고 분석 step은 skipped다.
  GHAS 종합 check `103157381384`는 **neutral**, 제목 `2 configurations not found`,
  annotations 0이다. devel에 있는 JS/TS·Python 설정의 비교 자료가 이번 PR 분석에 없다는
  경고이며 전체 언어 보안 검증 또는 경고 0건으로 확대하지 않는다.
  저장소의 현재 필수 check는 `Build & Test`이며 성공을 확인했다. 보안 분류·정책은 변경하지 않았다.
- 동일 source의 로컬 9459/9459 및 신규/인접 focused 16/16 근거를 유지했다.
  code/test 변경이 없고 exact head Full CI가 성공했으므로 workflow 3.2.2에 따라 전체 Cargo
  회귀를 중복 실행하지 않았다. 이 PR은 serializer 구조 보존 범위로 검토하며,
  visual_fixture_evidence 3.5에 따라 renderer 시각 개선을 직접 검증했다는 주장을 하지 않는다.
- #6871의 다섯째 사례는 로컬 구조 보존까지만 확인한 상태를 유지한다. PR의 자동 종료 제거로
  전체 이슈가 미확인 상태에서 닫히지 않도록 했다. #6869의 한컴 실측은 contributor 보고다.
- 결과: **보정된 head 승인**. 검토 문서 반영과 최종 head 재확인 후 merge commit 병합을
  요청할 수 있다. 이번 self-review는 로컬 문서 판정이며 GitHub approve·comment·추가 push·
  merge·issue close는 수행하지 않았다. 이 기록 자체를 원격 승인으로 해석하지 않는다.
