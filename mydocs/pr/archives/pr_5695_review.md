---
kind: pr-review
status: approved
pr: 5695
issue: 3315
---

# PR #5695 검토 기록 - Track 4 그림 편집 성능 종결 프로브

- PR: [#5695](https://github.com/edwardkim/rhwp/pull/5695) `test(#3315): Track 4 종결 측정 프로브를 추가한다`
- 관련 이슈: [#3315](https://github.com/edwardkim/rhwp/issues/3315), 후속 [#5694](https://github.com/edwardkim/rhwp/issues/5694)
- 작성자: `@lpaiu-cs`, `maintainer_can_modify=true`
- source code candidate: `034ac279d55c820e036d089374d6a85554d9b26c`
- 검토 기준: `upstream/devel@1139f28d1` 위 `review/open-prs-20260820`
- 체리픽: `9ab35483b` (`-x`, 원 작성자·원 SHA 보존)
- 메인터너 보정: `c264a575d`가 0ms baseline에서 `NaN`과 거짓 FAIL을 내던 종결 판정을 해상도 이하 상태로 구분한다.
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `multi_pr_update_branch`

## 검토 범위

- 그림 없음과 JPEG 1장 문서를 같은 브라우저 세션에서 측정하는 Track 4 진단 프로브를 추가한다.
- 타이핑, `document-changed`, 개체 이동 1틱과 bridge 호출 누적 시간을 기록한다. 산출은 gitignored `output/issue-3315/`에 남으며 CI 게이트가 아니다.

## 보정 사유

- 첫 실행에서 baseline과 JPEG의 `document-changed`가 모두 `0.00 ms`라 `0 / 0`이 `×NaN`으로 표시됐고, 프로브는 “종결 기준 FAIL”을 출력하면서 exit 0이었다.
- 보정 후 양쪽이 타이머 해상도 이하이면 비율 대신 `해상도 이하 (양쪽 0.00 ms)`와 PASS를 기록한다. baseline만 0ms이고 JPEG 측정이 검출되면 FAIL로 남으므로 회귀를 숨기지 않는다.

## 검증 근거

- `CARGO_TARGET_DIR=target/pr-review wasm-pack build --target web --release`가 성공했다.
- `npm --prefix rhwp-studio run e2e:issue-3315-perf`를 보정 후 재실행했다. 타이핑은 `×0.89 PASS`, `document-changed`는 양쪽 해상도 이하 PASS, 개체 이동은 약 `28,571 fps`였고 종결 기준은 PASS였다.
- source head의 Frontend package gate, Canvas visual diff, CodeQL, Proptest, adapter inter-diff가 성공했다.

## 결론

**승인 (메인터너 보정 포함).** 프로브는 환경별 수치를 CI 합격 조건으로 쓰지 않고 재현 가능한 측정 경로만 제공한다. 0ms 타이머 해상도 경계도 의미를 보존하도록 정정했다. #3315와 #5694는 성능 후속 범위를 계속 추적하므로 이번 수용으로 닫지 않는다.
