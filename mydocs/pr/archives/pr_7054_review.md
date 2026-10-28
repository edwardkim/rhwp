---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-12
---

# PR #7054 — 한글 Rust 테스트 duration 증거 재사용 보정 검토

**최종 판정: 승인.** Unicode 테스트 이름의 발행·소비 계약 불일치를 수정했고, 로컬 계약 검증과
정확한 head의 CI를 확인한 뒤 병합했다. 배포 후 [#7052](pr_7052_review.md)의 실제 fork PR에서
CI 재사용·무거운 작업 skip·duration 데이터 갱신까지 확인했다.

## 1. 대상과 변경 범위

| 항목 | 확인 내용 |
| --- | --- |
| PR | [#7054](https://github.com/edwardkim/rhwp/pull/7054) — fix(ci): 한글 Rust 테스트 duration 증거 재사용 보정 (#6901) |
| 작성자 / 검토자 | `jangster77` / `jangster77` self-review |
| 경로 | collaborator self-merge, 접수·리뷰, 로컬 검증, post-merge |
| base / head branch | `devel` / `fix/6901-unicode-duration-20260912` |
| 검증·병합 head | `00537652976f119a259e764d2972475279139a1c` |
| 규모 | 3개 파일, +74 / -1 |
| 병합 | `38f1133803a85f6dc68bd62476e83fc75321a6b4`, 2026-09-12 03:54:57 UTC / 12:54:57 KST |
| 병합 직전 참고값 | 최신 head 일치, `MERGEABLE / CLEAN`, 실패·대기 check 없음 |

변경은 [duration 소비기](../../../scripts/trusted-postmerge-duration-evidence.mjs),
[회귀 테스트](../../../scripts/tests/trusted-postmerge-duration-evidence.test.mjs),
[단계 1 기록](../../working/task_m100_6901_unicode_duration_stage1.md)이다.
Rust 제품 코드·렌더링·fixture·workflow 권한을 변경하지 않았다.

## 2. 문제와 코드 검토

[#7044의 post-merge CI](https://github.com/edwardkim/rhwp/actions/runs/34669914465/job/103489207024)가
`candidate-duration-artifact-check-error`로 Full 실행에 전환했다. JUnit 발행기는 한글 Rust
테스트명을 보존했지만, 소비기의 이름 정규식은 ASCII만 허용했다. 실제 B/C/D 자료에서
한글 이름 13개가 거부됐으며, 중복 항목이나 잘못된 duration이 원인은 아니었다.

수정은 이름을 Unicode XID 식별자의 `::` 경로로 검증한다. 각 segment의 시작·계속 문자를
구분하며 default-ignorable 문자를 거부한다. 빈 segment, 숫자·결합문자 시작, 경로·공백·NUL,
bidi·ZWJ·variation selector를 허용하지 않는 회귀 계약을 확인했다.

저장소·PR·head·attempt·tested merge SHA의 출처 검증, ZIP 항목·크기·링크·중복 JSON 키,
duration 수치와 target 소유 검사는 유지된다. 발행기는 이미 Unicode 이름을 보존하므로 수정하지 않았다.
이름을 실행할 코드가 아닌 자료로 처리하며, CI 권한과 재사용 gate를 완화하지 않았다.

## 3. 완료한 검증

| 검증 | 실제 결과 |
| --- | --- |
| JavaScript post-merge/duration 계약 | 520 PASS, 0 FAIL |
| Python workflow 계약 | 17 PASS |
| 한글 JUnit → ZIP → 소비 → policy 갱신 | PASS |
| 음성 대조 | 수정 전 소비기는 한글 자료를 거부하고 수정 후 소비기는 승인 |
| 실제 #7044 CI 자료 | B 1,973 / C 1,555 / D 1,909, 합계 5,437 test case 승인; Unicode 13개 보존 |
| 변경 무결성 | `git diff --check` 및 자동 병합 tree 검증 통과 |

단계 1의 집중 JavaScript 89 PASS는 전체 520개와 중복되므로 합산하지 않는다.
병합 검토에서 아래 전체 JavaScript 계약과 Python workflow 계약을 확인했다.

```bash
node --test --test-reporter=tap \
  scripts/tests/*postmerge*.test.mjs \
  scripts/tests/nextest-target-duration-policy.test.mjs
python3 -m unittest discover -s scripts/tests \
  -p 'test_trusted_postmerge_ci_reuse_workflow.py'
```

실제 입력은 [#7044 PR CI 34662415596](https://github.com/edwardkim/rhwp/actions/runs/34662415596)의
B/C/D ZIP이며 자세한 재현은 단계 1 기록에 있다. 로컬 보조 로그는
`/tmp/rhwp-pr7054-node-contracts-20260912.log`에 남겼다. Rust·브라우저·렌더러 변경이 없어서
로컬 Rust 전체 회귀·WASM 빌드·PDF/raster visual sweep은 반복하지 않았다.

| GitHub 검증 | 결과 / 실행 |
| --- | --- |
| 정확한 PR head CI | SUCCESS — [34670400088](https://github.com/edwardkim/rhwp/actions/runs/34670400088) |
| 정확한 PR head CodeQL | SUCCESS — [34670400117](https://github.com/edwardkim/rhwp/actions/runs/34670400117) |
| 병합 후 devel CI | SUCCESS — [34671632246](https://github.com/edwardkim/rhwp/actions/runs/34671632246) |
| 병합 후 CodeQL | SUCCESS — [34671632225](https://github.com/edwardkim/rhwp/actions/runs/34671632225) |

#7054 자체는 CI enforcement surface를 변경하므로 post-merge verifier가 다음 사유로 Full CI를 선택했다.
`Build & Test`와 `Refresh nextest target duration data`의 성공을 확인했지만, 이 실행을 PR 증거
재사용 성공으로 세지 않았다.

```text
reuse=false reason=pr-changes-ci-enforcement-surface source_run_id=none refresh_duration_data=false
```

## 4. 배포 후 실제 fork PR 실증

CI 정책을 변경하지 않는 HWP3 스타일 PR #7052를 실증 대상으로 선정했다. 최신 base 정렬만으로는
원 코드 head의 기존 CI를 재사용한 fast-pass가 선택됐다. 따라서 승인된 빈 커밋으로 새
`pull_request` Full CI를 실행하고, 같은 tree를 merge한 뒤 post-merge 결과를 확인했다.
자세한 SHA·artifact ID·A/B 검증은 [#7052 검토 기록](pr_7052_review.md)에 보존했다.

- [새 Full CI 34673335479](https://github.com/edwardkim/rhwp/actions/runs/34673335479):
  head `e8e4b041c279bdea118f25a65c9be3abc281b388`, base `38f1133803a85f6dc68bd62476e83fc75321a6b4`,
  archive A/B/C/D·Native Skia·lint·Build & Test 성공.
- [Post-merge CI 34674078926](https://github.com/edwardkim/rhwp/actions/runs/34674078926):
  `reuse=true`, `source_run_id=34673335479`, `refresh_duration_data=true`.
  무거운 작업은 재사용으로 skip, Build & Test와 duration 갱신은 SUCCESS.
- 실제 새 B/C/D 자료의 5,442개 test case 중 비ASCII 이름 13개를 승인했다.
  [Duration 데이터 commit 4b2d6e8b](https://github.com/edwardkim/rhwp/commit/4b2d6e8b612dd6c5a7940274d533deea7568905a)의
  세 `measurement_sources`가 모두 해당 PR run·head·tested merge를 가리키는 것까지 확인했다.
- CodeQL·Adapter·Proptest에서도 각각의 새 PR Full 실행을 실제 재사용했다.

이는 합성 fixture나 로컬 validator 호출에 한정된 결과가 아니다. 실제 upstream Actions가
fork PR 증거를 소비해 devel의 중복 작업을 생략하고 duration 데이터 branch를 갱신한 성공 사례다.
다만 [#6901](https://github.com/edwardkim/rhwp/issues/6901)의 모든 경계 사례를 추가 검증하거나
이슈 전체 종료를 판정한 것은 아니며, 문서 작성 시 이슈는 OPEN이다.

## 5. 후속 상태와 comment 계획

사용자 승인 후 `--merge --match-head-commit`으로 병합했고 로컬 `devel`을 동기화했다.
소유한 #7054 검토용 worktree와 review branch를 정리했으며, 사용자 작업공간·다른 브랜치·공유
산출물은 보존했다. 새 Cargo target은 만들지 않았다. 기존 `fix/6901-unicode-duration-20260912`
브랜치는 이번 검토 이전에 생성된 대상이므로 정리하지 않았다.

사용자가 미뤘던 review·오늘할일을 이번 후속 문서 PR에 보존한다. 다음은 게시 계획이며
GitHub comment/review를 이미 게시했다는 뜻이 아니다.

1. 문서 반영 및 게시 승인 뒤 기존 댓글과 중복되는지 확인한다.
2. #7054에 merge `38f113380`, 로컬 520+17 계약 결과, 정확한 head CI, 자체 post-merge Full 성공을 기록한다.
3. #6901에는 #7052의 실제 `reuse=true`·heavy skip·duration 갱신과 원격 증거 링크를 연결한다.
   #7054 자체의 Full 실행과 이후 fork 실증을 구분하고, 이슈를 자동 close하지 않는다.
4. UTF-8 파일과 `--body-file`로 게시하고 API에서 본문·링크 일치를 확인한다.

[오늘할일](../../orders/20260912.md) · [#7052 검토 기록](pr_7052_review.md)
