---
kind: pr_review
status: merged
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
---

# PR #6567 검토 — #6544 회귀 핀의 오라클 판 표기 정정

## 최종 판정

**승인.** contributor head `b5abb6f629c40db60d66e20d451cb843eab90279`은
`tests/issue_1139_inline_picture_duplicate.rs`에서 #6544 당시 기여자가 실제로 사용한
한글 판을 2022에서 2024로 바로잡는다. 7개 변경은 주석과 assert 실패 메시지에만 있으며,
수치·조건·fixture·renderer 동작은 바뀌지 않는다.

이 판정은 “rhwp의 프로젝트 표준 오라클은 항상 한글 2024”라는 PR 본문의 일반화를
채택하지 않는다. 프로젝트 정본은 원본의 마지막 저장 제품에 따라 engine을 선택한다.
2022 이하 저장본은 engine 2020, 2024 저장본만 engine 2024다. 이번 source 변경의
`한글 2024`는 #6544 기여자 측 **재측정 환경의 provenance**를 뜻하며, #6554에서
메인터너가 사용한 engine 2020 기준 PDF를 대체하거나 무효화하지 않는다.

## 라우팅

- 기본 경로: `maintainer_general.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `post_merge.md`
- 작성자는 기존 기여자이므로 `first_time_contributor.md`는 적용하지 않았다.
- 단일 commit·단일 파일·100줄 미만이며 메인터너 보정 계획이 없어 별도
  `pr_6567_review_impl.md`는 만들지 않았다.

## 메타데이터와 검토 대상

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6567](https://github.com/edwardkim/rhwp/pull/6567) / @planet6897 |
| 관련 이슈 | [#6544](https://github.com/edwardkim/rhwp/issues/6544); 이 PR은 이슈를 close하지 않음 |
| base / draft | `devel` / 아님 |
| contributor head | `b5abb6f629c40db60d66e20d451cb843eab90279` |
| 검토 기준 `devel` | `8b55fcf064fd639f25e1d139845b10cf75cd292b` |
| current-base review candidate | `599afb4dcaa129e30b50968bb893562d27c31609` |
| current-base merge tree | `72468df35fe59d35ea7c2ce591170ea839afa01d` |
| 규모 | 1 commit, 1 file, `+7/-7` |
| 작성 시점 상태 | open, non-draft, `MERGEABLE/CLEAN`; merge 전 재확인 필요 |
| reviewer / metadata | `edwardkim`; assignee `planet6897`, milestone `v1.0.0`, labels `documentation`, `rust`, `test` |

GitHub의 기존 `pull/6567/merge` ref `2b37caa355e6bacecc5c19b5e33a8f6ff71e25cd`는
과거 base `b33752594e5adaa85adc78481f171f81272aeb58`로 만든 stale merge였다. 최신
`devel`은 contributor head의 merge base `8b17a07737e6b72910a0b7c422cdf22ac628d759`보다
29 commits 전진했지만 그 사이 대상 test 파일을 바꾼 commit은 없다. 최신 base로 직접 계산한
merge-tree는 충돌 없이 생성됐고, contributor commit을 저자 보존 cherry-pick한 review candidate의
tree와 정확히 일치했다.

## 변경 범위 검토

변경된 7곳은 다음 세 종류다.

- `[#6544] 한글 2022 오라클 재측정` 주석 3곳을 `한글 2024`로 정정
- p20 기준 PDF를 설명하는 주석 1곳을 `한글 2024 PDF`로 정정
- 세 회귀의 assert 실패 메시지 3곳을 `한글 2024`로 정정

함수명에 포함된 `2022`는 시험 문서의 연도이므로 바꾸지 않았다. assert predicate,
좌표 범위, 페이지·단 위치, expected 값과 sample 경로도 바뀌지 않았다. 따라서 PR 제목의
`문서`는 변경 의도를 설명하지만 CI 영향 분류상 이 PR은 Rust test source 변경이다.

## 오라클 provenance 교차 확인

기여자는 다음 세 곳에 같은 정정을 남겼다.

- [#6544 정정 comment](https://github.com/edwardkim/rhwp/issues/6544#issuecomment-5491603802)
- [#6550 정정 comment](https://github.com/edwardkim/rhwp/issues/6550#issuecomment-5491604066)
- [#3386 정정 comment](https://github.com/edwardkim/rhwp/issues/3386#issuecomment-5491604323)

세 기록은 한글 2022 `12.0.0.1`이 설치돼 있지만 COM 미등록이고,
`HWPFrame.HwpObject`가 한글 2024 `13.0.0.223`에 연결된 환경에서 수치를 측정했다고
일관되게 설명한다. 따라서 source에 처음 적힌 `한글 2022`는 실제 측정 프로그램과 맞지 않았다.

한편 #6554 메인터너 검토의 원본 `samples/3-09월_교육_통합_2023.hwp`는 마지막 저장 제품이
`hancom-office-2022`였고, canonical 정책에 따라
`pdf/3-09월_교육_통합_2023-hwp-2020.pdf`를 기준으로 사용했다. 메인터너가 engine 2020
p13·p20을 직접 확인한 결과도 목표 문단과 수식이 같은 단으로 이동해야 한다는 좁은 방향을
지지했다. 즉 기여자 측 2024 재측정과 프로젝트 engine 2020 검증은 역할이 다르며, 이번 PR은
앞쪽 provenance의 잘못된 이름만 고친다.

## 검증 결과

### current-base simulation과 focused test

- 최신 `devel`과 contributor head의 `git merge-tree --write-tree`가 충돌 없이
  `72468df35fe59d35ea7c2ce591170ea839afa01d`를 생성했다.
- contributor commit을 최신 `devel`에 저자 보존 cherry-pick한 candidate
  `599afb4dcaa129e30b50968bb893562d27c31609`의 tree가 위 merge tree와 일치했다.
- `git diff --check`가 통과했고 유효 diff는 한 파일의 7개 표기 교체뿐이었다.
- `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`가 1,106 sources,
  4,771 static test attrs, 48/48 integration targets를 확인했다. 파생 변경은 남지 않았다.
- 다음 명령으로 해당 source module을 실행했다.

```text
node scripts/run-rust-test.mjs issue_1139_inline_picture_duplicate -- \
  --cargo-profile release-test --target-dir target/pr-review
```

선택된 86건이 모두 통과했으며 변경된 세 회귀도 포함됐다. compile은 2분 8초,
test 실행은 1.967초였다. 73건은 module filter 밖의 test로 skip됐다. 첫 시도에서 함수명을
manifest case 이름으로 전달해 Cargo 실행 전에 선택기 오류로 종료됐고, source case 이름으로
바로잡은 위 실행만 검증 결과로 사용했다.

로컬 nextest는 저장소 권장 0.9.140보다 낮은 0.9.137이어서
`junit.report-skipped` 미지원 경고를 냈다. 실행 자체는 종료 코드 0이었고, 정확한 contributor
head의 GitHub CI가 권장 runner 환경의 광범위 검증을 보완한다.

### GitHub Full CI 재사용

정확한 contributor head `b5abb6f6`에서 CI run `33490340463`, CodeQL run
`33490340425`, Proptest run `33490340633`, Adapter run `33490340817`이 완료됐다.
최종 집계는 success 24, 정책상 skip 5, neutral 1, failure·pending 0이다. CI의
`Lint (fmt, clippy, WASM check)`, 네 archive build·test와 `Build & Test` aggregate도
성공했다.

메인터너는 source·test·fixture·workflow 보정을 추가하지 않았고 current-base tree도
충돌 없이 동일 diff를 유지했다. 따라서 `local_validation.md` 4.3.0에 따라 로컬 Rust lint
묶음과 release-test 전체 회귀는 중복 실행하지 않고 exact-head Full CI와 current-base focused
test를 근거로 재사용했다.

## 시각 검증 판정

fresh visual sweep은 수행하지 않았다. 이 PR은 renderer, expected 좌표, fixture, 기준 PDF를
바꾸지 않고 실패 메시지와 provenance 주석만 정정한다. #6554에서 이미 보존한 engine 2020
기준 PDF·review asset과 기여자 측 2024 측정 comment를 교차 확인했으며, 이 확인을 새로운
pixel fidelity 또는 한글 2024 기준 PDF의 독립 재현으로 승격하지 않는다.

## 잔여 위험과 merge 조건

- PR 본문의 “프로젝트 표준 오라클이 한글 2024”는 canonical 정책이 아니다. merge 후 contributor
  comment에서 이번 정정이 기여자 측 측정 provenance이며 저장 버전별 engine 선택은 그대로임을
  명시한다.
- #6544에는 `pi=659` 잔여 원인이 있으므로 이 PR로 close하지 않는다.
- GitHub merge ref가 stale였으므로 merge 직전에 최신 `devel`, exact head, current-base
  merge-tree, required checks와 `MERGEABLE/CLEAN`을 다시 확인한다.
- 이 문서는 GitHub approve·comment·push·merge 승인이 아니다. 실제 원격 조치는 각각
  작업지시자의 별도 승인을 받는다.

## Merge 후 contributor PR comment 계획

- 실제 merge commit과 exact contributor head를 링크한다.
- source 표기 정정은 타당하지만 “프로젝트 표준 오라클이 항상 2024”는 채택하지 않으며,
  2022 이하 저장본은 engine 2020, 2024 저장본만 engine 2024라는 canonical 경계를 알린다.
- focused 86건과 Full CI 결과, 수치·조건·동작 불변을 기록한다.
- #6544는 잔여 원인 때문에 계속 open임을 명시한다.
- 새 visual asset은 만들거나 게시하지 않는다. 게시 승인을 받은 뒤 UTF-8 without BOM body file을
  사용하고 API로 한글·BOM·`??` 치환을 재확인한다.

## 원격 조치 상태

승인된 Stage 1에서 reviewer, assignee, milestone과 labels를 적용했다. exact contributor
head `b5abb6f629c40db60d66e20d451cb843eab90279`에 `APPROVED` review를 게시한 뒤,
merge 직전 최신 `upstream/devel@8b55fcf064fd639f25e1d139845b10cf75cd292b`과의
merge-tree `72468df35fe59d35ea7c2ce591170ea839afa01d`, `MERGEABLE/CLEAN`, success 24,
skip 5, neutral 1, failure·pending 0을 재확인했다.

PR은 2026-09-01에 정상 2-parent merge commit
`86d5a361e5e7a5f349717c50d295d56d2c50e733`으로 `devel`에 병합됐다. merge 직후
`upstream/devel`이 이 commit을 포함함을 확인했고, #6544는 `pi=659` 잔여 원인 때문에
계속 open이다. source push와 issue close는 수행하지 않았다. contributor 후속 comment는
archive 운영 기록이 `devel`에 반영된 뒤 별도 승인 범위에서 게시한다.
