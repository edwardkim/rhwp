# #7084 Stage 5 — 서식 경계를 넘는 결합열 보호

- 2026-09-14 구현계획 8절 승인 후 착수.
- 기준: `27a4c1a4d`, 작업 브랜치 `task_m100_7084`.
- 선행: [Stage 4](task_m100_7084_stage4.md).
- 범위: 논리 문단의 문자 묶음 판정을 요청 수집·폭 계산·paint에 전달한다.
  VS16/ZWJ 전체 조형 지원, 폰트 배포, baseline 완화는 하지 않는다.
- 원본 HWP/HWPX의 메인테이너 시각 통과는 유지하며 다른 위치의 단독 이모지가 캐시에
  등록되어도 결합열 내부에 적용되지 않아야 한다. 합성/메모리 편집 계약과 한컴 증거는 구분한다.
- 순서: 재현 계약 RED → 공통 경계 보완 → 집중 GREEN → Docker WASM/실제 브라우저 → 영향 게이트.
- 원격 push·PR·댓글은 수행하지 않는다.

## 현재 판정

**서식 경계를 넘는 결합열 보호 보완과 영향 검증은 완료했다. 제품/테스트 커밋은
`281fb2826`이다. #7084의 C 최종 완료는 아직 선언하지 않는다.**
통제한 descriptor 불일치에서 화면 자동 재준비가 호출되지 않음을 확인했다.
오류 거부와 명시적 재준비 복구는 통과했으며, 정상 폰트 변경 복구와 구분한다.
아래 실패 기록은 보완 과정의 이력이고, 최종 검증 결과는 마지막 절을 따른다.

## 진행 기록

문자 묶음의 전체 경계를 잃는 지점은 서식·언어별 run 분할이다. 수집에서만 누락시키면 다른
위치의 동일 style/문자가 cache에 있을 때 소비 경로가 다시 적용할 수 있어 충분하지 않다.
요청과 소비가 같은 보존된 경계 정보를 사용해야 한다.

## 구현과 집중 검증

- `supplemental_clusters`가 논리 문단의 grapheme(사용자가 한 문자로 인식하는 묶음)를
  scalar 주소로 투영한다. 서식/언어 run은 보충 측정 금지 여부를 보존한다.
- 요청 수집과 run 기반 폭/positions/paint 스타일, 문자 단위 토큰화 및 인라인 표 옆
  텍스트 흐름이 같은 판정을 소비한다. 단독 문자의 캐시 등록으로 금지가 해제되지 않는다.
- 원본 문서의 메모리 편집으로 VS16·ZWJ 서식 경계와 단독 이모지가 함께 있는 경우를 검사한다.
  이는 독립 한컴 출력 판정이 아니라 보충 측정의 적용 범위에 대한 계약 테스트다.
- RED: 기준 `27a4c1a4d`에서 서식 경계 요청 테스트 실패(7 PASS / 1 FAIL).
- GREEN: 첫 보완 후보의 포맷 소스에서 세션 **9 PASS**, 보충 메트릭 **21 PASS**.
  로그: `output/7084/stage5/01r-session.log`, `02-metrics.log`.
- 초기 기계적 치환에서 ComposedTextRun이 아닌 두 타입에 메서드를 적용한 컴파일 오류를
  복원했고 `cargo check --lib` 재검사에 통과했다. 원인 로그를 보존했다.
- 집중 실행 첫 시도는 포맷 후 파생 suite 재준비 누락으로 묶음 번호가 달라져 **0 tests**로
  실패했다. 제품 회귀가 아니며 통과로 계산하지 않는다. review worktree에서 재준비·manifest
  검사를 수행한 뒤 위 9+21건을 실제 실행했다. 파생 파일은 PR 대상에 넣지 않는다.
- Studio TypeScript 및 npm 전체: **1,678 PASS / 0 FAIL / 2 skipped**.
- 전체 Rust·Native Skia·lint 및 새 Docker WASM/브라우저 검증은 진행 중이다.

### 전체 lib에서 발견한 표시 투영 회귀와 정정

