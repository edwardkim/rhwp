# #8174 — S53 기준 다페이지 WASM 로딩·편집 성능 조사

2026-10-27. 제품 source: `4024cf68fcab01cfdf6e10e7bbfab640e5f09bb5`.
사용자 요청에 따른 중간 성능 조사이며 제품 코드는 변경하지 않았다.

**문서 열기의 중앙값은 약 2–5% 줄었지만, 편집 시 긴 지연은 해결되지 않았다.**
일반 입력·문단 분할/병합의 중앙값은 대체로 유지 또는 소폭 개선됐다. 다만 일부 최대값과
연속 입력의 프레임 대기는 악화돼, 편집 응답성이 일관되게 좋아졌다고 판정하지 않는다.
지연의 주요 경로는 양쪽 모두 입력 뒤 실행되는 문서 전체 Canvas 메트릭 요청 수집이다.
WASM 파일 크기는 늘었으므로 최초 접속 비용까지 개선됐다고 해석해서도 안 된다.

## 비교 범위와 실행 조건

| 항목 | 고정한 조건 |
| --- | --- |
| 변경 전 | T04 직전 S09, `1e7282ad64fae37bbf9684c1e77008e132473075`의 보존된 최적화 WASM |
| 변경 후 | S53의 보존된 최적화 WASM. `validated-commit.json`이 위 제품 source와 연결됨 |
| 빌드 | 양쪽 `release` + `wasm-opt`, codegen-units 16. 기존 producer·로그·실제 파일 hash 확인, 재빌드 없음 |
| 프런트엔드 | 양쪽 모두 동일한 현재 Studio. 별도 Vite 포트의 alias로 제품만 교체함 |
| 환경 | WSL2 Linux, i9-9900KF, 논리 CPU 16개, 약 31.3 GiB RAM, Node 24.15.0, HeadlessChrome 155.0.8059.79 |
| 브라우저 | 1280×900, DPR 1, 동일 실행 인자/GPU 비활성화. 매 trial 새 BrowserContext |
| 글꼴 | 외부 웹폰트 비활성화, 실제 외부 resource 요청 0건. 두 제품의 FontFace 수는 문서별 12/14/20개로 같음 |
| 반복 | 예비 실행 제외, 문서별·제품별 3회. 순서는 회차별 candidate/base와 base/candidate를 교대 |
| 실행 격리 | 빌드·다른 벤치마크와 겹치지 않게 순차 실행. 사용자의 기존 서비스는 중지하지 않음 |

브라우저가 **실제 초기화 때 받은 WASM 응답**을 CDP로 읽어 SHA-256을 검증했다.
root `pkg/`와 Studio 배포 파일은 덮어쓰지 않았다. OS 파일 캐시를 비우는 실험이나
실제 배포 CDN/Windows GUI 브라우저의 지연 측정은 아니다. 기존 프로세스와 WSL 스케줄링의
영향이 남으며, 3회 비교의 작은 차이를 일반적인 성능 보장으로 확대하지 않는다.

이 비교는 **S09 → S53의 누적 영향**이다. S53 한 절편만의 효과나 최신 devel 대비 최종
PR 전체의 성능 판정과 구분한다. PDF 디코딩·인식/import 자체는 이번 시간 측정 대상이 아니다.

