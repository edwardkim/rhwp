---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-12
---

# PR #7052 — HWP3 스타일 풀 중복 제거와 post-merge CI 재사용 검토

**최종 판정: 승인.** 검토 범위의 신규 차단 결함을 발견하지 못했고, 검증한 최종 head를
2026-09-12 13:50:56 KST에 병합했다. 실제 fork PR의 Full CI 증거를 post-merge CI가
재사용하고 B/C/D duration 데이터를 갱신한 것까지 확인했다.

## 1. 대상과 변경 범위

| 항목 | 확인 내용 |
| --- | --- |
| PR | [#7052](https://github.com/edwardkim/rhwp/pull/7052) — 수정: HWP3 스타일을 문단·런·셀마다 새로 등록하지 않는다 (#4680) |
| 작성자 / 검토자 | `planet6897` / `jangster77` |
| 경로 | collaborator 매개 외부 PR, 접수·리뷰, 로컬 검증, post-merge |
| 원격 head branch | `planet6897/rhwp:fix/4680-hwp3-style-pool-dedup` |
| 원 코드 head | `ed2dcce05ce9d4cb1428731e00f65397bd74a77f` |
| devel 정렬 head | `32f0368a22356701bc5a872c9d4495461a1ecbec` |
| 최종 검증·병합 head | `e8e4b041c279bdea118f25a65c9be3abc281b388` |
| 검증 base | `38f1133803a85f6dc68bd62476e83fc75321a6b4` (`devel`) |
| 규모 | 7개 파일, +290 / -25; 마지막 커밋은 빈 커밋 |
| 병합 | `a192396cb46cff738cf6e37c3cb9463d074ff197`, 2026-09-12 04:50:56 UTC / 13:50:56 KST |
| 병합 직전 참고값 | non-Draft, `MERGEABLE / CLEAN`; check 30 SUCCESS, 6 SKIPPED, 1 NEUTRAL |

HWP3의 문단·런·표 셀마다 같은 글자 모양, 문단 모양, 테두리/배경을 새로 등록하던 경로를
값이 같은 기존 항목의 ID를 재사용하도록 바꿨다. 테두리를 먼저 등록해 문단 모양의 테두리 ID도
정규화하고, 저장하는 테두리 참조의 1-based 규칙을 유지한다.

변경은 [HWP3 파서](../../../src/parser/hwp3/mod.rs), [스타일 모델](../../../src/model/style.rs),
중복 제거 회귀 테스트와 기존 변환·진단 계약 테스트 5개 파일이다. CI 정책·workflow·fixture는
변경하지 않았다. [#4680](https://github.com/edwardkim/rhwp/issues/4680)의 한컴 열기 거부,
`SHAPE_COMPONENT` 페이로드 문제 전체를 해결하는 PR은 아니다.

## 2. 코드 검토와 검증 범위

- `CharShape`·`ParaShape`의 동등성 비교와 `BorderFill` 계열 7개 타입의 `PartialEq` 추가를 확인했다.
  동등성에서 제외하는 `raw_data`, `hwpx_plain_para_margin`은 HWP3 변환 경로에서 기본값이며
  회귀 테스트도 이를 검사한다.
- ID 공유가 등록된 스타일의 변경으로 다른 문단에 영향을 줄 수 있는지 확인했다. 검토한 HWP3
  경로는 풀을 읽거나 추가하며, 문서 편집의 관련 경로는 스타일을 복제해 새 항목을 등록한다.
- 문단·런·셀의 ID 유효성, 테두리 1-based 참조, 중복 제거 후 진단 출력 차이를 확인했다.
  `q5` 기대값은 풀 개수와 참조 ID 변경을 반영하며, 변환의 음영·상대크기·장평 계약은 유지된다.
- 기존 테스트의 `CHAR_SHAPE > 1000` 하한은 중복 포함 개수에 의존했다. SO-SUEOP에서
  2,512개가 고유 63개로 줄었고, 실질 속성 검사는 유지되는 것을 확인했다.

중복 제거 후 ID와 값의 가짓수 비교만으로 모든 과도한 병합을 검출했다고 단정하지 않았다.
독립 A/B에서 실제 참조를 스타일 값으로 펼쳐 문단·스타일 의미를 비교했다. 이번 PR은 배치 개선이나
새 시각 기준 자료를 주장하지 않으므로 PDF/raster visual sweep을 실행하지 않았다. 실제 한컴오피스의
열기 성공도 검증하지 않았으며, 원 이슈 해결이나 시각 통과로 승격하지 않는다.

## 3. 완료한 로컬 검증

검토 전용 worktree와 `CARGO_TARGET_DIR`을 사용했다. 원 코드에 보정을 추가하지 않았고,
기존 Full CI를 광범위 회귀 증거로 재사용하면서 아래 집중 검증을 수행했다.

| 검증 | 실제 결과 |
| --- | --- |
| 집중 nextest | 25 PASS, 0 FAIL, 929 filter-skipped; 최종 candidate 복원 후 재실행도 exit 0 |
| 기존 파서 음성 대조 | 새 테스트 5개 중 중복 제거 검사 3개 예상 FAIL, 나머지 2개 PASS; exit 100 |
| 실제 HWP3 fixture 26개 파싱 A/B | 스타일 ID를 실제 값으로 펼친 문단·스타일 의미 26/26 일치 |
| HWP5 저장·재파싱 | 양쪽 26/26 성공, 저장 후 의미 비교 25/26 일치 |
| 차이 1건 반복 확인 | `hwp3-sample10.hwp` 외부 이미지 경로 순서가 기존 파서와 PR 파서 각각의 반복 실행에서도 달라짐 |
| 최종 소스 상태 | 음성 대조용 교체를 원 head 내용으로 복원, `git diff --check` 통과 |

집중 테스트 실행 명령은 다음과 같다. 필터로 제외된 테스트를 통과 수에 더하지 않았다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /Users/tsjang/rhwp/target/review-pr7052-20260912 \
  --test regression_suite_008 --test regression_suite_009 \
  --test regression_suite_003 --test regression_suite_019 --test regression_suite_025 \
  -E 'test(issue_4680_hwp3_style_pool_dedup::) | test(issue_4161_ratio_default_contract::) | test(q5_diagnostic_output_contract::) | test(issue_4141_hwp3_relative_size_contract::) | test(issue_4155_hwp3_char_shade_contract::)' \
  --no-fail-fast
```

음성 대조는 `src/parser/hwp3/mod.rs`만 검증 base의 내용으로 교체한 격리 환경에서 실행했다.
추가된 `PartialEq`와 candidate 테스트는 유지하고, 실행 뒤 candidate 소스를 바이트 단위로 복원했다.
A/B는 파싱 → 참조 스타일 값 확장 → HWP5 저장 → 재파싱을 양쪽 바이너리로 수행했다.
공개 작업공간의 실제 HWP3 26개를 검증했으며, 기여자가 보고한 비공개 38개 corpus를 검증한 것은 아니다.

이미지 경로 차이는 기존 `pic_name_to_id` HashMap 순회와 Link 목록 순서에 연결된다. 이 PR에서
수정하지 않은 기존 경로이며, 26개 저장 후 결과가 전부 같다고 기록하지 않는다.
임시 진단 소스·생성 suite·로그는 제품 또는 테스트 변경으로 커밋하지 않았다.

## 4. 새 Full CI 증거를 만든 이유와 결과

정렬 head `32f0368a2`의 [CI 34671910281](https://github.com/edwardkim/rhwp/actions/runs/34671910281)는
원 코드 head의 [Full CI 34670133776](https://github.com/edwardkim/rhwp/actions/runs/34670133776)를
재사용한 PR fast-pass였다. 그 Full CI의 tree에는 [#7054의 검증기 수정](pr_7054_review.md)이
없어서 실제 post-merge 재사용의 같은 tree 증거로 사용할 수 없었다.

사용자가 승인한 빈 커밋 `e8e4b041c`를 기존 원 PR branch에 fast-forward push했다.
소스 tree는 그대로이고, PR preflight가 아래 실제 로그와 함께 Full CI를 실행했다.

```text
fast_pass=false reason=unusable-file-list:e8e4b041c279bdea118f25a65c9be3abc281b388
```

| 최종 head의 PR 검증 | 결과 / 실행 |
| --- | --- |
| CI: archive A/B/C/D 빌드·테스트, Native Skia, lint, Build & Test | SUCCESS — [34673335479](https://github.com/edwardkim/rhwp/actions/runs/34673335479) |
| CodeQL: Python·JavaScript·Rust | SUCCESS — [34673335520](https://github.com/edwardkim/rhwp/actions/runs/34673335520) |
| Render Diff | SUCCESS — [34673335389](https://github.com/edwardkim/rhwp/actions/runs/34673335389) |
| Adapter inter-diff | SUCCESS — [34673335591](https://github.com/edwardkim/rhwp/actions/runs/34673335591) |
| Proptest roundtrip | SUCCESS — [34673335482](https://github.com/edwardkim/rhwp/actions/runs/34673335482) |

Immutable merge-tree artifact `10291352686`의 schema는 2, run attempt는 1이다.
저장소 ID `1193345564`, fork ID `1215119718`, PR 번호 7052도 실제 API와 대조했다.

| 증거 | SHA |
| --- | --- |
| PR tested merge | `c900264616dff4977581952e41b99c3882cc48cb` |
| 부모 1: 최신 devel | `38f1133803a85f6dc68bd62476e83fc75321a6b4` |
| 부모 2: 최종 PR head | `e8e4b041c279bdea118f25a65c9be3abc281b388` |
| PR CI 및 실제 merge 공통 tree | `92fe2db9df17f0712af105a9f4e5dd1b15d03794` |

최신 head·필수 check·mergeability와 자동 병합 tree를 확인한 뒤 `--merge --match-head-commit`으로
병합했다. contributor commit의 rebase·amend·force push는 하지 않았다.

## 5. 실제 post-merge 재사용과 duration 갱신

[Post-merge CI 34674078926](https://github.com/edwardkim/rhwp/actions/runs/34674078926)의 실제 로그다.

```text
reuse=true reason=exact-green-pr-workflow-reused source_run_id=34673335479 refresh_duration_data=true
```

빌드·테스트·lint·Native Skia는 재사용으로 skip했고 `Build & Test`와
`Refresh nextest target duration data`는 SUCCESS였다. 갱신 작업은 trusted PR Archive B/C/D
다운로드 3단계를 실제 실행했다. CodeQL·Adapter·Proptest도 아래 PR run의 증거를 재사용했다.

| Post-merge workflow | 실행 | source PR run | 결과 |
| --- | --- | --- | --- |
| CI | [34674078926](https://github.com/edwardkim/rhwp/actions/runs/34674078926) | 34673335479 | reuse=true, refresh=true, SUCCESS |
| CodeQL | [34674078903](https://github.com/edwardkim/rhwp/actions/runs/34674078903) | 34673335520 | reuse=true, SUCCESS |
| Adapter inter-diff | [34674078912](https://github.com/edwardkim/rhwp/actions/runs/34674078912) | 34673335591 | reuse=true, SUCCESS |
| Proptest roundtrip | [34674078892](https://github.com/edwardkim/rhwp/actions/runs/34674078892) | 34673335482 | reuse=true, SUCCESS |

새 duration ZIP을 검증기에 넣어 run·PR·head·attempt·tested merge SHA, ZIP·schema·이름·수치 계약을
대조했다. B/C/D의 5,442개 test case에는 비ASCII 테스트명 13개가 포함된다.

| Archive | Artifact ID | test case | 비ASCII test case |
| --- | --- | ---: | ---: |
| B | 10291845969 | 1,280 | 7 |
| C | 10291995648 | 1,826 | 2 |
| D | 10291990665 | 2,336 | 4 |

GitHub Actions가 duration 데이터 branch에 만든
[commit 4b2d6e8b](https://github.com/edwardkim/rhwp/commit/4b2d6e8b612dd6c5a7940274d533deea7568905a)의
`measurement_sources`를 직접 확인했다. B/C/D 모두 `run_id=34673335479`,
`ref=refs/pull/7052/merge`, `sha=c900264616dff4977581952e41b99c3882cc48cb`,
`head_sha=e8e4b041c279bdea118f25a65c9be3abc281b388`, `run_attempt=1`이었다.
성공 badge만 확인한 것이 아니라 PR 자료가 실제 갱신 입력이 된 것까지 확인했다.

## 6. 후속 상태와 기록 보존

- 로컬 `devel`을 merge `a192396cb`까지 fast-forward했고 upstream과 일치했다.
- 검토용 worktree·`review/pr7052-ci-20260912`·소유한 추적 ref를 제거했다.
  실행 중인 Cargo/Rust가 없음을 확인하고 전용 target 1.1GB를 휴지통으로 이동했다.
- 기본 작업공간의 다른 브랜치와 미커밋 오늘할일, 공유 target 및 contributor fork branch는 보존했다.
- 사용자 지시로 미뤘던 review·오늘할일을 이번 후속 문서 PR에 작성했다.
  원 코드와 CI 수정은 이미 병합됐고, 이 기록 PR에 제품 변경은 없다.
- 기록 시 #4680과 [#6901](https://github.com/edwardkim/rhwp/issues/6901)은 OPEN이다.
  이번 실제 재사용 성공만으로 두 이슈 전체를 종료하지 않는다.

로컬 보조 자료는 `/tmp/rhwp-pr7052-final-focused-20260912.log`,
`/tmp/rhwp-pr7052-negative-20260912.log`, `/tmp/rhwp-pr7052-review-probe-20260912.rs`,
`/tmp/rhwp-pr7052-{baseline,candidate}-semantics-20260912.jsonl`,
`/tmp/rhwp-pr7052-full-ci-e8e4b041c/`, `/tmp/rhwp-pr7052-postmerge-a192396cb/`에 보관했다.
이 경로들은 임시 자료이며, 장기 기록의 핵심 SHA·명령·결과·원격 증거는 본문에 보존했다.

## 7. Merge 후 contributor PR comment 계획

이 절은 게시 초안 계획이며 실제 GitHub comment/review 게시를 뜻하지 않는다.
문서 PR이 devel에 반영되고 게시가 승인되면 중복 댓글 여부를 먼저 확인한다.

- 기여에 감사하고 merge `a192396cb`, 검토 head `e8e4b041c`, 25개 집중 테스트와 26개 A/B의
  실제 결과·기존 이미지 경로 차이를 설명한다. #4680의 한컴 열기 문제는 미해결로 구분한다.
- PR Full CI와 post-merge 실행 링크, 실제 `reuse=true` 및 duration 데이터 commit을 연결한다.
- #6901에는 이 fork PR 사례의 성공 증거와 검증한 경계를 기록하는 초안을 준비하며 자동 close하지 않는다.
- 게시 시 UTF-8 파일과 `--body-file`을 사용하고, 게시 뒤 API로 본문·링크 일치를 확인한다.

[오늘할일](../../orders/20260912.md) · [#7054 검토 기록](pr_7054_review.md)