- `03-lib.log`: rhwp lib **3,872 PASS / 1 FAIL / 13 ignored**. 기존
  `legacy_hancom_product_names_use_display_projection_only`의 원문 run 보존 계약이 실패했다.
  새 보호용 run 분할을 기존 옛한글 표시 투영보다 먼저 수행한 것이 원인이다.
- 기대값·옛한글 변환 규칙은 바꾸지 않았다. 기존 표시 투영을 먼저 완성하고, 원문 문자열을
  보존하는 표시 run은 분할하지 않은 채 보충 메트릭 금지로 처리한다.
- `inserted_control_text`로 컨트롤 payload에서 삽입된 글자를 명시한다. 이 글자는 문단 원문의
  scalar 주소를 소비하지 않는다. 반대로 PUA 한 글자를 글자겹침으로 표시한 경우는 원문
  한 글자 주소를 소비한다. 모양이나 특정 Unicode 값으로 둘을 추정하지 않는다.
- 컨트롤 삽입과 원문 PUA 표시 뒤의 VS16 묶음·단독 이모지가 서로 바뀌지 않는 계약을
  기존 integration source에 추가했다. 수정 후 lib 컴파일 검사 통과, 집중/전체 재검증 중이다.
- 첫 Docker WASM 빌드는 성공했지만 위 정정 전 중간 소스다. 최종 성공 증적으로 사용하지 않는다.
- 새 컨트롤 계약의 첫 실행은 fixture의 PUA 선택 오류로 실패했다. `F02B1`은 실제
  `pua_enclosed_border_type` 경로가 아니므로, 그 경로에 정의된 `F02CE`로 fixture만
  정정했다. 이 실패를 제품 회귀나 한컴 출력 오류로 분류하지 않는다. `01s-session.log` 보존.

### 정정 후 재검증

- 세션 **10 PASS**, 보충 메트릭 **21 PASS** (`01t-session.log`, `02r-metrics.log`).
- 전체 lib **4,055 PASS / 0 FAIL / 13 ignored** (`03r-lib.log`, 컴파일 포함 174초).
  옛한글 원문 run 보존 계약을 포함한다. 앞선 실패 기록과 구분하며 기대값은 변경하지 않았다.
- 전체 integration·Native Skia·lint·최종 Docker WASM 검증은 같은 소스로 계속 진행 중이다.

### 전체 integration에서 발견한 비활성 경로 변경과 정정

- `04-nextest.log`: **9,766 PASS / 6 FAIL / 51 skipped**, 실행 466.279초.
  6건 모두 #4969의 `shaping_emitted_run_mapping`·`shaping_atomic_activation` 계약이다.
  보충 측정을 사용하지 않는 경로에서도 새 보호 처리가 `ᄒᆞᆫ말` 원문 run을 나눠,
  기존 공통 shaping 대안 또는 nominal fallback의 원문 보존을 깨뜨렸다.
  실제 실행 회귀이며, 한컴 시각 불일치를 별도로 관측했다는 뜻은 아니다.
- 보충 측정이 없는 공개 compose 경로는 기존 구성을 그대로 반환한다. 측정 요청 수집에는
  논리 경계 판정을 사용하고, 실제 조판에는 현재 snapshot을 전달하여 등록된 보충 측정이
  소비되는 run만 보호용으로 분할한다. 다른 위치의 이모지 등록이 옛한글 등 무관한 run을
  바꾸지 않는 계약을 추가했다. 기존 #4969 기대값·baseline은 변경하지 않았다.
- 컨트롤 삽입 출처 및 보호 플래그로 인한 재분할도 활성 측정 경로에 한정한다.
  저장 줄 높이·각주 조각 수 등 메트릭을 소비하지 않는 geometry 조회는 기존 compose를 유지한다.
- 두 차례 성공한 Docker 빌드는 위 정정 전 중간 소스이므로 최종 증거에서 제외한다.
  집중 계약과 전체 integration 통과 후 최종 소스를 다시 빌드한다.
- 현재 정정 소스의 lib 컴파일 검사 통과(`check-context3.log`); 집중·전체 게이트 재실행 중.

### 정정 소스 집중 통과

