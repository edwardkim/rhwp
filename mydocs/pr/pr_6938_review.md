# PR #6938 검토 기록

## 발견 사항과 최종 판정

### 최신 검증 결과 (2026-09-09, 문자열 보정 및 추가 HWP/HWPX 복원 확인)

**최종 판정: 메인터너 보정 후 수용 가능.** 원 head의 복합 차트 의미 왜곡과 추가 두 파일의
단일축 막대 폴백에 따른 보류 사유를 해제했다. 원 PR head 자체를 승인한 것은 아니다.
최신 통합 PR head CI와 작업지시자 merge 승인은 별도 조건이다.

아래 직접 검증은 `71783397d478cf383be933e03d48cff0bad84f77` 위의 메인터너 보정본에 대한
결과다. 검증 당시 미커밋이었으므로 source 및 바이너리 SHA-256으로 식별한다.
2026-09-09 작업지시자는 증적 보관, review 갱신, 최신 `upstream/devel` 리베이스와 PR 생성을
승인했다. 오늘할일 이외의 충돌이 없으면 추가 테스트를 실행하지 않도록 지시했다.
리베이스 후 결과를 리베이스 전 검증 결과와 동일한 것으로 주장하지 않는다.

#### 해결한 차단 사유와 남은 범위

통합 [PR #6951](https://github.com/edwardkim/rhwp/pull/6951)을 생성했다. 보정/증적 commit
`d3f8dff09` 뒤 `upstream/devel` `144c224193f5508a66a7dc374036995ec6de2738`로 충돌 없이
리베이스한 code candidate는 `e0dc1bc8a92ccd5d940419765d2e51bfe0a4d697`이다.
오늘할일 충돌도 없었으며 추가 테스트는 실행하지 않았다. [통합 self-review](archives/pr_6951_review.md)에
PR 채번과 리베이스 결과를 같은 PR의 문서 전용 후속 기록으로 남긴다.

- `bin_data_id`를 기존 호출부와 같은 `u32`로 맞춰 컴파일 오류를 해결했다.
- `VtPicture`의 embedded 분기에서도 null 객체 참조 `-1`을 빈 그림으로 수용했다.
- [VtString 처리](../../src/ole_chart/legacy_presentation.rs#L204)는 길이 뒤의 예약 바이트를
  반드시 NUL로 요구하지 않도록 보정했다. 추가 HWP의 `Contents[293] = 0x17`도 유효하며,
  길이와 읽기 범위 검사는 유지한다. 기존 grid 파서의 처리와 일치한다.
- 원 HWP 3쪽과 추가 `mixed_chart.hwp`/`mixed_chart.hwpx` 1쪽 모두에서 빨간 꺾은선,
  청록색 막대, 왼쪽 80~110 / 오른쪽 0~6 축, 연월 라벨과 추세 복원을 직접 확인했다.
  추가 두 SVG 모두 `data-ole-chart-presentation="legacy-mixed"` 경로를 사용했다.
- 기반 클래스 검사와 객체 역참조 값 보존 보정도 포함한다. 지원하지 않는 표시 그래프는 기존
  렌더러로 fallback하므로 모든 종류의 레거시 차트를 지원한다고 일반화하지 않는다.
- 잔여는 범례, 세로 격자, 외곽선, 글꼴 크기와 배치 차이다. 이번 판정은 복합 차트의 계열·축·값
  의미 복원 범위이며 픽셀 완전 일치 판정이 아니다. 원 문서의 19 SVG쪽 / 기준 PDF 18쪽 차이도
  기존 잔여로 남고 전체 문서 페이지 수 복원으로 표현하지 않는다.

#### 검증 대상별 결과

| 대상 | 실행 결과 | 귀속 |
| --- | --- | --- |
| 마지막 문자열 보정 전 메인터너 보정본 | fmt/check, workspace build, native/WASM/workspace-all-target Clippy, manifest, release-test build 모두 exit 0 | 마지막 문자열 보정에는 소급 적용하지 않음 |
| 같은 보정 전 후보의 전체 nextest | 9,346 passed, 0 failed, 46 skipped, 432.032초, exit 0 | 12 threads, compile 포함 799초 |
| 마지막 문자열 보정본 metadata probe | 원 문서/최소 HWP의 계열 종류·축·색상 및 각각 5개 잘린 입력 검사 통과, exit 0 | 임시 Rust probe, 영구 회귀 테스트가 아님 |
| 마지막 문자열 보정본 CLI build | `cargo build --locked --target-dir target/pr-review --bin rhwp`, exit 0, 2분 5초 | 아래 debug 바이너리 |
| 마지막 문자열 보정본 visual sweep | 추가 HWP/HWPX 1쪽 및 원 HWP 3쪽, 두 명령 exit 0 | 아래 대표 PNG를 직접 열어 확인 |
| 마지막 문자열 보정 후 전체 회귀·Clippy / 리베이스 후 재검증 | 추가 실행하지 않음 | 작업지시자의 충돌 없는 경우 재시험 생략 지시; 최신 통합 head CI는 별도 |

보정 전 전체 회귀 명령은 `cargo nextest run --locked --cargo-profile release-test --target-dir
 target/pr-review --tests --test-threads 12 --no-fail-fast`였다. 앞선 종료 코드 143 실행을 성공으로
세지 않는다. Native Skia 별도 묶음, wasm-pack, 별도 OVR 전체 검증은 완료한 것으로 기록하지 않는다.
파생 suite/manifest는 검증 전 상태로 복원했으며 커밋하지 않는다. 로그와 임시 probe는
`output/pr_6938_maintainer_20260909/`에만 보관한다.

#### 기존 한컴 PDF 재사용

| 기준 | PDF metadata | SHA-1 |
| --- | --- | --- |
| [원 문서 PDF](../../pdf/pr6938-148735526-2020.pdf) | Creator Hwp 2022 0.0.0.0, Producer Hancom PDF 1.3.0.550, PDF 1.6, A4 595x841pt, 18쪽, 635023 bytes | `121f755d99bdf7d4d221e1719f840f2448bbb9c6` |
| [추가 두 파일의 공통 기준 PDF](../../samples/issue6938/mixed_chart.pdf) | Creator Hwp 2024 13.0.0.3901, Producer Hancom PDF 1.3.0.550, PDF 1.4, A4 595x841pt, 1쪽, 29142 bytes | `cbd7f8961846fcdb889a7fae3aed2d60d6ef42cd` |

파일명의 `2020`은 engine bucket이며 Creator 2022와 구분한다. 기존 기준을 재사용했으며 이번
문자열 보정 검증에서 PDF를 다시 출력하지 않았다. 앞선 불필요한 rhwp PDF 산출은 중단했고 그
산출물은 `output/pr_6938_maintainer_20260909/unused-pdf/`에만 두며 최종 증적에 포함하지 않는다.

| 입력 / 기준 | SHA-256 |
| --- | --- |
| 원 HWP: `148735526_2012년_6월_소비자물가동향.hwp` | `c9354fd934d772bd3c50c3f2c2a019b50b824c705563063a321db0a4ee4745ff` |
| 원 문서 기준 PDF | `53b10ae46ca5edcc34cf4fa6f0b267e3d9e5573e57dc2bf7e5a6327aa0bc11fe` |
| [mixed_chart.hwp](../../samples/issue6938/mixed_chart.hwp) | `109bbe0a2ef5c1d056817e3c596c5579af08503e7b2556dd5b79f91689407d92` |
| [mixed_chart.hwpx](../../samples/issue6938/mixed_chart.hwpx) | `e09ab367c1c3c16040083fea34416692fddc5d111067490628fdb1fbe596f04c` |
| mixed_chart.pdf | `b4939db19915b270a8ad032c50268ae4dae32212ab1ba591534ab7d394f24786` |

#### 실제 시각 비교 결과

rsvg, 96 DPI, threshold 32 기준이다. 최신 후보에서 원 문서는 3쪽만 재비교했고, 보정 전 후보의
전체 1~18쪽 sweep(0/18 flagged, 평균 pixel 91.76181%, proxy 23.34109%)과 구분한다.

| 최신 후보 대상 | 페이지 | SVG/PDF 총쪽 | flagged | pixel_match | visual_accuracy_proxy_percent | 사람 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| 원 HWP | 3 | 19/18 | 0/1 | 84.18302% | 14.14239% | 복합 차트 의미 복원, 범례/격자/배치 잔여 |
| mixed_chart.hwp | 1 | 1/1 | 0/1 | 93.77858% | 20.70838% | 빨간 선 + 청록 막대 + 이중축 복원 |
| mixed_chart.hwpx | 1 | 1/1 | 0/1 | 93.77858% | 20.70838% | 빨간 선 + 청록 막대 + 이중축 복원 |

추가 두 파일은 같은 rsvg 조건의 보정 전 pixel 91.45158% / proxy 9.83795%에서 개선됐다.
flagged 0이나 pixel 수치만으로 수용하지 않고 실제 패널의 계열·축·라벨을 판독했다.
proxy는 사람의 정확도나 완전한 fidelity 비율이 아니다.

~~~bash
venv/bin/python scripts/visual_sweep.py \
  --file-target mixed-hwp samples/issue6938/mixed_chart.hwp samples/issue6938/mixed_chart.pdf \
  --file-target mixed-hwpx samples/issue6938/mixed_chart.hwpx samples/issue6938/mixed_chart.pdf \
  --page 1 --out output/pr_6938_maintainer_20260909/stringfix-sweep-mixed \
  --rhwp-bin target/pr-review/debug/rhwp --dpi 96 --svg-rasterizer rsvg
venv/bin/python scripts/visual_sweep.py --key original \
  --hwp /home/tsjang/Downloads/korea_downloads/korea_policy_downloads/148735526_2012년_6월_소비자물가동향.hwp \
  --pdf pdf/pr6938-148735526-2020.pdf --page 3 \
  --out output/pr_6938_maintainer_20260909/stringfix-sweep-original \
  --rhwp-bin target/pr-review/debug/rhwp --dpi 96 --svg-rasterizer rsvg
~~~

#### 대표 증적과 코드 식별

| 산출물 | SHA-256 |
| --- | --- |
| [원 HWP 3쪽](assets/pr_6938_maintainer_20260909/pr6938-original-p003-review.png) | `b037ccd9bb2335ed56fe43215faa01217c30e6c760005df4b61ac910beb5db08` |
| [추가 HWP 1쪽](assets/pr_6938_maintainer_20260909/pr6938-mixed-hwp-p001-review.png) | `f88ae3aaea9710cb621d584c03932b1ac1c5007ec63352c8dbedbbd5fe99034c` |
| [추가 HWPX 1쪽](assets/pr_6938_maintainer_20260909/pr6938-mixed-hwpx-p001-review.png) | `5f8231c84b516395db0e9218653abfdd7a8f33351d8a5c997264f1540cd37055` |
| 최신 legacy_presentation.rs | `a1bb0e9f53a133d16f71134712e39ec226201b2f170720e7d3a09b1328aefbf3` |
| 최신 debug/rhwp | `4a182ae6bf7a5699aa1033076d46b4caf0e7361323487820bfadf92a346661a1` |
| 문자열 보정 전 legacy_presentation.rs | `c1a158477f2a8b056a9e15a37b4a7c91d03423275a4544fe2a91c36f839e3baa` |
| 문자열 보정 전 release-test/rhwp | `78792fe8d4db57e7ba63a101853c6075ca87880d184a1c4391844206383ad1da` |

중복 raster, SVG, metric JSON, probe, 바이너리, generated suite 및 `*.log`는 커밋하지 않는다.
기존 원 head의 실패 패널은 보정 전후를 구분하는 역사 증적이며 최신 결과로 사용하지 않는다.

#### Merge 후 contributor PR comment 계획

- 정본: [Visual Sweep GitHub merge comment 절](../manual/verification/visual_sweep_guide.md#github-merge-comment).
- 원 HWP 3쪽과 추가 HWP/HWPX 1쪽의 위 실제 수치, 각각 0/1 flagged, 사람이 확인한 복합 차트
  의미 복원과 범례/격자/외곽선/페이지 수 잔여를 함께 기록한다. 마지막 문자열 보정 전 전체 회귀와
  최신 focused 검증, 리베이스 후 재시험 생략 지시를 구분한다.
- 대표 PNG는 위 세 안정 경로를 사용한다. raw URL 형식은
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6938_maintainer_20260909/<대표-PNG-파일명>`이다.
- asset이 merge commit을 통해 `devel`에 포함되고 최신 integration head CI와 실제 merge가 완료된
  뒤에만 승인 범위에 따라 `--body-file`로 게시하고 API로 실제 Markdown/이미지를 확인한다.
- 현재 작업은 commit/push/PR 생성 승인 범위이며 merge, 원 PR close, issue close, contributor comment
  게시를 수행하는 단계가 아니다. 아래 이전 실패 기록은 현재 보류 사유로 읽지 않는다.

### 이전 검증 실행 이력 (2026-09-09, 최초 컴파일 실패)

- **머지 보류 유지.** 메인터너 복합 차트 구현에 대해 실제 검증을 시작했으나 native Clippy의
  컴파일 단계에서 `E0308` 2건으로 종료했다(exit 101). 새 바이너리의 시각 검증은 하지 못했다.
- 원인: `src/ole_chart/mod.rs::render_ole_chart_svg_fragment_with_contents`의
  `bin_data_id` 인자를 `u16`으로 선언했지만 기존 렌더 API와 `ole.bin_data_id`는 `u32`이다.
  메인터너 추가 코드의 오류이며 원 contributor 구현의 실패와 구분한다.
- 이 오류는 아직 수정하지 않았다. 인자를 기존 API와 같은 `u32`로 보정한 뒤 아래 미실행
  항목을 포함해 검증을 다시 진행해야 한다. 이전 바이너리/기존 성공 로그를 재사용하지 않았다.

| 실행 항목 | 이번 결과 |
| --- | --- |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | 통과: 1,235 sources, 28 suites + 20 exceptions |
| `cargo fmt --all` | 완료 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | 실패: E0308 2건, exit 101 |
| WASM Clippy / workspace build / all-target Clippy | 앞 단계 실패로 미실행 |
| release-test 시각 검증용 build / 전체 nextest 회귀 | 앞 단계 실패로 미실행 |
| 원 HWP 및 mixed_chart HWP/HWPX의 수정 후 PDF/SVG 시각 비교 | 미실행. 새 코드 바이너리 없음 |

- 실행 로그: `pdf/pr_6938_maintainer_20260909/validation/`의 `status.tsv`,
  `prepare.log`, `fmt.log`, `fmt-check.log`, `clippy-native.log`.
  원시 로그와 파생 suite는 PR에 포함할 시각 증적이 아니다.
- 실행 환경: Linux, 16 logical CPUs, RAM 15 GiB, 시작 시 가용 메모리 약 13 GiB,
  디스크 여유 약 40 GiB. Cargo 실행은 순차로 구성했고 전체 회귀 동시성은 12로 계획했으나
  회귀 단계에는 도달하지 않았다. 검증 전 실행 중인 Cargo/Rust 프로세스는 없었다.

#### 이번 시각 검증 입력과 정답지 고정

| 입력/정답지 | SHA-256 |
| --- | --- |
| 원 HWP: `148735526_2012년_6월_소비자물가동향.hwp` | `c9354fd934d772bd3c50c3f2c2a019b50b824c705563063a321db0a4ee4745ff` |
| 원 문서 기준: `pdf/pr6938-148735526-2020.pdf` | `53b10ae46ca5edcc34cf4fa6f0b267e3d9e5573e57dc2bf7e5a6327aa0bc11fe` |
| `samples/issue6938/mixed_chart.hwp` | `109bbe0a2ef5c1d056817e3c596c5579af08503e7b2556dd5b79f91689407d92` |
| `samples/issue6938/mixed_chart.hwpx` | `e09ab367c1c3c16040083fea34416692fddc5d111067490628fdb1fbe596f04c` |
| `samples/issue6938/mixed_chart.pdf` | `b4939db19915b270a8ad032c50268ae4dae32212ab1ba591534ab7d394f24786` |

- 원 HWP의 위치는 `/home/tsjang/Downloads/korea_downloads/korea_policy_downloads/`이다.
- 원 문서 기준 PDF는 18쪽이며 파일명의 `2020`과 달리 metadata Creator는
  `Hwp 2022 0.0.0.0`이다. 파일명만으로 한컴 버전을 확정하지 않는다.
- 추가한 HWP와 HWPX는 각각 독립 실행하고, 사용자 제공 `mixed_chart.pdf` 1쪽을 같은 정답지로
  대조해야 한다. 원 문서는 기존 결함이 확인된 3쪽 차트와 전체 페이지 수를 함께 확인해야 한다.
- 아직 새 review PNG나 출력 PDF를 만들지 않았으며 시각 통과 수치도 없다.
  기존 PNG는 보정 전 증적이다. 아래 과거 분석/검증 기록을 최신 통과 결과로 해석하지 않는다.

- **판정: 머지 보류.** 한컴 기준과 다른 차트 의미를 직접 판독했고, 별도로 아래 두 코드 문제를 최소 입력으로 재현했다. 원 PR CI와 기존 전체 회귀의 성공으로 이 실패를 덮지 않는다.
- **P1: 이중축 복합 차트를 단일축 막대 차트로 표시한다.** [3쪽 대표 패널](assets/pr_6938_6940_20260909/pr6938-chart-p003-review.png)의 한컴 기준은 소비자물가지수를 왼쪽 축(약 80-110)의 빨간 꺾은선으로, 전년동월비 등락률을 오른쪽 축(0-6)의 청록색 막대로 표시한다. rhwp는 두 계열을 약 0-106의 공통 축 막대로 표시해 등락률을 바닥에 눌러 놓으며, 가로축의 연월 라벨도 일부 서수로 바뀌고 겹친다. 색상/스타일 차이가 아니라 추이와 계열 간 관계를 잘못 전달하는 출력이다. 자리표시자가 사라졌다는 사실만으로 차트 수정 완료를 인정할 수 없으며, 차트 종류·계열별 축·연월 라벨을 보존하거나 해당 형상을 명시적으로 미지원 처리해야 한다.
- **P1: 입력이 지정하는 기반 클래스를 무제한 재귀 호출한다.** [grid.rs:425](../../src/ole_chart/grid.rs#L425)의 `read_base`는 기대한 기반 클래스인지 확인하지 않고 `read_from(name)`을 호출한다. `VtDataGrid` 타입 id를 기반 타입으로 반복하면 약 400 KB 입력으로 stack overflow와 프로세스 abort(exit 134)가 발생했다. 알려진 기반 클래스 사슬을 강제하고 잘못된 순환/깊이를 오류로 반환해야 한다.
- **P2: 객체 역참조를 빈 셀로 바꿔 값을 잃는다.** [grid.rs:500](../../src/ole_chart/grid.rs#L500)는 이미 본 object id에 `Ok(None)`을 반환한다. 2x3 그리드의 두 데이터 슬롯이 같은 `VtDouble(42.0)`을 참조하면 첫 셀은 `Some(42.0)`, 두 번째는 `None`이었다(단언 실패, exit 101). `parser.rs::grid_value`는 이 값을 0으로 채우므로 잘못된 차트 값을 조용히 표시한다. 참조 대상 값을 복원하거나 지원하지 않는 참조를 명시적으로 거부해야 한다.
- PR의 `Closes #6922`는 현재 출력으로 충족되지 않는다. 위 차트 의미 불일치는 별도 비차단 잔여로 넘기지 않고 이번 PR의 수용 차단 사유로 유지한다.

## 대상과 적용

### 메인터너 보정 진행 (2026-09-09, 검증 전)

#### 복합 차트 표시 설정 복원 구현 (추가, 검증 전)

- 사용자 제공 최소 입력: `samples/issue6938/mixed_chart.hwp`,
  `samples/issue6938/mixed_chart.hwpx`. 기준 출력: `samples/issue6938/mixed_chart.pdf`
  (Hancom HWP 2024, 1페이지). HWPX에도 OOXML chart XML 대신 legacy OLE가 포함되어 있다.
- 원 문서와 최소 HWP의 `Contents` 객체 그래프를 분석했다. 두 입력 모두 `VtChart v6`,
  `VtChartPlot v4`, `VtAxis v3`, `VtSeries v2`이며, chart type 9의 계열 painter는
  첫 계열 6(선), 둘째 계열 1(세로 막대)이다. 첫 계열은 주축 80~110 / 6구간,
  둘째 계열은 보조축 0~6 / 6구간이다. 색상은 각각 빨강과 청록이다.
- `src/ole_chart/legacy_presentation.rs`에 타입 버전, 객체 참조, 배열, 축 범위,
  계열별 painter/보조축/색상/선 두께를 읽는 제한된 파서를 추가했다. 파일별 절대 위치,
  계열 이름, 값의 크기로 종류를 추측하지 않는다. 객체 깊이/개수/읽기 범위를 제한한다.
- `src/ole_chart/legacy_combo_renderer.rs`에서 각 계열을 해당 축에 배치하고 막대 위에
  선을 그린다. 희소 날짜 레이블과 수동 축 범위를 유지한다. 기존 데이터 IR은 변경하지 않고
  shape layout의 OLE 렌더 경로에서 표시 설정을 함께 전달한다.
- 이번 지원 범위는 위 버전의 수동 이중축 2D 선/세로 막대 조합이다. 미지원 버전/종류는
  기존 경로로 돌아가며, 이를 시각 복원 성공으로 판단하지 않는다. 범례/글꼴/축 안쪽 여백의
  정밀 일치는 미완료이며 새 렌더 결과를 기준 PDF와 대조해야 한다.
- 이 구현 이후 회귀테스트, Clippy, HWP/HWPX 렌더 및 PDF/PNG 비교는 아직 실행하지 않았다.
  이전 검증 성공이나 기존 review.png를 이번 코드의 증적으로 재사용하지 않는다.
  머지 보류 해제 및 `메인터너 보정 됨 수용 가능` 판정은 후속 검증 전에는 하지 않는다.
- 사용자 원본과 임시 분석 파일은 수정하지 않았으며, 새 중간 산출물을 커밋에 포함하지 않는다.

#### 아래는 표시 설정 복원 구현 이전의 보정 기록

- `grid.rs`: 기반 클래스를 `VtDataGrid -> VtMatrix -> VtCollection -> VtObject`와 `VtDouble/VtString -> VtValue -> VtObject`로 제한했다. 셀 객체도 `VtDouble`/`VtString`으로 제한해 중첩 그리드 재귀를 막도록 수정했다.
- 객체 역참조는 저장한 `GridValue`를 복원한다. 수치의 원본 offset과 문자열 record도 보존하며, 역참조를 빈 칸이나 0으로 바꾸지 않도록 수정했다.
- `parser.rs`: 일부 카테고리 레이블만 있는 축에서는 없는 레이블을 빈 문자열로 유지한다. 전체 레이블이 없는 축에만 서수 폴백을 적용한다. 범주 개수와 데이터 순서는 유지한다.
- 위 내용은 코드 수정 기록이며 통과 판정이 아니다. 이번 보정의 테스트, Clippy, 시각 재검증은 아직 실행하지 않았다. 앞 절의 9,346건 성공과 기존 PNG는 보정 전 head의 증적이다.
- **복합 차트 보정은 미완료다.** 파서는 여전히 `chart_type=Unknown`이며 계열별 차트 종류와 보조축을 IR/렌더러에 연결하지 못했다. 차트 의미 불일치의 머지 보류 사유는 해제하지 않는다.
- 원본 `BIN0001.OLE`를 메모리에서 분석했다. 내부 스트림은 `Contents`(44,401 bytes)와 `HWPChart.Info`(128 bytes)이며 `OlePres` 미리보기는 없었다. OLE 바이트 앞의 4-byte 봉투를 제외하고 CFB를 읽었다.
- `Contents`의 0-based byte offset에서 `VtChartPlot` 이름은 7,784(version 4), `VtAxis`는 8,075(version 3), `VtSeries`는 12,621(version 2)에 있었다. 이는 객체 선언의 위치이며 종류/축 연결 필드의 의미가 검증되었다는 뜻은 아니다. 이 위치를 production 코드에 하드코딩하지 않았다.
- 후속 작업은 위 객체의 직렬화 구조와 계열-축 참조를 해석하고, IR과 렌더러에 전달하는 것이다. 값 크기나 기준 PDF에 맞춘 추측으로 차트 종류/축을 결정하지 않는다. 파서 보정의 합성 경계 입력 및 sparse label 회귀를 추가하고 보정 head에서 검증해야 한다.

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#6938](https://github.com/edwardkim/rhwp/pull/6938), 작성자 `planet6897` |
| base | `devel` |
| 원 head | `a343125084db800bdd7bbb8b719ee8b2158665cd` |
| 원 PR 규모 | 6파일, +580/-251, 3커밋; 이미지 바이너리 별도 |
| 검토 기준 | `upstream/devel` `9a96eef92458112d5d7998f4b3390261c0e3c811` |
| 검토 branch | `review/planet6897-6938-6940-20260909` |
| 누적 검증 head | `71783397d478cf383be933e03d48cff0bad84f77` |
| reviewer | `jangster77` 지정 완료 |
| 마지막 원격 조회 | 2026-09-09, OPEN, head 유지, mergeable/mergeStateStatus는 UNKNOWN |

선행 가드 `893be62716f05378251e93264d6805bd67dd0d06`은 `git cherry`에서 이미 적용된 패치로 확인했다. 나머지 `00a6f375`와 `a3431250`을 순서대로 `-x` 체리픽했고 각각 `d95e51875ec8311f53bbc66192f26af41c3fd878`, `b48e57d9f503c93e0609aab2f2dc80654e719c9a`로 적용됐다. 충돌은 없었다. 이후 #6940 두 커밋을 누적했다.

- [고정 head의 변경 전체](https://github.com/edwardkim/rhwp/pull/6938/changes/a343125084db800bdd7bbb8b719ee8b2158665cd)
- [그리드 판독기 변경](https://github.com/edwardkim/rhwp/pull/6938/changes/a343125084db800bdd7bbb8b719ee8b2158665cd#diff-838f81c68eab94af50672732a176f7b280bd09473f578f601323264bc9afe218)
- [차트 데이터 좌표 변경](https://github.com/edwardkim/rhwp/pull/6938/changes/a343125084db800bdd7bbb8b719ee8b2158665cd#diff-d3e69f1651e4c16f7428f099dca9661a6983f5172524b52bfe6ba00e30651480)

## 검증 결과

- 네이티브/WASM/workspace all-targets Clippy, workspace build, fmt check, suite manifest, source-side 테스트 정책을 모두 통과했다.
- 차트/각주 관련 focused 회귀 53/53을 통과했다. 전체 회귀는 9,346/9,346 통과, 46 skipped, 409.746초였다.
- 별도 경계 입력 재현은 위 두 건 모두 실패했다. production 또는 정식 test source를 수정하지 않고, 현행 `grid.rs`를 직접 포함하는 임시 Rust 실행 파일로 확인했다.
- [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/34326860100): Build & Test와 lint 성공. Native Skia는 이 PR에서 skipped이며 실행 성공으로 쓰지 않는다.
- 원 PR 자체의 comment/review/inline comment는 없었고, 관련 이슈 #6922의 전체 comment를 확인했다. 선행 #6934 수용과 그리드/전체 차트 잔여를 구분했다.
- 명령과 공통 결과는 [통합 검토 기록](pr_6938_6940_review_impl.md)을 따른다.

## 직접 시각 증적

원본은 `/home/tsjang/Downloads/korea_downloads/korea_policy_downloads/148735526_2012년_6월_소비자물가동향.hwp`이다. 원본 SHA-256은 `c9354fd934d772bd3c50c3f2c2a019b50b824c705563063a321db0a4ee4745ff`이며, [이슈 #6922](https://github.com/edwardkim/rhwp/issues/6922)가 지목한 코퍼스 문서다. 원본은 기존 외부 코퍼스 위치를 사용했다.

- `info --json`: `format=hwp5`, `lastSavedWith.product=null`, version `6.0.5.771`.
- MCP는 저장 버전 정책에 따라 `--engine 2020`을 지정했다. job `1f79bdb2-0853-41a0-8987-ff67795cf9d0`, `succeeded` 뒤 download `success`, 635,023 bytes와 SHA-256을 확인했다.
- 기준 PDF: [pr6938-148735526-2020.pdf](../../pdf/pr6938-148735526-2020.pdf). SHA-256 `53b10ae46ca5edcc34cf4fa6f0b267e3d9e5573e57dc2bf7e5a6327aa0bc11fe`, SHA-1 `121f755d99bdf7d4d221e1719f840f2448bbb9c6`.
- PDF metadata: Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.6, 18쪽, A4 595x841 pt. 요청한 engine bucket과 실제 PDF metadata를 함께 기록하며 제품 버전을 추정하지 않는다.
- 전수 fidelity text/layout ledger는 요청 18쪽 모두 완료했다. 전체 rhwp SVG/render tree는 19쪽이었다. 기준과의 +1쪽 차이를 이번 변경이 만든 회귀로 단정하지 않았다.
- Visual Sweep은 3쪽을 직접 비교했다. flagged=0/1, `pixel_match_percent=79.96046`, `visual_accuracy_proxy_percent=8.79091`이다. 구조 후보 0건은 차트 의미/축/종류의 일치를 뜻하지 않는다.
- 대표 [3쪽 review PNG](assets/pr_6938_6940_20260909/pr6938-chart-p003-review.png)를 열어 한글·도구 라벨·범례를 판독했다. 위 이중축/차트 종류 차이는 실제 이미지에서도 확인됐다.
- PNG SHA-256: `3173269d40ae37fb1cbbed57bd364426c4a859267c37a8f53f30d84b4190c999`.

최종 임시 진단 위치는 `/tmp/rhwp-review-6938-6940/visual/chart-sweep/pr6938-chart/`이다. `compare/compare_003.png`, `overlay/overlay_003.png`, `review/review_003.png`를 생성했다. 초기 저장소 내 임시 출력은 검증 뒤 이 위치로 옮겼으며 커밋하지 않는다.

## 최소 재현

아래 하네스는 검토 시 `/tmp/rhwp-review-6938-6940/grid_probe.rs`에서 실행했다. 합성 바이트만 사용하며 원본 비공개 문서가 필요 없다. `grid.rs` 경로는 검토 checkout에 맞춘다.

```rust
#[allow(dead_code)]
#[path = "/home/tsjang/rhwp/src/ole_chart/grid.rs"]
mod grid;
fn i32v(v: &mut Vec<u8>, n: i32) { v.extend(n.to_le_bytes()); }
fn u16v(v: &mut Vec<u8>, n: u16) { v.extend(n.to_le_bytes()); }
fn ty(v: &mut Vec<u8>, id: i32, name: &[u8]) {
    i32v(v, id); u16v(v, name.len() as u16); v.extend(name); u16v(v, 1);
}
fn main() {
    let mode = std::env::args().nth(1).unwrap();
    let mut v = vec![0u8; 16];
    ty(&mut v, 2, b"VtDataGrid\0");
    if mode == "recursive-base" {
        for _ in 0..100_000 { i32v(&mut v, 2); }
        println!("{:?}", grid::scan_legacy_grid(&v));
        return;
    }
    ty(&mut v, 3, b"VtMatrix\0");
    ty(&mut v, 4, b"VtCollection\0");
    u16v(&mut v, 2);
    ty(&mut v, 5, b"VtObject\0");
    u16v(&mut v, 2); u16v(&mut v, 3);
    for _ in 0..4 { i32v(&mut v, -1); }
    i32v(&mut v, 1000);
    ty(&mut v, 6, b"VtDouble\0");
    v.extend(42.0f64.to_le_bytes()); u16v(&mut v, 65535);
    ty(&mut v, 7, b"VtValue\0"); i32v(&mut v, 5);
    i32v(&mut v, 1000);
    u16v(&mut v, 1); u16v(&mut v, 1); u16v(&mut v, 2); u16v(&mut v, 1);
    v.extend(b"VtPlot\0");
    let g = grid::scan_legacy_grid(&v).expect("valid backreference");
    assert_eq!(g.number(1, 1), Some(42.0));
    assert_eq!(g.number(1, 2), Some(42.0));
}
```

```bash
rustc --edition=2021 /tmp/rhwp-review-6938-6940/grid_probe.rs \
  --extern encoding_rs=target/pr-review/debug/deps/libencoding_rs-9982ea704fabdcdc.rlib \
  -L dependency=target/pr-review/debug/deps -o /tmp/rhwp-review-6938-6940/grid_probe
ulimit -c 0
/tmp/rhwp-review-6938-6940/grid_probe backreference  # exit 101
timeout 10 /tmp/rhwp-review-6938-6940/grid_probe recursive-base  # exit 134
```

`encoding_rs` rlib 이름은 해당 검토 빌드의 실제 파일명이다. 컴파일러/feature가 달라지면 현재 빌드의 경로를 사용한다.

## Merge 후 contributor PR comment 계획

- 현재는 머지 보류이므로 merge 완료나 승인 코멘트를 게시하지 않는다. 코드 재현 두 건과 차트 종류/이중축/연월 라벨의 불일치를 해소한 보정 head에서 회귀와 직접 시각 검증을 완료해야 한다.
- 비교 정본: [Visual Sweep GitHub merge comment 절](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- 현재 증적을 인용할 경우 3쪽, flagged=0/1, pixel 79.96046%, proxy 8.79091%와 이중축/차트 종류 잔여를 함께 적는다. 낮은 proxy는 사람 판정 정확도가 아니며 전체 차트 수용을 뜻하지 않는다.
- 대표 이미지 형식: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6938_6940_20260909/pr6938-chart-p003-review.png`.
- 향후 asset이 실제 merge commit으로 `devel`에 존재하고 게시 단계가 승인된 뒤에만 `--body-file`로 게시하고 API에서 본문/이미지를 재조회한다. #6922는 전체 차트 잔여가 해소되기 전 close하지 않는다.

## 자산 정책

새 장기 증적은 기준 PDF 1개와 대표 review PNG 1개뿐이다. 원 PR이 이미 포함한 before/after PNG는 기여 commit으로 보존한다. 임시 하네스, 원시 raster, 중복 compare/overlay, SVG, render tree, metric/MCP JSON, 로그와 파생 테스트 파일은 추가 커밋 대상이 아니다.
