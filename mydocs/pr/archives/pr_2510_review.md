# PR #2510 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2510](https://github.com/edwardkim/rhwp/pull/2510) |
| 관련 이슈 | [#2430](https://github.com/edwardkim/rhwp/issues/2430) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| 검토 범위 | 한양·휴먼 계열 ASCII 실측 메트릭, 별칭 해소, native/WASM 회귀 계약 |
| 원 PR 규모 | +2,365/-1,106, 24 files |
| 원 PR merge | 2026-07-21, [`215a174`](https://github.com/edwardkim/rhwp/commit/215a17430a558ebd93f579cf6df3c343a48b1b4e) |
| 최종 판단 | 수용 및 merge 완료. [#2430](https://github.com/edwardkim/rhwp/issues/2430)는 잔여 문서군과 증적 정정 추적을 위해 open 유지 |

## 변경과 판단

이 PR은 한양신명조·한양중고딕·한양견명조·한양견고딕·휴먼명조를 HY 계열로 오귀속하던
별칭 해소를 바로잡고, 다섯 face의 ASCII 메트릭 테이블을 추가했다. 대표 문서
`21868765_별표2_보건소_분장사무`는 한컴 기준 4쪽에 맞춰 rhwp가 7쪽에서 4쪽으로 회복했다.

Studio/WASM 경로는 등록된 face에 대해 Canvas `measureText`보다 임베디드 메트릭을 우선한다.
따라서 새 테이블은 native뿐 아니라 Studio의 줄바꿈·캐럿·선택 좌표에도 적용된다. 표시 글리프는
로컬 글꼴 우선, 부재 시 HY 웹 대체를 쓰는 hybrid 정책이다. 레이아웃 정합에는 수용 가능하지만,
원 HFT 글리프까지 동일하게 그리는 문제는 글꼴 배포와 CanvasKit 정책의 별도 추적 축이다.

## 검증

- `python3 tools/task2430/gen_metrics.py --ladder-dir tools/task2430/measured --verify`: 다섯 face 모두
  `95/95 exact match - OK`.
- `issue_2430_hft_faces_ascii_embedded_coverage`: 다섯 face의 ASCII 전 범위가 임베디드 메트릭으로
  해소됨을 확인.
- `issue_2214_page_local_repaint` native focused binary: HWP/HWPX cold/warm tree 및 셀 flow 3건 통과.
- `wasm-pack build --target web --out-dir pkg`: 이 후속 기록 worktree에서 성공.
- 임시 Vite `127.0.0.1:7701`에서 실제 Chrome headless E2E를 각각 실행했다. 사용자 7700 서버는 사용하거나
  변경하지 않았다.
  - HWP: `GREEN`, `flush=1`, boundary `1196.00ms`, stable P95 `62.00ms`.
  - HWPX: `GREEN`, `flush=1`, boundary `1171.10ms`, stable P95 `59.50ms`.
- 원 PR 최신 head [`978620a`](https://github.com/edwardkim/rhwp/commit/978620a317b4003775e3a85cde6ae49a36744c3e)의
  GitHub Actions CI, CodeQL, Render Diff, Native Skia, frontend package gates 및 8개 default-feature shard가
  모두 성공했다. CI의 `WASM Build` job은 경로 조건으로 skipped였고, 위 로컬 WASM build로 별도 확인했다.

## 시각 증적

- 임시 산출물: `output/poc/task2214/stage4/focused/{hwp,hwpx}/run-1/`.
- 보존 asset: `mydocs/pr/assets/pr_2510/review_001.png`.
- `review_001.png`는 좌상단부터 시계 방향으로 HWP 55 입력 직후, HWP 56 입력 뒤 두 RAF,
  HWPX 56 입력 뒤 두 RAF, HWPX 55 입력 직후 프레임을 배치했다. 두 포맷 모두 경계 입력 뒤 렌더가
  안정화되어 E2E의 GREEN 결과와 일치한다.
- `review_002_hwp_diff.png`, `review_003_hwpx_diff.png`는 각 55/56 경계 프레임의 diff다.

## 비차단 후속 보완

`tools/task2430/EVIDENCE.md`의 두 증적 표현은 보정이 필요하다.

1. 문서에 적힌 ladder TSV SHA-256은 현재 Git checkout의 LF 바이트가 아니라 CRLF 변환 바이트와 일치한다.
   체크섬의 canonical 줄바꿈 규칙을 명시하거나, LF checkout 기준 SHA-256으로 다시 기록해야 한다.
2. `measured/preflight_report.tsv`에는 현재 휴먼명조 한 행만 있는데 EVIDENCE는 다섯 face preflight의
   커밋 증적이라고 설명한다. 다섯 face를 합친 report를 저장하거나 face별 report로 분리해야 한다.

두 항목은 실행 결과와 정적 테이블의 `--verify` 정합을 바꾸지 않으므로 이번 merge의 차단 사유는 아니다.
다만 장기 재현성의 증적 설명이므로 [#2430](https://github.com/edwardkim/rhwp/issues/2430) 후속에서 정정한다.

## 후속 상태

[#2430](https://github.com/edwardkim/rhwp/issues/2430)의 11건 코호트 중 대표 `21868765`는 해소됐다.
나머지 10건은 다른 폰트 또는 재래핑 경로 축으로 분류되어 있으며, 이 PR merge만으로 이슈를 close하지 않는다.