- 세션 **11 PASS**, 보충 메트릭 **21 PASS** (`01w-session.log`, `02s-metrics.log`).
  무관한 nominal run 보존과 cache가 있는 결합열 내부의 보충 폭 차단을 함께 검증했다.
- 앞서 실패한 #4969의 두 integration source를 동일 기대값으로 재실행해 모두 통과했다
  (`02s-shaping-mapping.log`, `02s-shaping-atomic.log`). 전체 검사는 이어서 진행한다.
- 추가 검증 코드의 타입 오류(`01u-session.log`, `u32` 페이지 번호의 불필요한 변환)와
  관측 단위 오류(`01v-session.log`)도 정정했다. 후자는 보호 분할 전후 run 배열 길이를
  비교한 잘못된 가정이다. 같은 원문 문자 주소의 이모지 진행폭이 보존되는지 비교하며,
  첫 이모지의 기존 폭 source와 다음 단독 이모지의 보충 source를 각각 검사한다.
  기존 테스트/baseline을 바꾼 것이 아니라 이번에 추가한 계약의 비교 단위를 바로잡았다.

### 전체 회귀 재통과

- 전체 lib **4,055 PASS / 0 FAIL / 13 ignored** (`03t-lib.log`, 컴파일 포함 163초).
- 전체 integration **9,773 PASS / 0 FAIL / 51 skipped** (`04r-nextest.log`).
  실행 362.646초, 컴파일 포함 451초. 앞선 6건을 포함하며 기대값/래칫/샘플은 변경하지 않았다.
- lib 재실행 직전 기존 각주 단위 테스트 한 곳의 잘못된 context 함수 치환을 복원했다
  (`03s-lib.log`). 제품 함수 변경이나 테스트 기대값 변경은 아니다.
- Root와 review worktree의 변경 Rust source/test bytes가 일치함을 확인했다.
  전체 회귀 통과 후 `docker-wasm-verified.log`의 최종 빌드를 시작했다.
  Docker named target과 native review target은 별도이며 Native Skia/lint와 병행한다.

## 최종 소스 검증 결과

제품/테스트 `281fb2826`의 source bytes를 root와 review worktree에서 대조했다.
검증 후 제품 소스를 변경하지 않았다. review 전용 경로는
`/home/edward/mygithub/rhwp-rust-review-7084-c`, native target은
`/home/edward/mygithub/rhwp/target/pr-review`다. 기존 다른 worktree와 공유 cache는 보존했다.

| 검증 | 결과 | `output/7084/stage5/` 증적 |
| --- | --- | --- |
| 세션·보충 메트릭·기존 shaping 집중 | 11 + 21 + 12 + 11 PASS | `01w-session.log`, `02s-*.log` |
| 전체 lib | 4,055 PASS / 13 ignored | `03t-lib.log` |
| 전체 integration | 9,773 PASS / 51 skipped / 0 FAIL | `04r-nextest.log` |
| Native Skia lib 및 집중 2종 | 4,112 PASS / 13 ignored, 집중 2 + 4 PASS | `05-skia-lib.log`~`07-skia-pdf.log` |
| fmt·native/WASM/workspace Clippy·workspace build | 모두 PASS, `-D warnings` 유지 | `08-fmt.log`~`12-clippy-workspace.log` |
| doc test | 8 PASS / 3 ignored | `13-doc.log` |
| integration manifest·source unit tier | PASS, 생성물은 review 전용 | `14-manifest.log`, `15-unit-tier.log` |
| Docker release WASM | PASS, 8분 46초 | `docker-wasm-verified.log` |
| Studio TypeScript·npm 전체 재검사 | PASS, 1,678 PASS / 2 skipped | `studio-ts-final.log`, `studio-tests-final.log` |
| 기본 Render Diff | 3/3 PASS | `render-diff.log`, `render-diff-results.json` |
| 직접 PDF 호환 gate | 3/3 PASS, 0 failure | `direct-pdf.log`, `direct-pdf-results.json` |

집중 검사는 전체 integration의 부분집합이며 위 숫자를 합쳐 총 테스트 수로 쓰지 않는다.
nextest 0.9.137에서 권장 0.9.140 및 지원하지 않는 JUnit 설정 키 경고가 있었다.
도구 버전·정책을 임의 변경하지 않았으며 경고를 테스트 실패로 세지 않았다.

