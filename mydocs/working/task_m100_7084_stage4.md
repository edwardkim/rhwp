# #7084 Stage 4 — C 통합 검증

- 시작: 2026-09-13 23:14 KST
- 상태: **2026-09-14 기존 자동 게이트 통과, 서식 경계의 결합 문자 부분 적용을 추가 재현. C 완료·PR 준비 판정 보류.**
- 계획: [구현계획 C](../plans/task_m100_7084_impl.md)
- 선행: [Stage 3](task_m100_7084_stage3.md) HWP/HWPX Studio 이모티콘 시각 판정 통과.
- 제품/테스트: `bdd63167ccad59d0560fd8a6a57e018057ba8c06`.
- 검증 checkout: `08125c255` (이후 변경은 시각 판정 문서뿐).

## 검증 순서와 재사용

별도 detached review worktree에서 integration suite를 준비한다. 원본 worktree 및 기존
`rhwp-review-7040`은 보존한다. 공유 `target/pr-review`를 사용하고 Cargo 명령은 순차 실행한다.
현재 16 logical CPU·31GiB RAM이며 전체 nextest는 기본 동시성을 사용한다.

1. release build·release lib·release-test 전체 nextest.
2. Native Skia 공식 3종.
3. fmt·diff·세 Clippy·workspace build·doc test·manifest.
4. Studio TypeScript·전체 npm test.
5. Render Diff 대응, 지원 경계 및 cold/warm 비용 측정.

실패 시 해당 명령의 원인을 확인하고 이후 게이트를 전체 통과로 간주하지 않는다.
baseline/golden 완화, 신규 sample 추가, 원격 push·PR·GitHub 댓글은 수행하지 않는다.

동일 제품 SHA의 Stage 3 Docker release WASM과 실제 Chrome 검증은 재사용한다.
WASM SHA-256 `20cc5759452bc2e7c245897a89f6019ec1f67a71a845cfece8fe2c8d9d6f97fd`.
코드가 변경되면 영향 게이트와 WASM/브라우저 증적을 새 코드에 맞춰 갱신한다.

## 결과

명령별 로그와 종료 코드를 `output/7084/stage4/`에 보존한다.

- suite 준비 성공, release build 성공 (11분 20초).
- release lib: 테스트 실행 전 E0063 컴파일 실패. 새 `ResolvedStyleSet.supplemental_metrics`
  필드를 기존 source-side 테스트 initializer 6곳에 누락했다. `paragraph_layout/issue_2439_lineseg_indent_tests.rs`
  1곳, `layout/integration_tests.rs` 1곳, `layout/tests.rs` 4곳이다.
- 이번 변경으로 발생한 테스트 구성 오류이며 사용자 시각 판정의 번복이나 기존 문서 조판 회귀 판정이 아니다.
- 같은 회차에서 6곳에 `supplemental_metrics: None`만 추가한다. 새 source-side 테스트/helper나
  기준값 변경은 없고 기존 portable 측정 의미를 유지한다. unit-tier 검사와 중단된 lib 검사를 다시 실행한다.
  동일 형태의 다른 initializer도 전체 검색해 확인한다. 수정 전 실패 로그는 보존한다.
- 보정 후 unit-tier: 4,205 tests / 298 modules 검사 통과. 원본/review 수정 파일 3개의 SHA-256 일치.
- release lib 재실행: **4,055 passed / 13 ignored / 0 failed** (rhwp 3,873 + 내부 crate 182).
  컴파일 포함 455초. `02r-release-lib.log`가 GREEN 증적이다. 전체 nextest는 이어서 실행 중이다.
- nextest 설치 버전 0.9.137은 최소 요구 0.9.91을 만족하지만 권장 0.9.140보다 낮다는 경고가 있다.
  사용하지 않는 `ci-duration-observation` JUnit `report-skipped` 키도 무시 경고가 있다.
  현재 실행은 기본 profile이며 이것을 테스트 실패나 테스트 건너뛰기와 혼동하지 않는다.
- 전체 nextest 1차: **9,768 passed / 1 failed / 51 skipped**, 실행 334.372초 (컴파일 포함 606초).
  유일 실패는 #2724 `classification_drift_is_blocked`의 새 메서드 5개 미분류다.
  `collect_canvas_metric_requests`, `register_canvas_metric_replies`, `begin_canvas_metric_session`,
  `register_canvas_metrics`, `select_canvas_metrics`는 문서 IR 아닌 렌더 세션/캐시만 갱신한다.
  같은 전체 실행에서 #7084의 원본 HWP/HWPX IR·portable SVG 보존 및 세대 교체 검사가 통과했다.
- #2724 가드의 명시된 갱신 절차에 따라 위 5개를 근거/동작 테스트와 함께 `SessionState`로 분류한다.
  raw stream 무효화를 억지로 추가하거나 검사 자체·Pending 상한·baseline 수치를 완화하지 않는다.
  가드 5종과 #7084 세션 집중 검사를 먼저 재실행한 뒤 전체 nextest를 다시 수행한다.
