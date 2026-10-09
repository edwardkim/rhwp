# #7473 PR Render Diff release WASM 검증·CI 비용 결과

[Issue #7473](https://github.com/edwardkim/rhwp/issues/7473) · [Draft PR #7474](https://github.com/edwardkim/rhwp/pull/7474)

## 결론

PR Render Diff에서 배포와 같은 release + wasm-opt 경로를 검증하도록 전환할 수 있다.
동일 코드의 dev/release 각 3회가 모두 성공했고, 각 실행의 PNG 43개가 프로필·반복 간 byte 단위로 같았다.
다만 캐시 없는 전체 job 중앙값은 **8분 20초 → 12분 40초(+4분 20초, +52%)**다.
검증 경로 정렬이라는 기여 효과와 추가 CI 비용을 함께 판단해야 한다.

wasm-opt/마무리는 약 118초로 후속 비용 절감 조사 가치가 있다. 현재 PR은 검증 경로와 관측을
정리하며, 캐시 구현은 별도 검증 후 별도 PR로 다룬다. 이번 표본은 모두 cache miss이므로
캐시 적중 시 절감량이나 최적화 결과 재사용의 안전성을 입증하지 않는다.

## 구현과 검증 범위

- PR 실행은 release로 고정하고 수동 실행에만 dev/release 비교 입력을 제공한다.
- 기존 locked wrapper를 통해 빌드하며 tool 실행 명령·버전·SHA-256, source/lockfile 및 산출물 hash를 보존한다.
- E2E 앱 탐색 전에 CDP 관측을 연결해 실제 수신 WASM 응답을 manifest와 대조한다.
  별도 fetch로 확인하지 않으며, 응답 누락·오류·불일치 및 release의 opt 실행 누락은 실패한다.
- Cargo·bindgen 준비/실행·opt/마무리 근사 시간을 Actions step/job 시간과 구분해 기록한다.

로컬 구현·검증 명령과 이전 운영 기준선은 [작업 기록](../working/task_m100_7473_stage1.md),
입력 hash와 self-review는 [PR 검토 기록](../pr/archives/pr_7474_review.md)에 있다.
Rust 제품 코드·Cargo 프로필·렌더링 알고리즘·시각 기준값은 바꾸지 않았다.

## 통제 조건과 측정 방법

측정 source SHA는 `ddf5ce2d2c13d4c87aa4a3b165d39b7506d3a25f`다. 동일 SHA를 가리키는
6개의 측정 ref에서 `PR Render Diff`를 `workflow_dispatch`로 실행했다.
각 `wasm-profile=dev/release`, `write-images=true`이며 동일 branch concurrency에 의한 취소를
피하려고 ref를 분리했다. 작업 tree는 모두 clean이고 fixture/lockfile은 동일하다.

| 조건 | 6회 공통 값 |
| --- | --- |
| Runner image | Ubuntu 24.04 / `20260920.314.1` |
| 브라우저 | Chrome `152.0.7946.0`, Chromium build `1660786` |
| 설치 폰트 패키지 | fonts-dejavu-core `2.37-8`, 저장소 폰트·fixture 동일 |
| Rust / wasm-pack | `1.93.1` / `0.15.0` |
| wasm-bindgen | `0.2.127`, executable hash 동일 |
| release wasm-opt | `117`, `-O`, executable hash 동일 |
| Cargo cache | 6/6 miss, matched key 없음, 측정 ref에서 저장 없음 |
| 작업 병렬도 | Cargo 기본값, 개별 GitHub hosted VM |

Actions attempt별 jobs API의 step/job 시작·종료 시각을 사용했다. 전체 job은 Canvas visual diff의
시작부터 종료까지이며 workflow 큐 대기·preflight는 포함하지 않는다. Cargo·bindgen·opt 구간은
로그 수신 경계의 wall 근사다. bindgen 구간에는 도구 준비가, opt 구간에는 패키징 마무리가
포함될 수 있으므로 독점 CPU 시간이나 격리된 프로세스 실행 시간으로 해석하지 않는다.
WASM step은 측정 wrapper 외부의 metadata 수집 등도 포함한다.

새 관측 workflow 안에서 프로필만 비교했다. 이전 workflow 전체와 새 workflow 전체의 관측 오버헤드를
분리한 실험은 아니다. VM의 CPU·네트워크 부하는 고정하지 못했고 표본은 각 3회다.

## 실행별 관측값

단위: 초. Cargo와 opt는 로그 경계 근사, 나머지는 Actions API 값이다.

| 실행 | WASM step | Cargo 근사 | opt/마무리 근사 | Canvas/PDF | Readiness | 전체 job | 결과 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [dev 1](https://github.com/edwardkim/rhwp/actions/runs/36436604581/job/108976114154) | 122 | 99.613 | — | 58 | 84 | 498 | 성공 |
| [release 1](https://github.com/edwardkim/rhwp/actions/runs/36436608897/job/108976158258) | 443 | 303.178 | 120.887 | 43 | 72 | 788 | 성공 |
| [dev 2](https://github.com/edwardkim/rhwp/actions/runs/36436613036/job/108976187638) | 127 | 104.478 | — | 58 | 84 | 525 | 성공 |
| [release 2](https://github.com/edwardkim/rhwp/actions/runs/36436617318/job/108976190130) | 421 | 285.084 | 118.144 | 43 | 72 | 760 | 성공 |
| [dev 3](https://github.com/edwardkim/rhwp/actions/runs/36436621337/job/108976198072) | 114 | 93.463 | — | 58 | 85 | 500 | 성공 |
| [release 3](https://github.com/edwardkim/rhwp/actions/runs/36436626093/job/108976558861) | 369 | 236.616 | 118.129 | 37 | 65 | 677 | 성공 |

## 중앙값과 범위

단위: 초, `중앙값 (최소–최대)`.

| 측정 항목 | dev, n=3 | release, n=3 |
| --- | ---: | ---: |
| WASM build step | 122 (114–127) | 421 (369–443) |
| Cargo 근사 | 99.613 (93.463–104.478) | 285.084 (236.616–303.178) |
| bindgen 준비/실행 근사 | 9.081 (8.583–9.686) | 4.671 (4.082–4.746) |
| wasm-opt/마무리 근사 | 실행하지 않음 | 118.144 (118.129–120.887) |
| Native CLI step | 122 (117–125) | 119 (100–124) |
| Canvas/PDF step | 58 (58–58) | 43 (37–43) |
| Readiness step | 84 (84–85) | 72 (65–72) |
| 전체 Canvas job | 500 (498–525) | 760 (677–788) |

WASM build 중앙값은 +299초(3.45배), 전체 job은 +260초다. 시각 검사 구간은 이 표본에서 짧았으나
3회만으로 일반적인 성능 개선을 단정하지 않는다. 서로 다른 중앙값의 차이를 합산해 정확한 시간
분해로 취급할 수 없다. release Cargo 약 285초가 가장 큰 구간이고, opt 약 118초는 WASM step의
약 28%, 전체 job의 약 16%에 해당한다.

## 출력과 실제 WASM 소비 확인

| 검사 | 관측 결과 |
| --- | --- |
| Ubuntu Canvas/PDF 및 readiness | 6회 성공, readiness 각각 8/8 통과 |
| 실제 WASM 응답 hash | 각 18건, 총 108건이 해당 실행의 source/profile/artifact와 일치 |
| PNG 프로필·반복 비교 | 실행마다 43개(시각/PDF 27 + readiness 16), dev 1 기준 변경 0 |
| WASM 반복 재현 | 프로필별 3회 bytes/hash 동일 |
| 대표 CI 이미지 직접 판독 | dev 1/release 1의 KTX p1 지도·시간/요금표와 table-border-style 표·세로 글줄의 위치/형태 동일 |
| macOS 로컬 readiness | dev/release 공통 7/8, table-border-style의 visualParityFailed를 보존 |

CI WASM은 dev 41,227,131 bytes, release 11,341,620 bytes다. SHA-256과 tool hash는 아래 JSON에 있다.
PNG 동등성은 프로필 변경으로 관측한 출력 차이가 없다는 증거다. 한컴 정합이나 로컬 renderer 간
기존 차이 해결을 주장하지 않는다. macOS 실패는 Ubuntu 통과와 분리해 기록하며 기준값을 완화하지 않았다.

최종 코드의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/36436443999),
[PR Render Diff](https://github.com/edwardkim/rhwp/actions/runs/36436442474),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/36436443955) 및 CI Impact Policy가 성공했다.
초기 후보 `a4220e1`의 Lint 실패는 trusted policy 경로 mirror 누락으로, 최종 후보에서 수정했다.
이 초기 실행과 서로 다른 SHA의 예비 측정은 위 6회 통계에 포함하지 않았다.

## 후속 개선 순서

1. 이 PR의 배포 프로필 검증과 추가 CI 비용을 maintainer가 검토한다. PR은 Draft로 유지한다.
2. 후속 조사에서 trusted default branch seed와 PR restore의 실제 적중률·복원 비용을 확인한다.
   Cargo가 가장 큰 구간이므로 opt 캐시만으로 전체 비용을 해결한다고 가정하지 않는다.
3. 같은 opt 입력이 반복되는 빈도와 약 118초 비용이 warm CI에서도 남는지 측정한다.
4. 반복 비용이 확인되면 입력 WASM·도구 executable/version·옵션을 key에 반영하는 결과 캐시를
   별도 PR로 검증한다. miss/hit 결과 동등성, 잘못된 key의 무효화, 신뢰 경계, 캐시 다운로드 비용을
   확인하고 실제 순절감량으로 채택 여부를 결정한다.

현재까지는 검증 경로 정렬과 비용 측정이 완료됐고, 캐시 최적화 효과는 미검증이다.
별도 캐시 구현·ready 전환·merge·이슈 종료는 수행하지 않았다.

## 증적

- [CI 실행·시간·도구·산출물 원자료 요약](assets/issue-7473/ci-results.json)
- [환경·readiness·PNG 해시·소비 증거 집계](assets/issue-7473/ci-evidence.json)
- 각 실행 링크의 `render-diff-artifacts`에 build log/manifest/consumption, PNG, baseline report가 있다.
  Actions artifact 보존 기간은 14일이며, 위 JSON은 repository에 남긴다.
- 원본 다운로드는 로컬 `output/issue-7473/ci-controlled/`에 보존했다.
