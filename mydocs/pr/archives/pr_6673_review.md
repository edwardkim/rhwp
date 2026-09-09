---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-03
pr: 6673
issue: 6641
author: edwardkim
---

# PR #6673 self-review — 필드 편집 후 소유 문단 LineSeg 재조판

## 결론

**승인.** PR #6673은 필드 값을 바꾼 뒤 LineSeg를 비우기만 하고 소유 문단 폭에서 다시 조판하지
않아 메모리 문서와 저장·재적재 문서가 달라지던 #6641의 원인을 수정한다. 검증기의 diff·종료 코드를
완화하지 않고, 본문·표 셀·글상자·깊이 2 중첩 경로를 기존 reflow와 vpos 계산기로 수렴시켰다.

소스·테스트 후보 `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`의 로컬 필수 Rust 검증과 전체
8,973건이 통과했다. 이후 보고서만 추가한 최초 PR head
`362e912a6f0da06dd3cbd902d069adba87f0ddfc`의 CI·CodeQL·Proptest·Adapter inter-diff도 모두
완료됐고, 최신 `devel`과의 merge tree가 해당 head tree와 정확히 일치했다. 이 review와 오늘 작업
기록만 추가하는 후행 head의 review-only checks와 mergeability를 다시 확인한 뒤 정상 merge할 수 있다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
review event를 만들지 않으며, remote push와 merge는 각각 별도 사용자 승인 게이트다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`, `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. contributor 보정·충돌 해소·다중 선택 경로가 없는 단일 self PR이다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6673](https://github.com/edwardkim/rhwp/pull/6673) / @edwardkim |
| 관련 이슈 | [#6641](https://github.com/edwardkim/rhwp/issues/6641) (`Closes #6641`) |
| 부모 계보 | [#6628](https://github.com/edwardkim/rhwp/issues/6628) Gym 전수에서 BO05·BO15로 발견 |
| base | `devel@900b56edcaff3c1f84567c3f7c9e398a0dd9e8bb` |
| source/test candidate | `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784` |
| 최초 PR head | `362e912a6f0da06dd3cbd902d069adba87f0ddfc` |
| 규모 | 13 files, `+1,214/-26`, 17 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`, `CLEAN`; 모든 실행 대상 check 완료 |
| reviewer | self PR이므로 지정하지 않음 |

1,000줄을 넘으므로 대형 PR 경로를 적용했다. 증가분은 작업 계보 문서 820줄, 기존 test source의
계약 강화 237줄, 제품 source 157줄로 구성된다. 새 integration source·fixture·golden·baseline·workflow는
없다. 17개 commit에는 Stage별 red/green·검증 기록과 작업 중 최신 `devel`을 보존한 merge 세 건이
포함되며, 현재 base는 PR head의 조상이고 이미 병합된 과거 source commit이 diff에 혼입되지 않았다.

## 코드 검토와 보호 불변식

### 공개 setter와 mutation 순서

- `set_field_value_by_id`는 `collect_all_fields()`가 공개한 HWPX 가상 셀 field ID를 더 이상 일반
  ClickHere 범위 치환으로 보내지 않고, by-name 경로와 같은 셀 텍스트 mutation으로 보낸다.
- setter는 mutation 전에 본문 또는 중첩 소유자 경로가 실제로 존재하는지 확인한다. 지원하지 않는
  경로나 범위 초과를 조용한 성공으로 바꾸지 않는다.
- 본문은 편집 전 흐름 끝을 보존하고 해당 문단을 reflow한 뒤 section vpos를 잇는다. 중첩 경로는
  `TableCell`과 `TextBox` 의미를 path tuple로 옮겨 기존 최내곽 owner width·padding/margin 계산기를
  재사용한다.
- reflow 뒤 LineSeg가 비면 `InvalidField`를 반환한다. 표 tree dirty, section raw passthrough 무효화와
  section compose/cache 무효화는 기존 mutation 계약을 유지한다.
- `set_field_value_by_id`가 이제 raw passthrough 무효화를 직접 소유하므로, 이를 단순 위임자로 보던
  passthrough guard 예외를 제거한 것은 구현과 일치한다.

### 회귀 계약

- 정상 `batch fill --verify`는 산출물을 남기면서 exit 0, `identical=true`, `diffCount=0`이어야 한다.
- 기존 `issue_838_field_set_value.rs` 안에서 HWP5 본문 두 필드, HWPX 가상 셀 by-id, HWP5 중첩 셀
  ClickHere, 셀 안 글상자, 깊이 2 HWPX ClickHere를 분리해 검증한다.
- HWP5는 LineSeg 수·시작점·vpos를 저장 왕복 전후 직접 비교한다. HWPX는 bit 31 합성 LineSeg 생략
  규약을 유지하면서 값·문자모양 경계·`char_count`·`char_offsets`·`field_ranges`를 비교한다.
- 새 integration source나 generated suite·manifest를 제출하지 않아 테스트 배치 정책을 우회하지 않는다.

코드 diff에서 검증기 예외, 종료 코드 완화, serializer 변경, Gym task/reference/oracle 변경은 발견되지
않았다. 범위 밖 변경이나 병합 blocker도 발견되지 않았다.

## 로컬·GitHub 검증

[최종 보고서](../../report/task_m100_6641_report.md)와
[Stage 4 기록](../../working/task_m100_6641_stage4.md)의 exact source/test candidate에서 다음 검증을
완료했다.

| 검증 | 결과 |
| --- | --- |
| 기존 field 56건 + 최신 devel 인접 layout 4건 | 60/60 통과 |
| native·WASM32·workspace all-targets Clippy `-D warnings` | 통과 |
| workspace build, fmt, manifest, unit-tier | 통과; 1,132 sources / 48 targets, 4,221 tests / 299 modules |
| 전체 integration nextest | 8,973/8,973 통과, 정책상 ignored 46건 |
| #6628 Gym positive·discrimination·trajectory | 1,035/1,035, false-pass 0, 239/239 load-bearing |
| latest-base merge simulation | tree `845cdcfb6d4bf4a5d81895f7499efe0f65631276`, 최초 PR head tree와 일치 |
| `git diff --check`, 변경 Markdown 링크 | 통과 |

최초 PR head의 GitHub Actions도 같은 source/test content를 Full Rust lane으로 검증했다.

- [CI 33705240921](https://github.com/edwardkim/rhwp/actions/runs/33705240921): preflight, 네 archive
  build/test, Lint, Build & Test 성공
- [CodeQL 33705241253](https://github.com/edwardkim/rhwp/actions/runs/33705241253): Rust·Python·
  JavaScript/TypeScript 분석 성공, 최종 CodeQL 집계 정상 neutral
- [Proptest 33705240889](https://github.com/edwardkim/rhwp/actions/runs/33705240889): preflight와
  prop roundtrip 성공
- [Adapter inter-diff 33705241027](https://github.com/edwardkim/rhwp/actions/runs/33705241027):
  preflight와 본 검사 성공
- [CI Impact Policy 33706535503](https://github.com/edwardkim/rhwp/actions/runs/33706535503):
  selective Rust 영향 판정 성공

## 시각 검증 판정

필드 편집 뒤 줄과 vpos가 바뀌므로 시각·fixture 보조 경로를 검토했지만, 이번 최종 판정에 한컴 기준
PDF나 pixel visual sweep을 요구하지 않는다. 이 PR은 renderer/layout 알고리즘, WASM·Studio 출력,
sample·기준 PDF·golden을 바꾸지 않고 기존 owner-aware reflow를 필드 mutation 관문에서 호출한다.
결함의 정답지는 특정 한컴 페이지와의 시각 동등성이 아니라 편집 직후와 저장·재적재 뒤의 동일한
LineSeg·vpos·필드 metadata 및 CLI verify 판정이다.

기존 공개 HWP/HWPX fixture의 본문·셀·글상자 구조 계약과 전체 Rust 회귀로 그 범위를 직접 확인했다.
따라서 영향도 정책에 따른 Native Skia·WASM·Frontend skip은 정상이며 이를 시각 fidelity 통과로
확대 해석하지 않는다. 이 PR은 한컴 조판 동등성이나 일반 renderer 개선을 주장하지 않는다.

## 잔여 위험과 후속 경계

- HWPX 합성 LineSeg의 저장 생략 때문에 깊은 HWPX edit verify가 저장 전·후 파생 상태를 일반적으로
  정규화하는 문제는 #6641에서 해결하지 않는다.
- #6628에는 독립·한컴 external oracle이 아직 0개다. Gym 통과는 내부 benchmark 정합 근거다.
- API 호출 순서 교환을 검사하는 `order-dependency audit`는 #6628 후속 백로그이며 #6641 비범위다.
- 성능 수치는 동일 host 단일 실행 기초값이다. 필드 mutation 때만 문단 reflow·vpos 연결 비용이
  추가되며 읽기·일반 렌더 경로의 성능 보장을 뜻하지 않는다.
- 로컬 nextest 0.9.137은 저장소 권고 0.9.140보다 낮았지만 버전 검사를 우회하지 않았고 전 테스트가
  종료 코드 0으로 완료됐다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: source/test candidate `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`와 최초 PR head
  `362e912a6f0da06dd3cbd902d069adba87f0ddfc`
- trailing 조건: 이 review·오늘할일·상태 현행화만 추가한 최신 head에서 review-only checks 성공,
  `MERGEABLE`·`CLEAN` 재확인
- merge 조건: 최신 head SHA 고정, 사용자 merge 승인, `--admin` 우회 없는 정상 2-parent merge commit
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 후: 실제 merge SHA와 #6641 자동 종료를 확인하고, #6628에 제품 blocker 해소 계보를 기록한
  뒤 최신 local `devel` 동기화와 승인된 branch/worktree 정리를 수행한다.
