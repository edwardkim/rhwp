# PR #9204 검토 — Native Skia CI의 APT 실패 재시도

**승인 — 정확한 코드 head의 Full CI를 확인하고 devel에 병합했다.**

[PR #9204](https://github.com/edwardkim/rhwp/pull/9204)는 `jangster77`이 작성했다. 검증 head `74a531951b65a7769d6503f8369dbbe2d84ffc51`, 검증 base `9440d05e7d4738151822d02a8392d6938796e474`. 병합 직전 최신 devel `341f36138e656401acad2d9b07488d719552b0cd`에서 CI·스크립트 변경 경로에 차이가 없고 병합 시뮬레이션이 충돌 없이 끝났다. 2026-11-19 23:24:39 KST에 [`b68e334c2`로 병합](https://github.com/edwardkim/rhwp/commit/b68e334c2edbbea22ddfb3d5e13db74a3836282b)했다.

## 원인과 변경

[#9200의 Native Skia 잡](https://github.com/edwardkim/rhwp/actions/runs/44530849923/job/135584318195)과 [#9202의 잡](https://github.com/edwardkim/rhwp/actions/runs/44533963794/job/135594571759)에서 Ubuntu 저장소 응답을 기다리던 `apt-get update`가 180초 제한으로 종료됐다. [#9203](https://github.com/edwardkim/rhwp/issues/9203)은 같은 CI 잡에서 일시적인 APT 실패를 복구하는 범위다.

- `update`와 `install` 각각에서 실패한 명령을 60초 뒤 다시 실행한다. 최초 실행 후 최대 3회 재시도하며, 호출마다 기존 180초 상한을 적용한다.
- 마지막 시도도 실패하면 그 종료 코드를 잡에 전달한다. 설치가 끝나면 원래와 같이 글꼴·개발 라이브러리를 다시 확인하고 Native Skia 결과를 필수 Build & Test에서 확인한다.
- 변경 파일은 `.github/workflows/ci.yml`과 `scripts/tests/test_ci_impact_workflow.py`다. 제품 코드와 조판은 변경 범위 밖이다.

## 검증

- 실제 workflow 셸에 APT·대기 대역을 연결했다. 수정 전 재시도 계약 4건이 실패했다. 수정 후 `python3 -m unittest scripts.tests.test_ci_impact_workflow -v` 81건이 통과했다. 세 번째 재시도 성공, 재시도 소진 뒤 최종 종료 코드, 설치 실패 시 갱신을 반복하지 않는 경로를 확인했다. `actionlint .github/workflows/ci.yml`과 `git diff --check`도 통과했다.
- [코드 preflight](https://github.com/edwardkim/rhwp/actions/runs/44537439407/job/135606631514)는 `fast_pass=false`, `ci-execution-surface-change`로 판정했다. 후보 head에서 검사 32건 성공, 5건 조건부 건너뜀, 실패 0건이었다. [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/44537439407/job/135606751935)와 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44537439407/job/135624351171)가 통과했다. 원격 APT 갱신과 설치는 첫 시도에 성공했으며 실패 복구 경로는 위 로컬 계약에서 검증했다. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/44537439407) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44537439169) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44537439093) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44537439248)
- 코드 PR 병합 후 [#9203](https://github.com/edwardkim/rhwp/issues/9203)이 2026-11-19 23:24:59 KST에 자동으로 닫혔다. 이 검토와 오늘 할 일은 옵션 2에 따라 별도 문서 PR로 반영한다.

## 병합 후 확인

[테스트 소요 시간 갱신](https://github.com/edwardkim/rhwp/actions/runs/44543439410)은 원 코드 CI 첫 실행의 worker B–E 측정값으로 44개 대상을 갱신했다. 게시 커밋은 `4a1ae612248f28346bbd769582092c9d26cf0906`이다.
