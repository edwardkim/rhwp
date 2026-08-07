---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4073 검토 - schema-validator 중첩 오류 전파와 `oneOf` 판정

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4073](https://github.com/edwardkim/rhwp/pull/4073) / @kevin9327 |
| contributor 원 head | `a59f8bc36aff9acabca566e75e54dda874735bec` |
| 메인터너 보정 commit | `f106912300ef79504ca0bf272e6565e3902967b4` |
| 최신 `devel` update merge | `49d2511e9` |
| 가시성 검토 브랜치 | `review/kevin9327-4073-20260806` |
| 대상 경로 | `tools/schema-validator/` 3개 파일 |
| 시각 검증 | 비대상. Rust renderer, HWP/HWPX fixture, Studio 화면 변경이 없다. |

원 contributor 변경은 canonical ingest schema를 직접 읽는 표준 라이브러리 Python 검증기, README,
회귀 테스트 15건을 추가한다. 관련 [#4044](https://github.com/edwardkim/rhwp/issues/4044)는 이미 닫힌
상위 검토 기록이며, 이번 PR은 그때 요청된 schema-validator 결함만 보완한다.

## 발견 사항과 메인터너 보정

최초 검토에서 `oneOf`가 정확히 하나와 일치할 때 선택된 대안의 scratch WARNING을 버리는 결함을
확인했다. 따라서 아래 두 입력은 검증기가 `valid:true`, 오류 0건, 경고 0건, 종료 코드 0으로 통과시켰다.

```json
{"type":"text","text":"a","bold":true}
{"type":"text","text":"a","ref":"img/q1.png"}
```

그러나 Rust `StemBlock`은 `RawStemBlock`의 `deny_unknown_fields`와 변형별 필드 검사로 두 입력을
거부한다. 검증기 README가 약속한 사전 WARNING과 실제 `build-from-ingest` 결과가 어긋나는 차단 결함이었다.

메인터너 보정 `f10691230`은 정확히 하나와 일치한 대안의 WARNING만 최종 오류 목록으로 옮겼다. 매치하지
않은 대안의 진단은 계속 격리하므로 draft-07 `oneOf`의 0개/2개 이상 ERROR 판정은 바뀌지 않는다. `bold`와
image 전용 `ref`를 각각 넣는 API 회귀와 CLI의 `valid:true`·경고 1건·종료 0 계약을 추가했고, README도
선택된 분기 기준의 `UNKNOWN_FIELD` 의미로 정정했다.

최신 `devel` update merge `49d2511e9`는 contributor 원 commit을 rewrite하지 않고 보정 commit 위에만
추가했다. 같은 날짜의 오늘할일과 review archive를 포함한 기준선을 반영해 PR 병합 시 add/add 충돌이 나지
않도록 했다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Python 구문 검사 | `python3 -m py_compile tools/schema-validator/schema_validator.py tools/schema-validator/test_schema_validator.py`를 실행해 통과했다. |
| schema-validator 회귀 | `python3 tools/schema-validator/test_schema_validator.py`를 실행해 17 passed를 확인했다. |
| 실물 schema sample | `sample_minimal.json`, `sample_structured.json` 모두 오류 0건·경고 0건·종료 0을 확인했다. |
| `oneOf` WARNING CLI | `bold`, `ref` 각각 경고 1건·`valid:true`·종료 0을 실제 CLI JSON 출력으로 확인했다. |
| 공백 오류 | 코드 보정 및 update merge 뒤 `git diff --check`를 실행해 통과했다. |
| 병합 정합 | 보정 전 최신 `devel` 위 no-commit merge simulation은 충돌·공백 오류 없이 통과했다. 이후 source 자체를 최신 `devel`에 update merge했다. |

## GitHub Actions와 수용 판단

보정 code/test head `f10691230`의 전체 GitHub Actions와 CodeQL은 통과했다. CI preflight, Lint,
Frontend package gates, Native Skia, archive 빌드 3개, 기본 test shard 4개, Build & Test aggregate,
CodeQL JavaScript/Python/Rust 분석이 모두 성공했다. renderer 변경이 없으므로 별도 시각 증적은 요구하지
않았다.

**메인터너 보정 포함 수용.** 최신 `devel` update merge와 이 archive review 묶음은 원 contributor 변경을
수정하지 않고 그 위에만 쌓였다. 최종 merge 전에는 현재 PR head의 CI, `mergeable=MERGEABLE`,
`mergeStateStatus=CLEAN`을 다시 확인한다.