- 보정 후 집중: #2724 가드 **5 passed**, #7084 세션 **7 passed**.
  최초 집중 명령에는 helper가 지원하지 않는 복수 case 인자를 전달해 사용법 오류가 났다.
  제품 테스트가 실행된 실패가 아니며 단일 case 명령 두 개로 정정해 순차 통과했다.
- 전체 nextest 2차: **9,769 passed / 0 failed / 51 skipped**. 실행 337.908초,
  컴파일 포함 417초. 명령/원문은 `run-native.sh`, `03s-nextest.log`를 따른다.
  Native Skia 및 이후 게이트는 아래 후속 결과를 따른다.
- 2026-09-14 Native Skia 공식 3종 통과: lib **4,112 passed / 13 ignored**,
  누락 그림 **2 passed**, 직접 PDF **4 passed**. 각각 160초/131초/6초(컴파일 포함).
- review worktree fmt·fmt check·diff check 통과. 세 Clippy·workspace build 및 후속 게이트 진행 중.

## 후속 검증 결과

- 세 Clippy(native/WASM32/workspace all-targets), workspace build, doc test **8 passed / 3 ignored**,
  manifest check 모두 통과했다. 원문은 `10`~`15` 로그를 따른다.
- TypeScript `npx tsc --noEmit` 통과. Studio 전체 npm 첫 실행은 **1,676 passed / 2 failed / 2 skipped**.
  두 실패는 기존 `canvas-view-zoom-preview.test.ts`의 prototype fixture에 `wasm`이 없어서 새
  `prepareCanvasMetrics` 호출이 TypeError로 중단된 것이다. 제품 로직·기존 assertion을 바꾸지 않고
  fixture에 비동기 `prepareCanvasMetrics: async () => false`를 추가했다. 줌 전용 테스트에서
  메트릭 준비가 조판을 변경하지 않는 문맥을 표현한다.
- 줌 집중 및 전체 npm 재실행 통과: **1,678 passed / 0 failed / 2 skipped** (11.28초).
  `17r-zoom-test.log`, `17s-studio-test.log`. Rust 보정과 함께 이번 회차 변경은 테스트 구성뿐이다.
- headless Chrome 기본 Canvas legacy/layer Render Diff 3쪽 통과. KTX 0.01593%,
  biz_plan·tac-case 각각 0%, 기준 0.05% 유지. `18-render-diff.log`.
- 직접 PDF/호환 PDF gate는 CI의 기본 3 fixture·2% 기준·direct raster DPI 144로 통과했다.
  biz_plan **1.158954%**, tac-case **0.390370%**, kps-ai **0.676720%**.
  최초에는 검사자가 native-skia 없는 debug 바이너리를 지정하여 비교 전 3건 실패했다.
  이미 Native Skia 검증에 사용한 `target/pr-review/release-test/rhwp`로 경로만 정정했다.
  `19-initial-pdf-summary.md`와 `19r-pdf-render-diff.log`에 양쪽 증적을 보존한다.
- 브라우저 Canvas/PDF 보고 전용 비교는 **4 warnings / 0 errors**다. 기본 PDF raster DPI 72와
  Canvas 크기도 다르므로 이 결과를 시각 일치로 보고하지 않는다. 직접 PDF gate와 별도이며
  이번 변경의 회귀 여부는 이 report-only 수치만으로 확정하지 않았다. 기준값을 바꾸지 않았다.

## 실제 Chrome 비용 관측

제품은 시각 판정된 `bdd63167c`, WASM hash는 위와 같다. 별도 Cargo/전체 테스트 실행이 끝난 뒤
Windows Chrome에서 측정했다. 증적은 `measure-studio.mjs`, `studio-cost.json`이다.
폰트가 이미 로드된 상태에서 **보충 측정 cache만 비운 cold** 1회와 warm 5회다.
앱 최초 시작·폰트 다운로드 시간이나 변경 전 commit과의 대조가 아니다.

| 입력 | cold 준비 | warm 준비 중앙값 | 측정 cache 항목 | portable/활성 paint 중앙값 | 전쪽 print SVG 확보 중앙값 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 원본 HWP, 2쪽 | 1.3ms | 0.2ms | 1 | 5.5 / 6.4ms | 4.1ms |
| 원본 HWPX, 2쪽 | 1.0ms | 0.1ms | 1 | 5.5 / 5.8ms | 2.3ms |
| synam-001.hwp, 35쪽 | 119.5ms | 47.6ms | 10 | 122.4 / 121.5ms | 550.6ms |

