---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4087 검토 - 검증된 ingest 경로 기반 HWPX 테스트 데이터 생성기

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4087](https://github.com/edwardkim/rhwp/pull/4087) / @kevin9327 |
| contributor 원 head | `aa68e4a05fefa5e4ee4ed47c11e13d9dabf32829` |
| 메인터너 보정 commit | `231924b8f2243021af5c07bf9fe57d2882f67d3d` |
| 최신 `devel` update merge | `84bd3be3ee26b0c17d1c6b1f301b12a1a1e9c905` |
| 가시성 검토 브랜치 | `review/kevin9327-4087-20260806` |
| 대상 경로 | `tools/test-data-gen/` 4개 파일 |
| 시각 검증 | 비대상. 생성기·문서·Python 회귀만 변경하며 renderer, Studio, HWP/HWPX fixture를 변경하지 않는다. |

원 contributor 변경은 수제 zip 조립을 제거하고, 템플릿을 결정적인 ingest JSON으로 바꾼 다음
`rhwp build-from-ingest --json`과 `rhwp info --json`으로 실제 HWPX를 생성·검증한다. 이는
[#4044](https://github.com/edwardkim/rhwp/issues/4044) 검토에서 지적된 `Contents/header.xml` 누락
비정합 HWPX를 표준 생성 경로로 대체한 후속 작업이다.

## 발견 사항과 메인터너 보정

원 구현은 사용자 JSON의 template 이름을 결과 파일명에 그대로 사용했다. 실제 `../escaped` 이름은
성공으로 반환하면서 `--output-dir` 상위에 `escaped.hwpx`를 만들었다. 또한 `--rhwp-bin`은 존재 여부만
확인해 비실행 일반 파일에서 Python `PermissionError` traceback과 종료 1을 반환했고, README가 약속한
설정 오류 종료 2 계약을 지키지 못했다.

메인터너 보정 `231924b8f`은 template 이름을 단일 파일명으로 제한했다. 빈 값, `.`·`..`, NUL,
POSIX/Windows 경로 구분자와 절대 경로를 거부한다. `--rhwp-bin`·`RHWP_BIN`은 이제 실행 가능한 일반
파일만 허용하고, 그 밖의 경우는 traceback 없이 `ConfigError`와 종료 2로 종료한다. README에는 출력
경계와 실행 권한 조건을 명시했고, unsafe 이름·경로형 config·비실행 바이너리 회귀를 추가했다.

최신 `devel` update merge `84bd3be3e`는 contributor 원 commit을 rewrite하지 않고 보정 위에만
추가했다. `git merge-tree --write-tree 231924b8f 09ff0c3bd` 결과와 merge tree가
`9c8f590a3352883f46e99eb8d92629afdf0068a2`로 일치함을 확인했다. 따라서 이 review·오늘할일
single-parent commit은 [#4102](https://github.com/edwardkim/rhwp/pull/4102)의 current-base
review-only fast-pass를 실제 PR에서 검증하는 후보다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 생성기 회귀 | 전용 `target/review-pr4087`의 `rhwp`로 `python3 tools/test-data-gen/test_hwp_test_data_generator.py`를 실행해 16 passed를 확인했다. |
| 실물 HWPX 생성 | 기본 템플릿 minimal·simple·structured·media·large가 각각 `build-from-ingest`와 독립 `rhwp info --json` 검증을 통과했다. 페이지 수는 1·1·2·1·7이다. |
| unsafe output 차단 | `../escaped` template을 실제 실행해 `ConfigError`로 거부되고 output 경계 밖 파일이 생성되지 않음을 확인했다. |
| 비실행 binary 계약 | 실행 권한 없는 일반 파일에서 traceback 없이 설정 오류 종료 2를 확인했다. |
| Python·공백 검사 | `python3 -m py_compile`과 `git diff --check`를 통과했다. |
| review-only workflow | `test_review_only_fast_pass_workflows.py` 4 passed, `test_ci_impact_workflow.py` 18 passed, `ci-impact-classifier.test.cjs` 27 passed를 확인했다. |
| 병합 정합 | 최신 `devel`로의 no-conflict update merge와 `merge-tree` tree 일치를 확인했다. |

## GitHub Actions와 수용 판단

보정 code head `231924b8f`의 [CI 31090102442](https://github.com/edwardkim/rhwp/actions/runs/31090102442?pr=4087)와
[CodeQL 31090101925](https://github.com/edwardkim/rhwp/actions/runs/31090101925?pr=4087)는 모두 성공했다.
CI preflight, Lint, Frontend package gates, Native Skia, archive build 3개, 기본 test shard 4개,
Build & Test aggregate 및 CodeQL Python/JavaScript/Rust 분석이 성공했다.

**메인터너 보정 포함 수용.** review-only commit push 뒤 current-base fast-pass aggregate와 최신
`mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`을 다시 확인한 뒤 병합한다.
