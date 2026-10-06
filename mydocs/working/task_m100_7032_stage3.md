# #7032 Stage 3 — 전체 회귀·lint·WASM 검증

- 선행: [Stage 2](task_m100_7032_stage2.md) R1 시각 판정 및 R2 focused 완료
- 승인: 메인테이너의 “다음 절차 진행을 승인합니다.”
- 상태: **신규 HWPX 기준선 2행 등록 후 전체 nextest 9,490 PASS / 0 FAIL / 46 skipped. WASM 준비 완료.** clipping 외부 원본 부재와 새 Docker 산출물의 메인테이너 시각 판정 여부는 아래 제약을 유지한다.
- 소스 기준: `532b74fc5` (Stage 2 구현), Stage 3 시작 문서 commit으로 review HEAD를 고정한다.
- 검증 worktree: `/home/edward/mygithub/rhwp-review-7032`
- 고정 Cargo target: `/home/edward/mygithub/rhwp-shared-review-target`
- 로그: `output/7032/stage3/`

## 실행 순서

1. review worktree 동기화 및 integration suite prepare, fmt, native/WASM/workspace Clippy.
2. `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 전체 회귀.
3. 새 HWPX fixture 보안 검사 및 여섯 코퍼스 래칫의 해당 여부 확인.
4. Native Skia lib + 그림 placeholder + 직접 PDF 출력 검증.
5. 표준 Docker `wasm` 빌드 후 동일 산출물로 Studio 검증 준비.

같은 Cargo target을 사용하는 실행은 순차로 한다. CPU 16개, RAM 31GiB(시작 시 가용 28GiB),
디스크 가용 366GiB를 확인했으며 nextest 기본 동시성을 사용한다. Docker 서버 응답도 확인했다.
테스트 실패·미실행·기존 ignored는 별도로 기록하고, 새 baseline이나 기대값을 임의 갱신하지 않는다.
원격 push·PR 생성·병합은 이 단계에 포함하지 않는다.

## Rust lint / workspace

검증 HEAD는 `63fe746c7`이며 아래 명령을 같은 review target에서 순차로 실행했다.
`cargo fmt --all` 뒤 tracked diff가 없음을 확인했다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| suite prepare, fmt, fmt check | PASS | `prepare.log`, `fmt.log`, `fmt-check.log` |
| native Clippy `-- -D warnings` | PASS, 55.32초 | `clippy-native.log` |
| WASM lib Clippy `--target wasm32-unknown-unknown` | PASS, 52.01초 | `clippy-wasm.log` |
| workspace build | PASS, 1분 24초 | `build-workspace.log` |
| workspace all-targets Clippy | PASS, 1분 20초 | `clippy-workspace.log` |
| suite manifest check | PASS, 48/48 integration targets | `manifest-check.log` |

source-side `#[cfg(test)]`를 수정하지 않았다. 파생 suite·manifest는 review worktree의
검증 산출물이며 제출 대상으로 stage하지 않는다.

## 전체 회귀 실행 조건

`RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 신규 HWPX 한 건
`samples/hwpx/21761835_jeonjik_exemption_table.hwpx`를 명시했다. IR / overflow-cell /
off-canvas / text-overlap dump 환경변수도 지정하여 래칫 실측을 보존한다.
실행 로그는 `nextest-full.log`다. 최적화 빌드 4분 49초 후 실행 416.528초에 종료했다.
**9,490건 중 9,489 PASS / 1 FAIL / 별도 46 skipped**다. 전체 통과가 아니다.

- #7032 신규 테스트 7/7 PASS, 새 HWPX를 입력한 보안 검사 PASS(보안 module 6/6 PASS).
- overflow-cell / off-canvas / text-overlap / oracle-page-count: 각각 16개 partition PASS.
- IR field sweep만 FAIL. 신규 HWPX의 미등록 경로 두 개가 원인이다(아래 대조 실험).
- 52,694셀 대형 표 #2063도 303.142초에 161쪽 pin PASS. `SLOW` 알림을 실패로 판정하지 않는다.

설치된 nextest `0.9.137`은 required `0.9.91` 이상이나 recommended `0.9.140`보다 낮아
권고 버전 경고가 출력된다. 별도 CI 관측 profile의 `junit.report-skipped` 키도 지원하지 않는다는
경고가 있다. 이번 명령은 default profile을 사용하며 경고를 테스트 성공 근거로 해석하지 않는다.

### clipping 검사 제약

`tests/fixtures/render_page_controlset.tsv`의 외부 원본 92건을 경로 존재 여부로 확인했다.
**92건 모두 부재하여 clipping gate는 미검증**이다. 검사 대상 0건을 통과로 기록하지 않는다.
신규 HWPX는 이 controlset에 등록된 문서가 아니다. 나머지 코퍼스 래칫의 실제 결과와 별도로 기록한다.

## IR 래칫 실패 원인 대조

review worktree를 잠시 `550d04840`으로 전환하여 같은 fixture를 검사했다. 이 commit은
샘플·계획만 준비하고 구현하기 전이며, `d408532ce`와 `src`, Cargo 설정·lock 차이가 없다.
원래 작업 브랜치의 소스나 메인테이너 파일은 되돌리지 않았다. 검사 후 review HEAD는
`63fe746c7`으로 복귀하고 suite를 다시 prepare했다.

| 정규화 경로 (`sections[].paragraphs[]` 이후) | 구현 전 | 구현 후 |
| --- | ---: | ---: |
| `controls[].cells[].paragraphs[].raw_header_extra[]` | 1,432 | 1,432 |
| `raw_header_extra[]` | 24 | 24 |

`ir-baseline-control.log`의 구현 전 focused 실행도 동일한 두 경로로 FAIL했다(검사 module
3 PASS / 1 FAIL). 전수 dump `ir-baseline.tsv`와 `ir-current.tsv`는 **260행 전체가
바이트 단위로 동일**하다. 이번 변경에는 parser/model/serializer/IR 진단 코드 차이도 없다.
따라서 이번 조판 구현의 회귀가 아니라 새 fixture의 기존 왕복 특성으로 판정한다.

### 값의 의미

HWPX 파서는 `<hp:p id>`를 `raw_header_extra[6..10]`에 보존한다
(`src/parser/hwpx/section.rs`). 기존 HWPX 저장기는 `next_para_id()` 전역 카운터로
본문·셀 문단 ID를 재부여한다(`src/serializer/hwpx/context.rs`, `table.rs`, `section.rs`).
이번 1,456건은 문단 ID 바이트의 차이이며 사라진 글자나 손상 문단 수가 아니다.
구현 전 sweep의 상세 예시를 모두 보존했고, 별도 CLI 재저장/`ir-sweep --max-lines 10000`
대조에서도 ID 바이트 차이 1,456건을 전부 확인했다: index 6=569건, 7=316건, 9=571건.

CLI export는 sweep의 직접 왕복 경로와 구분한다. CLI 상세에는 본문 pi=9의
`char_shapes.len` 0→1 한 건도 있어 총 1,457건이다. 이를 IR 래칫의 실패 건수와 섞지 않는다.
두 버전의 CLI 재저장 파일은 **서로 바이트 단위로 동일**하며 SHA-256은
`6af0973becc638bd34ab936a166f5a946fd2ddbfd212e807ad37e0f42a484857`이다.
이는 원본과 재저장본이 같다는 의미나 재저장본의 한컴 시각 판정 완료를 뜻하지 않는다.

- 입력 SHA-256: `53b33bfca8d8a80534f27564ecee86403c6d1e3dc83cf405c7d6d158ae9625d2`.
- 추가 증적: `current-ir-detail.json`, `current-roundtrip.hwpx`, `baseline-roundtrip.hwpx`.
- 선행 사례: [#6950 Stage 3 §20](task_m100_6950_stage3.md#203-필드-의미와-발생-계보).
- 권고: **이 신규 샘플의 hwpx lane 실측 2행만** IR baseline에 등록한 뒤 전체 회귀를 재확인한다.
  다른 샘플의 감소분, 검사기, 제품 코드는 바꾸지 않는다. 현재 baseline은 수정하지 않았으며
  메인테이너 승인 후 반영할 대상으로 남긴다.

## Native Skia

제품·검사 소스는 변경하지 않고 review `63fe746c7`에서 같은 Cargo target으로 순차 실행했다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| `cargo test --locked --profile release-test --features native-skia --lib` | 4,112 PASS, 13 ignored | `skia-lib.log` |
| `issue_2225_missing_picture_placeholder` | 2/2 PASS | `skia-placeholder.log` |
| `render_p37_direct_pdf_export` | 4/4 PASS | `skia-pdf.log` |

lib의 패키지별 PASS는 rhwp 3,930, contracts 15, ooxml-chart 165, password-crypto 2건이다.
두 integration 검사의 필터 제외(179 / 145)는 실행한 테스트 통과 건수에 포함하지 않는다.

## Docker WASM / Studio 준비

`docker compose -p rhwp --env-file .env.docker run --rm wasm`을 review worktree에서 실행한다.
실행 중인 rhwp build container가 없는 것을 확인하고, 기존 `rhwp` named volume을 재사용했다.
review의 `.env.docker`가 없을 때만 예제를 복사했다. 별도 캐시·개발 서버를 새로 만들지 않는다.
Docker 빌드는 **PASS, 6분 52초**다(`docker-wasm.log`). 생성된 review `pkg/`를 기존
checkout의 `pkg/`에 복사했고, 실행 중인 Studio 서버(7700)는 재시작하지 않았다.

- WASM: 10,512,661 bytes, SHA-256
  `0b4b7a40855c259a6c2203bb81116ec06a69bc63adc52a83f560c11074239c28`.
- review 산출물, Studio checkout의 파일, `http://localhost:7700/@fs/home/edward/mygithub/rhwp/pkg/rhwp_bg.wasm`
  HTTP 200 응답 파일의 SHA-256이 모두 일치한다(`wasm-http.json`).