paint는 동일 제품에서 보충 문맥 off/on 후 두 번째 쪽을 각각 5회 새 canvas에 그린 값이다.
print는 portable 전환→전쪽 SVG 문자열 확보→문맥 복원을 3회 측정했다. 프린터 출력 시간은 아니다.
세 입력 모두 warm 추가 측정 0건, cache hit는 각 요청 수와 같았고 오류가 없었다.
직렬화 cache payload는 두 원본 각각 752 bytes, synam 8,165 bytes다.
이는 **실제 JS/WASM heap 사용량이 아니다**. 실제 heap 및 변경 전 commit 대비 비용은 미계측이다.
WASM은 11,066,668 bytes이며 동일 제품의 기존 Docker 산출물을 재사용했다.

큰 문서에서 warm 준비 47.6ms, 동기 print 확보 550.6ms가 관측되므로 전체 경로의 비용이
미미하다고 일반화하지 않는다. 반복 폭 측정이 없어도 문서 요청 수집·조판 재구성 비용은 남는다.
이번 소수 반복 관측은 기초 자료이며 성능 회귀의 통계적 확정이나 전수 코퍼스 결과가 아니다.

## 지원 경계 재현과 판정

원본 두 파일은 변경하지 않았다. 실제 Studio API로 **메모리 안에서만** 다음 순서로 편집했다.
`check-run-boundary.mjs`, `run-boundary.json`에 입력 문자·요청·폭 trace를 함께 기록했다.
이는 한컴 비교나 새 손상 파일의 정답지 판정이 아니라 승인된 내부 적용 계약 검사다.

| 같은 셀의 상태 | 보충 요청 | 😀의 widthSource | 판정 |
| --- | --- | --- | --- |
| 원본 단독 😀 | U+1F600 | supplementalBackendMeasured | 기존 통과 유지 |
| 😀 뒤 VS16 추가, 동일 서식 | 없음 | heuristicHalfwidth | 결합열 전체 기존 경로 유지 |
| 동일 문자에서 VS16만 굵게 | U+FE0F와 U+1F600 각각 | supplementalBackendMeasured | **부분 적용 금지 계약 위반** |

HWP/HWPX 모두 같은 결과다. 요청 수집은 `compose_paragraph`의 **개별 run** 안에서만
grapheme을 나누고, `standalone_scalar_mask`도 전달된 문자열 안에서만 판정한다.
따라서 서식 경계가 문자 묶음의 경계인 것처럼 취급된다. 같은 논리 문자 묶음인데 VS16의
서식 변경만으로 적용 정책이 바뀌는 문제이며, 임의의 한컴 기대 좌표를 가정한 실패가 아니다.
계획 3.1의 미지원 결합열 전체 유지 규칙을 충족하지 못하므로 **기존 자동 게이트 통과만으로
C 완료를 선언하지 않는다**. 단독 이모지의 메인테이너 시각 통과를 번복하는 것은 아니다.

다음 보완은 문단의 논리 문자 묶음 경계를 서식 분할보다 먼저 보존하고, 요청 수집과 실제
폭 적용이 같은 허용 범위를 사용하도록 하는 것이다. 동일 문단/문서의 다른 위치에 단독 😀가
있어 cache에 등록돼도 결합열 안의 😀에는 새 폭이 새어 들어가지 않아야 한다.
VS16/ZWJ를 새롭게 조형하는 확장이 아니라 **미지원 전체 유지 경계의 보완**이다.
구현 전 [계획의 보완 절](../plans/task_m100_7084_impl.md#8-c-통합-검증-후-보완-요청)을 승인 요청한다.

## 남은 확인 (아직 재현 판정 아님)

- painter는 descriptor 불일치 때 오류를 반환한다. CanvasView의 일반 Canvas2D 오류 catch는
  로그/레이어 정리 후 false를 반환하므로, 이 경로의 자동 무효화·재준비 여부를 실제로 확인해야 한다.
  정상 폰트 변경 이벤트의 무효화·재준비는 Stage 3에서 별도로 통과했다.
- 인쇄의 동기 비용은 위에서 측정했다. 실제 heap 사용량 및 변경 전 제품과의 비교는 남아 있다.

위 목록은 한컴 오조판이나 신규 회귀를 확정한 것이 아니다. 관측 없이 임의의 조건을 추가하지 않는다.

## 용어

- run: 같은 문자 서식으로 나누어진 텍스트 구간. 논리 문자 묶음과 경계가 같다는 보장은 없다.
- grapheme/cluster: 사용자가 하나의 문자처럼 취급하는 묶음. 여러 Unicode 코드 포인트일 수 있다.
- VS16: Variation Selector-16, U+FE0F. 앞 문자에 이모지 표시를 요청하는 선택 문자.
- cold/warm: 여기서는 보충 측정 cache가 빈 상태/같은 문서·폰트 세대의 cache가 있는 상태.
- portable: 브라우저 전용 보충 폭을 적용하지 않는 기존 내보내기 문맥. 한컴 정답지라는 뜻이 아니다.
