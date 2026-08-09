---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4087 메인터너 보정 기록

## 대상과 commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `aa68e4a05fefa5e4ee4ed47c11e13d9dabf32829` | contributor의 build-from-ingest 생성기, 템플릿, README, 회귀 테스트 |
| 2 | `231924b8f2243021af5c07bf9fe57d2882f67d3d` | 출력 경계·실행 가능 binary 계약 보정과 회귀·README 추가 |
| 3 | `84bd3be3ee26b0c17d1c6b1f301b12a1a1e9c905` | 최신 `upstream/devel`을 같은 가시성 브랜치에 update merge |

`maintainerCanModify=true`를 확인했다. contributor 원 commit은 rebase, amend, reset, force-push하지
않았고, 보정은 `review/kevin9327-4087-20260806` 위의 별도 single-parent commit으로만 추가했다.

## 완료한 보정

1. config template 이름을 output directory 내부의 단일 파일명으로 제한했다.
2. 빈 이름, `.`·`..`, NUL, `/`, `\\`, 절대 경로를 configuration error로 차단했다.
3. `--rhwp-bin`과 `RHWP_BIN`에 대해 존재 여부 외에 regular file·실행 권한을 검사했다.
4. 비실행 binary는 Python traceback이 아니라 기존 설정 오류 계약인 종료 2로 반환하게 했다.
5. 실제 path traversal, path-like config, non-executable binary 회귀와 README의 출력·binary 조건을 추가했다.

## 검증 결과

| 항목 | 결과 |
| --- | --- |
| `test_hwp_test_data_generator.py` | 16 passed |
| 기본 템플릿 실물 생성 | 5종 전부 `rhwp info --json` 통과, 1·1·2·1·7쪽 |
| unsafe output / 비실행 binary | 각각 output 경계 보존·종료 2 계약 확인 |
| Python 구문·공백 | `py_compile`, `git diff --check` 통과 |
| fast-pass 회귀 | workflow 4건, CI 영향 18건, classifier 27건 통과 |
| 보정 head GitHub Actions | 전체 CI, Build & Test aggregate, CodeQL 성공 |
| current-base merge tree | source `231924b8f`와 base `09ff0c3bd`의 생성 tree가 update merge tree와 일치 |

## 원격 반영과 merge 전 조건

보정 code/test commit `231924b8f`은 contributor source `pr/tool-test-data-gen`에 fast-forward push했다.
이후 최신 `devel` update merge와 archive review·오늘할일 single-parent commit을 같은 source branch에
순서대로 반영한다. [#4102](https://github.com/edwardkim/rhwp/pull/4102)의 current-base fast-pass가
candidate `231924b8f`를 재사용해 heavy worker 없이 aggregate 성공을 반환하는지 확인한 뒤, 최신
mergeability와 작업지시자 승인에 따라 병합 및 후속 정리를 수행한다.