- 실제 WASM API로 원본 HWP / HWPX를 열어 각각 6쪽 SVG 출력 및 각 쪽의 `직렬` 텍스트 존재를
  확인했다(`wasm-smoke.mjs`, `wasm-smoke-final.json`). 이 smoke는 배치 좌표의 시각 판정을 대체하지 않는다.
- 최초 보조 smoke에서 SVG XML의 연속 문자열 `직렬`을 직접 찾은 단언은 실패했다.
  출력이 글립/하위 태그로 분리되는 것을 확인하고 text 노드 내용으로 검사하도록 정정했다.
  제품 코드는 수정하지 않았으며 최초 결과 `wasm-smoke.json`도 보존했다.
- 브라우저 새로고침 후 두 원본 파일을 다시 열어 첫 셀의 빈 줄·`직렬` 위치와 후속 쪽 반복 제목셀을
  메인테이너가 확인할 수 있다. R1 시각 판정은 이미 통과했지만 이번 Docker 산출물의 시각 판정은 미수행이다.

## 등록 전 권고 — 아래에서 승인·완료

1. IR baseline에 이 신규 HWPX의 실측 2행(1,432 / 24)만 등록한다.
2. baseline 변경 후 전체 회귀를 다시 실행하고, 통과하면 최종 보고·PR 준비 단계로 넘어간다.

등록 전 첫 전체 회귀의 1 FAIL을 PASS로 간주하지 않는다. 등록 후 최종 결과는 아래에 별도로 기록한다.
clipping 외부 원본 부재는 미검증으로 유지한다.
원격 push·PR 생성·병합은 수행하지 않았다.

## 신규 샘플 등록 승인 및 재검증

메인테이너가 “새로 추가된 샘플이기 때문에 신규 등록해서 처리하세요”로 승인했다.
위 대조에서 확인한 hwpx lane 2행(셀 문단 1,432 / 본문 문단 24)만
`tests/fixtures/ir_field_sweep_baseline.tsv`에 사전순으로 등록했다.
등록은 원래 정상인 HWPX 렌더링을 수정한 것이 아니다. 기존 저장기의 문단 ID 재부여를
새 샘플의 검사 기준으로 기록한 것이다. 제품 코드·검사기·다른 샘플 기준값은 변경하지 않았다.

등록 commit을 review HEAD로 고정하고 새 샘플 보안 입력을 유지하여 전체 nextest를 재실행한다.
로그는 `output/7032/stage3/nextest-registered.log`다. 직전 lint·Native Skia·WASM 검증과
제품/Rust test 소스가 동일하므로 그 결과는 유지하며, 재검증 결과는 완료 후 기록한다.

### 등록 후 최종 결과

- 등록 commit 및 검증 review HEAD: `4b74cf032`.
- 전체 nextest: **9,490 PASS / 0 FAIL / 46 skipped**, exit 0.
  최적화 빌드 4분 35초, 테스트 실행 422.106초. 느린 검사 8건도 모두 PASS다.
- 마지막 IR 왕복 검사도 89.182초에 PASS. 신규 HWPX의 보안 검사와 #7032 회귀 7건도 PASS다.
- `ir-registered.tsv`와 등록 전 `ir-current.tsv`가 바이트 단위로 동일하다.
  실제 재저장 동작을 바꾼 것이 아니라, 확인된 기존 문단 ID 재부여 특성을 새 샘플 기준으로 등록했다.
- 최종 suite manifest check PASS(`manifest-registered.log`). 생성 suite·manifest는 stage하지 않았다.
- 직전 검증 HEAD `63fe746c7` 대비 `src`, `tests/cases`, `Cargo.toml`, `Cargo.lock` 변경 없음.
  따라서 이미 통과한 Rust lint·Native Skia·Docker WASM 증적과 Studio 제공 산출물을 유지한다.

이번 승인 범위인 신규 샘플 등록과 전체 회귀 재확인을 완료했다. 다음은 최종 보고·PR 준비 절차다.
원격 push·PR 생성·병합은 아직 수행하지 않았다.