실행 명령은 다음과 같다. Cargo 계열은 동일 native target에서 순차 실행했다.

```bash
# review worktree에서만
cargo fmt --all
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_7084_canvas_metric_session -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
node scripts/run-rust-test.mjs issue_7084_supplemental_metrics -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
node scripts/run-rust-test.mjs issue_4969_shaping_emitted_run_mapping -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
node scripts/run-rust-test.mjs issue_4969_shaping_atomic_activation -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review
cargo test --locked --profile release-test --lib --target-dir /home/edward/mygithub/rhwp/target/pr-review
cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
cargo fmt --all -- --check
cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir /home/edward/mygithub/rhwp/target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo test --locked --doc --target-dir /home/edward/mygithub/rhwp/target/pr-review
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
# root
docker compose --env-file .env.docker run --rm wasm
# rhwp-studio
npx tsc --noEmit
npm test
node e2e/canvas-render-diff.test.mjs --mode=headless
RHWP_RENDER_DIFF_DIRECT_PDF=1 RHWP_RENDER_DIFF_DIRECT_PDF_GATE=1 RHWP_RENDER_DIFF_DIRECT_PDF_MAX_RATIO=0.02 RHWP_RENDER_DIFF_DIRECT_PDF_RASTER_DPI=144 RHWP_RENDER_DIFF_RHWP_BIN=/home/edward/mygithub/rhwp/target/pr-review/release-test/rhwp node e2e/pdf-render-diff-report.mjs --mode=headless
```

기본 Render Diff는 KTX 0.01593%, biz_plan/tac 각 0%로 기존 기준을 통과했다.
직접 PDF gate는 biz_plan 1.158954%, tac 0.390370%, kps-ai 0.676720%로 모두 2% 이하다.
별도 report-only PDF 비교 4건에는 기존과 같은 72 DPI raster/화면 크기 차이 경고가 남는다.
이를 직접 PDF gate 실패나 이번 변경의 신규 회귀로 합치지 않았다. 임계값·baseline은 변경하지 않았다.

## 최종 WASM과 실제 Chrome 확인

- Chrome `152.0.7977.83`, Studio `http://localhost:7700` HTTP 200.
- WASM SHA-256: `9a7500e19b1bf9e195eb7331697aed5edfc65214b77cda38f0ef0891ba58ca77`.
  11,080,648 bytes. 브라우저가 HTTP 200으로 받은 bytes/hash와 `pkg/rhwp_bg.wasm`이 일치한다.
- 원본 두 파일에서 보충 source 활성·portable 내보내기 후 복원·폰트 세대 무효화 후 복구 통과.
  이모지 paint의 x/y 변환 비는 1이며, 기능을 끈 음성 대조는 약 0.364로 다시 압축된다.
  이는 인위적 가로 축소 제거 증거이지 모든 이모지의 ink 가로/세로가 1:1이라는 뜻은 아니다.
- 장평 64%·첨자·120% 굵게/기울임의 기존 변환 계약도 통과했다.
- HWP/HWPX × VS16/ZWJ의 메모리 편집 4개 시나리오에서 같은 서식/다른 서식 모두
  부분 보충 요청 0건, 부분 보충 source 적용 0건, 오류 0건이다. 원본 파일은 수정하지 않았다.
  이는 미지원 묶음의 기존 경로 보존 증거이며 VS16/ZWJ 전체 조형 지원 주장이 아니다.
- 결과: `connected/runtime.json`, `run-boundary.json`; 실행 스크립트는 같은 Stage 5 output의
  `studio-connected-check.mjs`, `check-run-boundary.mjs`다.
- 직접 확인 이미지: `connected/hwp-after.png`, `connected/hwpx-after.png`.
  두 원본의 기존 메인테이너 시각 통과를 유지한다. 음성 대조 `hwp-before.png` 및 보존된
  `output/7084/oracle/hwp-page2.png`도 함께 확인했다. 한컴의 흑백 glyph와 OS 컬러 glyph의
  모양 동일성이나 표 간격 등 별도 결함까지 해결했다고 주장하지 않는다. PDF를 새로 생성하지 않았다.