| 입력 | 쪽수 | 편집 위치 |
| --- | ---: | --- |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp` | 115 | 기존 #3137 대형 셀 표준 위치 |
| 같은 이름의 `.hwpx` | 115 | 같은 셀·문단·글자 위치 |
| `mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp` | 415 | 1·208·415쪽의 실제 hit-test 셀 위치 |

## 문서 열기

단위는 ms이며 **3회 중앙값**이다. `loadDocument`는 파싱·초기 Core 구성/조판을 포함하는
동기 bridge 호출이다. UI 완료는 `open-document-bytes` 요청 발행부터 완료 이벤트까지로,
글꼴 준비와 초기 화면 구성도 포함한다. 파일 전송·base64 해제와 WASM 초기화는 앞에서 끝낸다.

| 입력 | 동기 loadDocument 전 → 후 | UI 열기 완료 전 → 후 | UI 중앙값 변화 |
| --- | ---: | ---: | ---: |
| 115쪽 HWP | 1,345 → 1,271 | 4,978 → 4,777 | −4.0% |
| 115쪽 HWPX | 1,197 → 1,161 | 7,978 → 7,808 | −2.1% |
| 415쪽 HWP | 3,992 → 3,794 | 15,741 → 15,001 | −4.7% |

UI 완료의 전/후 최솟값–최댓값은 각각 HWP 4,893–5,080 / 4,740–5,061,
HWPX 7,941–7,986 / 7,764–7,812, 415쪽 15,588–15,962 / 14,999–16,057이다.
HWP와 415쪽은 분포가 겹친다. 회차별로 UI 완료가 느려진 경우도 있으므로 매번 빨라진다는
판정은 하지 않는다. 동기 loadDocument는 세 입력 모두 3회 paired 비교에서 줄었다.

별도로 잰 빈 Studio의 앱 준비 시간은 양쪽 각 9회 중앙값 **644 → 716ms**였다.
범위는 609–966 / 624–797ms로 겹친다. 여기에는 dev 서버의 JS 로드·WASM 초기화 등이
섞여 있으므로 증가분 72ms를 WASM 컴파일 비용으로 단정하지 않는다. 위 문서 열기 수치와
최초 앱 접속 시간은 다른 지표다.

## 일반 입력과 화면 반영

115쪽에서는 회차마다 영문 6자와 한글 IME 3자(`ㅎ → 하 → 한` 3단계)를 입력했다.
415쪽은 세 위치에서 영문을 각각 3자 입력했다. 요청 간격은 150ms이며 **전체 234개 입력
이벤트**다. 입력 처리의 동기 시간과 rAF 대기를 별도로 기록했다.

| 입력/위치 | 동기 입력 중앙값 전 → 후 | 동기 입력 p95 전 → 후 | 첫 rAF 중앙값 전 → 후 |
| --- | ---: | ---: | ---: |
| 115쪽 HWP 영문 | 2.2 → 2.2 | 8.5 → 18.0 | 402 → 395 |
| 115쪽 HWP IME 단계 | 13.6 → 13.0 | 16.2 → 14.8 | 414 → 403 |
| 115쪽 HWPX 영문 | 3.4 → 3.4 | 7.7 → 8.5 | 400 → 397 |
| 115쪽 HWPX IME 단계 | 14.6 → 14.1 | 17.6 → 16.5 | 412 → 400 |
| 415쪽 / 1쪽 | 6.7 → 4.9 | 18.4 → 10.3 | 692 → 661 |
| 415쪽 / 208쪽 | 14.9 → 15.0 | 16.2 → 24.1 | 684 → 711 |
| 415쪽 / 415쪽 | 5.1 → 4.9 | 7.8 → 8.2 | 683 → 656 |

중앙값이 작아도 첫 프레임까지 수백 ms가 걸린다. 동기 입력 2ms를 체감 지연 2ms로
해석하면 안 된다. 영문 115쪽은 제품별 18개, IME는 27개, 415쪽 각 위치는 9개 이벤트로
표본이 작다. 첫 입력의 콜드 실행을 포함하며, 18개 표본의 nearest-rank p95는 최대값이다.
HWP 후보 첫 회의 첫 입력 18ms처럼 실제 관측한 꼬리 악화를 중앙값으로 숨기지 않는다.

두 번째 rAF는 다음 입력과 간섭한다. 115쪽은 영문 817 → 790 / 812 → 803ms,
IME 838 → 820 / 835 → 814ms였지만, 415쪽은 위치별로 다음처럼 크게 흔들렸다.

| 415쪽 위치 | 연속 입력 두 번째 rAF 중앙값 전 → 후 | 입력을 겹치지 않게 한 대조 실험 전 → 후 |
| --- | ---: | ---: |
| 1쪽 | 710 → 1,324 | 696 → 661 |
| 208쪽 | 785 → 1,335 | 708 → 678 |
| 415쪽 | 1,365 → 684 | 681 → 676 |

대조 실험은 별도 제품별 1회·각 위치 3자이며, **직전 입력의 두 rAF가 끝난 뒤 다음 입력**을
보냈다. 같은 경로에서 추가 입력 간섭을 제거하면 약 0.66–0.71초 범위다. 따라서 연속 입력의
두 번째 rAF 중앙값만으로 WASM 조판이 두 배 느려졌다고 해석할 수 없다. 반대로 연속 입력의
대기가 악화된 관측도 남기며, 타이핑 전체가 개선됐다는 근거로 사용하지 않는다.
rAF는 프레임 기회 대기 지표로 실제 디스플레이의 픽셀 표시 완료 시각은 아니다.

각 burst 뒤 잔여 지연 페이지네이션을 명시적으로 마무리했다. 115쪽의 완료 barrier는 약
0.99–1.06초 → 0.95–1.00초로 여전히 크다. 이는 기본 idle 완료 시간을 잰 수치가 아니며,
입력 동기 시간에 합산하지 않았다. 원시 JSON에는 호출·render·long task·잔여 pending을 보존했다.

## 조판을 다시 실행하는 편집

115쪽 HWP에서 같은 문단 끝의 **Enter → Backspace**를 별도 제품별 3회 실행했다.
분할로 문단 수가 하나 늘고 커서가 새 문단으로 이동하는지, 병합 뒤 문단 수·원래 텍스트·커서가
복원되는지 검사했다. 양쪽의 관측 쪽수는 **115 → 116 → 115**로 같았다. 116쪽 자체를
한컴의 정답 쪽수로 확정한 검사가 아니라, 같은 입력 전후의 동작 보존 확인이다.

| 동작 | 동기 처리 중앙값 전 → 후 | 두 번째 rAF까지 전 → 후 |
| --- | ---: | ---: |
| Enter | 1,528 → 1,481ms | 1,985 → 1,929ms |
| Backspace 병합 | 1,542 → 1,460ms | 2,020 → 1,929ms |

이 경로 역시 소폭 줄었으나 **약 1.5초의 동기 정지와 약 2초의 화면 대기**가 남는다.
일반 문자 입력의 빠른 경로 통과로 문단 분할·전체 재조판 성능을 대신하지 않았다.

## 병목의 실행 근거와 공통 코드 경로

115쪽 문서에서 제품별 6자 입력을 별도 CPU profile로 추적했다.
`collectCanvasMetricRequests` 아래 표본 시간이 baseline **2,324 / 2,552ms = 91.1%**,
후보 **2,279 / 2,508ms = 90.9%**였다. 메트릭 수집이 긴 지연의 주된 경로라는 실행 근거다.
샘플링 profiler의 가중 시간을 실제 allocator 계상량이나 정확한 함수 stopwatch와 혼동하지 않는다.

실제 호출은 다음과 같다.

1. [편집 쪽 무효화](../../rhwp-studio/src/view/canvas-view.ts#L436)가
   `selectMutationRevision`에서 `prepareCanvasMetrics`를 기다린다.
2. [bridge](../../rhwp-studio/src/core/wasm-bridge.ts#L357)가 현재 문서·글꼴 generation으로
   [CanvasMetricSession.prepare](../../rhwp-studio/src/core/canvas-metric-session.ts#L20)를 호출한다.
3. `collectCanvasMetricRequests`의
   [Rust 수집기](../../src/document_core/canvas_metric_requests.rs#L189)가 **모든 section의 문단과
   master page**, 중첩 표의 셀·개체 문단까지 순회하고 조판 결과에서 글꼴/글자 요청을 수집한다.
4. 준비 뒤에야 쪽 갱신/paint가 이어진다. 실제 키 입력·focused patch paint가 빨라도 이 앞의
   전역 수집 비용은 남는다.

수집기와 `canvas-metric-session.ts`는 S09와 S53 사이 diff가 없다. 프로파일에서도 같은
상위 경로가 지배적이다. 따라서 이번 예산 연결 때문에 새로 생긴 전역 순회로 분류하지 않는다.
그 하위 composer는 이번 작업에서 바뀌었으므로 모든 하위 계산 비용까지 같다고 주장하지 않는다.

일반 HWP/HWPX의 [Core 구성](../../src/document_core/commands/document.rs#L220)은 같은 공통
구현의 `Unbounded`를 사용한다. [Unbounded Storage](../../src/storage.rs#L183)는
`Infallible`과 일반 collect/clone/reserve를 사용하고 요청 바이트 계상은 no-op이다.
PDF import의 `Allocation` 전달 경로와 구분된다. 공통 코드가 달라져 간접 영향은 가능하지만,
일반 HWP 편집이 매번 PDF 요청 예산을 소진하는 구조는 아니다.

## 메모리·배포 크기와 판단의 한계

115쪽 경계 편집 실험의 WASM linear memory는 제품별 3회 모두 **14,221,312 bytes**로,
로드 직후와 편집 후가 같았다. 이것은 해당 시점의 linear memory capacity이며 최대 live heap,
renderer RSS·JS/Canvas 메모리 또는 415쪽 문서 전체 메모리의 회귀 판정은 아니다.
추가 JS heap 관측값은 원시 자료에 남겼고 WASM/RSS의 대용으로 사용하지 않았다.

| 제품 크기 | S09 | S53 | 변화 |
| --- | ---: | ---: | ---: |
| WASM 원본 | 23,497,145 bytes | 25,208,798 bytes | +7.3% |
| 동일 gzip level 9 | 8,864,547 bytes | 9,431,192 bytes | +6.4% |

gzip은 비교를 위한 로컬 압축 결과이며 실제 CDN 전송량 측정이 아니다. 최초 다운로드와
컴파일 측면의 비용 증가는 별도로 고려해야 한다. 앱 초기화 후 문서 열기의 소폭 개선으로
이 비용을 상쇄했다고 단정하지 않는다.

**판정:** 이번 누적 수정이 다페이지 문서 열기·일반 편집을 전반적으로 크게 악화시켰다는
근거는 발견하지 못했다. 중앙값은 대체로 유지 또는 소폭 개선됐고, 일부 지연 꼬리·프레임
대기의 악화도 관측했다. 기존의 큰 편집 지연을 해결한 작업으로도 볼 수 없다.

후속 성능 개선의 우선 후보는 **문서 전체 메트릭 수집을 변경 문단/셀과 새 글자·글꼴 요청으로
제한하는 증분화·재사용**이다. 글꼴·문단·개체·저장 줄·편집 revision의 무효화 계약과 조판 결과를
보존하며 같은 공통 결과를 소비해야 한다. 메트릭을 끄거나 자원 한도를 높이는 방식으로
지연을 숨기는 제안이 아니다. 다음은 전체 동기 재조판과 JS/Canvas 포함 메모리의 별도 측정이다.
이번에는 원인 후보와 실행 증거를 기록했고 성능 최적화 구현은 하지 않았다.

## 재현 자료와 검증 상태

당시 원시 자료는 ignored `output/pr-review/issue8174/wasm-large-doc-perf-20261027/`에 생성했으며 종료 정리 대상이다. 아래 집계와 당시 제품·조건을 보존하고 현재 후보의 성능 결과로 바꾸어 설명하지 않는다.

- `environment.json`: 제품·입력 hash와 host 정보. producer hash와 실제 브라우저 수신 hash 일치 확인.
- `benchmark-measured-source.mjs`·`benchmark.mjs`·`probe-module.mjs`·`server.mjs`: 기존 #3137
  진단 코드를 재사용한 조사 harness와 고정 제품 alias. 뒤의 isolated 옵션 추가는 별도 보존했다.
- `measured-results.json`·`analysis.json`·`summary.tsv`: **18회 / 234개 이벤트**, 회차별 원시 trace·분포.
- `boundary-results.json`: **6회**, Enter/Backspace의 실제 변경·복원과 메모리.
- `isolated-results.json`: **2회 / 18개 이벤트**, 다음 입력을 기다리는 대조 실험.
- `cpu-*-typing.cpuprofile`·`cpu-summary.json`: 제품별 CPU 표본과 수집 경로 비율.
- `correctness-audit.json`·`evidence-manifest.json`: 텍스트·커서·쪽수·완료 상태와 증적 hash.

실행 명령은 저장소 루트 기준이다. 보존된 S09/S53 제품과 명시한 Chrome 경로가 필요하다.

```bash
node output/pr-review/issue8174/wasm-large-doc-perf-20261027/server.mjs
BENCH_START=1 BENCH_RUNS=3 node output/pr-review/issue8174/wasm-large-doc-perf-20261027/benchmark-measured-source.mjs
node output/pr-review/issue8174/wasm-large-doc-perf-20261027/boundary-profile.mjs
BENCH_START=1 BENCH_RUNS=1 BENCH_FIXTURE=curriculum BENCH_ISOLATED=1 BENCH_PREFIX=isolated node output/pr-review/issue8174/wasm-large-doc-perf-20261027/benchmark.mjs
venv/bin/python output/pr-review/issue8174/wasm-large-doc-perf-20261027/analyze.py
```

서버만 별도 터미널에서 유지하고 나머지 명령은 **순차 실행**한다. 조사 종료 후 전용 서버와
브라우저를 종료했다. 기존 사용자 서버·소스·제품·테스트 기대값은 보존했다.
실제 수정 텍스트·커서, 분할/병합의 복원, 로드 및 완료 후 쪽수와 pending 해제를 확인했다.
이 시간 검사를 독립 한컴 출력과의 새 시각 일치 판정으로 승격하지 않는다.
문서 2개의 상대 링크와 diff 공백 검사를 통과했다. 장기 문서 메타데이터의 기존 오류
20건은 변경 전후 동일하며 신규 오류는 없다.

T04/T08 전체 완료 판정은 그대로 남으며 최종 제품 source에서는 성능도 다시 비교해야 한다.
이번 문서 변경으로 WASM/CI를 다시 실행할 필요는 없다. 원격 게시·push·PR 생성은 하지 않았다.
