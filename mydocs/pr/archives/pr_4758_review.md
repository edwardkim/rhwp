---
kind: pr-review
status: active
issue: 4756
pr: 4758
---

# PR #4758 리뷰 - Studio e2e 계약의 gym 과제 생성

## 접수와 누적 적용

| 항목 | 값 |
| --- | --- |
| PR | [#4758](https://github.com/edwardkim/rhwp/pull/4758) |
| 작성자 | `kevin9327` |
| source head | `40935d6ab2489e6985f05871e533c3220596f7c3` |
| 통합 순서 / 적용 commit | 2 / `129e38cfe`, `995f5c4a9` |
| 통합 PR | [#4767](https://github.com/edwardkim/rhwp/pull/4767) |
| 검증 code candidate | `f97dd8a9b47298b1b6a1e3050045dd955d662c87` |

이 PR은 Studio e2e의 `gymContract` 리터럴에서 CLI gym 과제의 CSV·task·reference를 생성한다.
브라우저 UI의 메뉴, 대화상자, undo 검증을 CLI에 중복 이식하지 않고, 두 경로가 공유하는 차트 데이터
계약만 gym에서 재현한다는 범위는 적절하다.

## 메인터너 보정

원 source의 `SE01`은 기존 `security/SE01`과 전역 task ID가 충돌했고,
`gym/profiles/maintainer.json`에 `studio-e2e`가 없어 repository contract test가 실패했다.
또한 정적 파싱을 표방했지만 `Function(...)`으로 e2e 파일의 표현식을 실행할 수 있었다. 신뢰하지 않는
source에서 생성기를 실행하는 경계에 맞지 않는다.

통합 branch에서 contributor commit을 재작성하지 않고 별도 commit `f97dd8a9b`로 다음을 보정했다.

- task ID를 `ST01`로 바꾸고 maintainer profile에 `studio-e2e`를 등록했다.
- 주석, 문자열, 유한 수와 중첩 객체만 허용하는 bounded literal parser로 교체했다.
- 다른 pack의 task ID를 생성 전에 검사하고, 생성 결과의 LF 직렬화와 재실행 불변성을 고정했다.
- 실행 표현식 거부, ID 충돌 거부, 유효 계약 수용을 검증하는 Node 회귀 3건을 추가했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `node --test gym/tools/from_e2e_contract.test.mjs` | 3 passed |
| gym Python contract 묶음 | 50 passed, 1 skipped |
| 실제 e2e 계약에서 `ST01` 생성 후 재생성 | CSV·task·reference SHA 동일 |
| `build_baseline` 및 `gym/score.py` | studio-e2e 3/3, 1/1 과제 |
| chart data edit headless e2e | 메뉴·대화상자·값 변경·Ctrl+Z·음성 경로 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| Studio build / unit | build 통과, 923 passed |
| release-test nextest | 6,021 passed, 38 skipped, 6 slow |
| Clippy 및 diff/merge-tree | 통과 |
| #4767 GitHub CI | Build & Test, CodeQL, Lint, Native Skia, Canvas visual diff 통과 |

**권고: 메인터너 보정 포함 수용.** trailing docs-only head의 fast-pass aggregate와 최신
`MERGEABLE`/`CLEAN`을 확인한 뒤 #4767로 merge한다. [#4756](https://github.com/edwardkim/rhwp/issues/4756)의
종료 상태와 원 PR close comment는 merge 뒤 공식 절차로 처리한다.