## 비용 재측정

모든 빌드·회귀·Render Diff가 끝난 뒤 같은 Windows Chrome/입력/계측 절차로 실행했다.
폰트 로드는 완료된 상태이며 cold는 보충 cache만 비운 1회, warm은 5회 중앙값이다.
paint는 두 번째 쪽 5회, print는 portable 전환→전쪽 SVG 문자열 확보→원래 문맥 복원 3회다.

| 입력 | cold 준비 | warm 준비 | portable / 활성 paint | 전쪽 print SVG | cache 항목 / JSON bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 원본 HWP, 2쪽 | 1.2ms | 0.2ms | 5.3 / 6.1ms | 3.2ms | 1 / 752 |
| 원본 HWPX, 2쪽 | 0.9ms | 0.2ms | 6.1 / 6.4ms | 2.2ms | 1 / 752 |
| synam-001.hwp, 35쪽 | 130.5ms | 51.9ms | 123.7 / 123.8ms | 574.5ms | 10 / 8,165 |

모든 warm 추가 측정은 0건, 준비/브라우저 오류는 0건이다. `studio-cost.json`과
`measure-studio.mjs`에 원자료/방법을 보존했다. JSON bytes는 직렬화 payload이며 실제 heap 수치가 아니다.
Stage 4 관측과 비교하면 큰 문서 warm 47.6→51.9ms(+4.3ms), print 550.6→574.5ms(+23.9ms)다.
교차 반복 A/B가 아닌 순차 시점 측정이므로 이를 전부 코드 변경의 인과적 증가율로 단정하지 않는다.
큰 문서의 동기 비용이 미미하다고도 보고하지 않는다. WASM은 Stage 4 대비 13,980 bytes 증가했다.

## 잔여: descriptor 오류 뒤 화면 자동 복구

`check-descriptor-recovery.mjs`는 Canvas font getter에 의도적인 불일치를 주입한다.
실제 사용자 문서나 폰트에서 자연 발생한 장애를 발견한 것이 아니다.

- HWP/HWPX 모두 직접 painter가 `Canvas metric descriptor changed`로 그리기를 거부했다.
- 실제 CanvasView catch 경로도 오류를 기록하고 false를 반환했다. 두 animation frame 동안
  자동 재준비 호출은 0회이며, 기존 보충 세션은 활성 상태로 남았다.
- 이후 **명시적으로** 폰트 문맥을 무효화하고 재준비하면 두 파일 모두 재그리기에 성공했다.
  정상 폰트 변경의 자동 무효화/재준비 통과와 이 통제 오류의 자동 복구 미구현을 구분한다.
- 증적: `descriptor-recovery.json`, `browser-recovery.log`.

보호 경계 보완은 종료하되, 이 결과를 C의 완전한 오류 복구 통과로 쓰지 않는다.
다음 절편은 #7084 안에서 descriptor mismatch의 명시적 오류 식별→동일 문서/세대 확인→
한 번만 무효화/재준비/재그리기하는 최소 복구 설계를 검토한다. 지속 실패는 반복하지 않고
진단으로 남겨야 한다. 이번 절편에서 CanvasView 변경·추가 이슈·원격 push·PR·댓글은 하지 않았다.

## 용어

- grapheme/cluster: 사용자가 하나의 문자처럼 취급하는 Unicode 문자 묶음.
- scalar: Unicode 코드 포인트 하나. glyph나 UTF-16 코드 유닛 수와 다를 수 있다.
- nominal run: 별도 shaping 대안으로 바꾸기 전의 원문 기반 텍스트 구간.
- snapshot: 특정 문서·폰트 세대·backend에 묶인 읽기 전용 측정 자료.
- descriptor: Canvas가 해석한 font 설정 문자열. 실제 font 파일 식별과 같은 개념은 아니다.
- portable: 브라우저 전용 측정 자료에 의존하지 않는 출력 문맥.
- VS16: Variation Selector-16(U+FE0F), 앞 문자에 이모지 표시를 요청하는 선택 문자.
- ZWJ: Zero Width Joiner(U+200D), 인접 문자의 결합 표현을 요청하는 문자.
