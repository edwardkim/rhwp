# #6689 Stage 5 — Fuzz smoke 6-matrix dogfood

## 1. 실행 기준

실행 직전 로컬 `upstream/devel`과 원격 `refs/heads/devel`은 모두
`2c144b180dd776aa450c499778510199ae6cdf89`였다. 같은 SHA의 기존 `workflow_dispatch` run이 없는 것을
확인하고, 메인테이너 승인에 따라 `max_total_time=60`으로 한 번만 dispatch했다.

| 항목 | 고정 값 |
| --- | --- |
| workflow | `.github/workflows/fuzz-smoke.yml` |
| Git blob | `b9c7394b1711d6d73463682a2989188cc42e5c30` |
| content SHA-256 | `425ce384543f6b064149dea2be4c3c567352634b6dea81adb1a5269e7ceb0f97` |
| run | [33959858373](https://github.com/edwardkim/rhwp/actions/runs/33959858373) |
| event / branch | `workflow_dispatch` / `devel` |
| run head SHA | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 실행 시각 | 2026-09-05 10:08:45Z ~ 10:21:27Z |
| run 결론 | `failure` |

기계 판독 영수증은
[`fuzz-smoke-dogfood.json`](../tech/investigations/issue-6689/fuzz-smoke-dogfood.json)에 있다. 재현 입력은
저장소에 복제하지 않고 GitHub Actions artifact와 내용 digest로 고정했다.

## 2. 6-matrix 전건 결과

| target | job ID | 결론 |
| --- | ---: | --- |
| `parse_hwp` | `101289731633` | success |
| `parse_hwp3` | `101289731577` | success |
| `parse_hwpx` | `101289731494` | success |
| `parse_hml` | `101289731632` | success |
| `parse_wmf` | `101289731422` | **failure** |
| `parse_ooxml_chart` | `101289731532` | success |

6개 job은 모두 exact `devel` candidate에서 실제 생성됐고, 5개 성공과 1개 실패가 빠짐없이 보존됐다.
따라서 Stage 5의 “실행 여부와 결론 보존” 종료 게이트는 통과했다. run을 녹색으로 만들기 위한 재시도,
waiver, `continue-on-error` 변경은 수행하지 않았다.

## 3. `parse_wmf` 제품 결함

`parse_wmf`는 seed corpus를 읽은 뒤 1,177개 unit을 실행하다 Rust 정수 부호 반전 오버플로로 panic했다.
libFuzzer의 직접 요약은 `deadly signal`, target exit status는 77이며 workflow step은 exit code 1로
정상 실패했다. 실패 artifact upload step은 성공했다.

| artifact 항목 | 값 |
| --- | --- |
| ID / 이름 | `9967737012` / `fuzz-crash-parse_wmf-33959858373` |
| 재현 입력 | `crash-ca3947cba11424abf10ceb43db04ca4e48c2bd8c` |
| 크기 | 84 bytes |
| SHA-256 | `2ac4a5d4e0c6ef488d6423151035068a36968c4fdb4c499016f29d90ecc6c2fa` |

직전 schedule run `33951593729`도 5개 성공과 `parse_wmf` 1개 실패였고 crash artifact를 남겼다. 다만
그 입력의 크기와 digest는 이번 입력과 다르므로, 같은 target의 반복 실패라는 사실까지만 확정한다. 두 입력의
동일 code-path 여부는 #6689의 workflow 승격 보호장치가 아니라 별도 WMF 제품 결함에서 분석해야 한다.

빌드 과정의 binary kebab-case 메시지는 warning이며 실패 원인이 아니다. 직접 실패 원인은 로그의
`attempt to negate with overflow` panic이다.

## 4. 판정 분리

- **Stage 5 증적 완결성**: 통과 — exact SHA, workflow hash, 6개 job, 실패 artifact를 모두 보존했다.
- **Fuzz 제품 건전성**: 실패 — `parse_wmf` panic이 현재도 존재한다.
- **#6689 promotion 장치 판정**: 영향 없음 — 이 이슈는 제품 실패를 녹색으로 바꾸는 작업이 아니라,
  workflow 실행 누락과 실패 은폐를 방지하는 작업이다. 이번 run은 오히려 그 경계가 필요한 이유를 실증했다.

Stage 6에서는 현재 task branch의 정적 검증과 원격 drift를 다시 확인하고, 최종 보고서에 이 두 판정을
분리해 기록한다.
