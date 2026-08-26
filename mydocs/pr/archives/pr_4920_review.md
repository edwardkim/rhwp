# PR #4920 검토 - 출력 backend 공통 trait 계층

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4920](https://github.com/edwardkim/rhwp/pull/4920) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합 및 메인터너 출력 계약 보정 |
| base / head | `devel` / `feat/render-backend-trait` |
| source candidate | `cccd298b2f11e753cb4a475eec1d878b622a3d49` |
| 통합 commit | `302c2986168cbffbfd47d8796667cdf2c684a7f4` |
| 규모 | 8 files, +1,579 / -0 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- `RenderBackend`, capability, page lifecycle, replay driver 및 SVG/trace/null reference backend를 신설한다.
- 원 변경의 `SvgBackend`는 clip을 실제로 적용하지 않고 다중 page SVG도 유효하게 결합할 수 없으면서
  해당 capability를 선언했다. #4931의 메인터너 보정 `210b3ee37`은 capability 광고를 실제 동작과 맞추고
  두 번째 쪽 시작을 명시 오류로 거부했다.
- 보정은 contributor history를 재작성하지 않고 별도 commit으로 추가했으며, 기존 renderer/layout의 페이지
  geometry와 fixture에는 영향을 주지 않는다.

## 검증

- source candidate의 Build & Test와 기본 feature 세 shard·slow shard는 성공했다.
- CodeQL 분석 job은 성공했고 roll-up은 `NEUTRAL`이며, Native Skia와 frontend/WASM job은 변경 영향에 따라
  skipped였다.
- 보정 뒤 capability와 multi-page 거부 Rust 계약 테스트를 추가하고, #4931 누적 tree 전체 `release-test`
  integration 회귀를 종료 코드 `0`으로 완료했다.

## 위험과 권고

multi-page SVG 지원은 조용한 잘못된 root 연결 대신 명시 오류가 된다. 실제 multi-page SVG가 필요해지면
단일 문서 결합 규격을 별도 설계해야 한다. 보정을 포함한 #4931 통합 merge를 권고하며, 원 PR은 merge 뒤
supersede 처리한다.
