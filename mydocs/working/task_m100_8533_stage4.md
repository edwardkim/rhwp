# #8533 제출 CI의 프런트엔드 디스크 부족 보정

## 확인한 실패

[PR #8565](https://github.com/edwardkim/rhwp/pull/8565)의 제출 code head는
`f5d3de4cf0b2bca688de529dc3de4b4b1f720fe6`다.
[Frontend package gates](https://github.com/edwardkim/rhwp/actions/runs/41758005032/job/126339810329)는
2026-11-04 11:10:48 KST에 여유 공간 55MB를 경고했고, WASM dev 링크 중
`rustc-LLVM ERROR: IO failure on output stream: No space left on device`로 실패했다.
이 실패를 제품 회귀 테스트의 assertion 실패나 통과로 기록하지 않는다.

같은 workflow의 archive·Native Skia job에는 30GB 미만일 때 미사용 사전 설치 도구를 정리하는
단계가 있지만 프런트엔드 package job에는 없었다. 기존 캐시는 약 724MB 압축본이며, 그 크기만으로
최종 WASM 링크 작업공간이 충분하다고 판단할 수 없다.

## 보정과 경계

`frontend-package-gates`의 checkout 이후, Rust 설치·캐시 복원·fresh WASM 빌드 이전에 기존
Native와 같은 공간 점검을 추가한다. 여유 공간 30GB 미만에서만
`/usr/local/lib/android`와 `/usr/share/dotnet`을 제거한다. 전후 `df`와 대상 용량을 기록한다.
작업공간·원본·캐시·검증 결과를 삭제하거나 검사를 생략하지 않는다.

운영 변경 분류는 job 명령을 추가하는 O3이다. 이벤트·권한·required check 이름·CI 영향 판정·
캐시 신뢰 경계와 제품 소스·검증 호스트·렌더링 증적은 바꾸지 않는다. 새 CI head에서 전체 검사를
다시 확인하며, 과거 실행 중·실패한 결과를 새 head의 통과로 재사용하지 않는다.

## 로컬 검증

- `actionlint .github/workflows/ci.yml`: exit 0, 진단 없음.
- CI 영향·workflow 배선·review-only·Undo E2E 연결의 기존 Python 계약: 71 PASS, 18.233초.
- 변경한 실제 shell 명령을 `df`·`sudo` 모의 프로그램으로 실행: 여유 공간 1·29GB에서 지정한
  미사용 도구 두 경로만 제거를 요청하며, 30·60GB에서는 제거를 요청하지 않는 4건 PASS.
  실제 Mac의 시스템 도구를 삭제하는 실행이 아니다.
- `git diff --check`: PASS. Rust·TS·시각 출력은 변경하지 않아 이미 검증한 Git object와 같다.

로그·모의 실행기·중간 JSON은 ignored output에만 보존한다. 임시 실행기의 첫 PyYAML import 실패는
필요 없는 의존성을 제거한 뒤 실제 shell 분기 실행으로 정정했으며 최종 검증과 혼합하지 않는다.

## 후속 기록 경로

이 보정으로 실제 `.github/workflows/ci.yml` 변경이 포함되므로
[옵션 2](../manual/pr_review/post_merge.md#711-ci-변경-pr은-옵션-2로-후속-기록)를 적용한다.
원 PR의 새 code head 전체 CI·실제 검토·병합을 먼저 완료하고, 검토 기록·오늘할일·확정 merge SHA는
최신 devel의 별도 기록 PR로 반영한다. 원 PR에 문서-only trailing commit을 추가하지 않는다.
