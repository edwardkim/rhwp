# v0.8.1 릴리즈 1단계 — 기준선 고정과 코드 검증

Issue: #3401
브랜치: `task/3401-release-v0.8.1`
기준선: `origin/devel` = `6814bf431` (main 대비 79커밋)

## 1. 기준선 확정 경과

기준선은 두 차례 갱신됐다. 갱신될 때마다 이전 검증 결과를 재사용하지 않고 재실행했다.

| 시점 | 기준선 | 사유 |
|---|---|---|
| 최초 | `0ff167a44` | 착수 시점 devel |
| 1차 갱신 | `ce2156dad` | dependabot 통합 2건 선행 처리 |
| 확정 | **`6814bf431`** | lpaiu-cs #3400 포함(작업지시자 지시) |

`origin/devel` 과 격차 0/0 동기화를 확인했다.

## 2. 검증 결과

`local_validation` 규약대로 `CARGO_INCREMENTAL=0` 을 적용하고 Cargo 명령을 순차 실행했다.

| 검증 | 결과 |
|---|---|
| `cargo build` | 통과 (36.4s) |
| `cargo test --profile release-test --tests` | **lib 2932 passed / 0 failed / 7 ignored**, 통합 테스트 전 항목 통과, exit 0 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | **경고 0** |
| Docker WASM 빌드 | 성공 (4m18s, `pkg/rhwp_bg.wasm` 7.2MB) |
| studio `npm run build` | 성공 (PWA precache 57 entries) |

새 의존성 base64 0.23.0, snafu 0.9.2 가 빌드·clippy·WASM 전 경로에서 정상 컴파일됐다.

### E2E — 10개 스위트 중 9 통과

| 스위트 | 결과 |
|---|---|
| `e2e` (text-flow) | 통과 |
| `e2e:undo` | 통과 — #3400 관련, 5개 시나리오 전부 PASS |
| `e2e:undo-object-selection` | 통과 |
| `e2e:renderer-contract` | 통과 |
| `e2e:unsaved-guard` | 통과 |
| `e2e:embed` | 통과 |
| `e2e:issue-2809` | 통과 |
| `e2e:form-edit-escape` | 통과 |
| `e2e:clipboard-priority` | 통과 |
| `e2e:drag-autoscroll` | 통과 |
| `e2e:issue-2214` | **실패** → [#3412](https://github.com/edwardkim/rhwp/issues/3412) |

## 3. 실패 처리 — #3412

`hwp run 1 after-56-sync` 체크포인트에서 누적 `wasmFlush` 가 기대값 0 대신 **2**. 2회 연속
재현했다. #2214 가 고정한 "페이지 로컬 리페인트 중 WASM flush 없음" 계약 위반이다.

**회귀 여부는 미확정이다.** v0.8.0(`main`) 기준 대조를 수행하지 않았다. 이번 릴리즈 범위의
studio 소스 변경은 #3387·#3348 두 건뿐이고 둘 다 flush 횟수와 직접 연관이 보이지 않으나,
대조 없이 단정하지 않는다.

작업지시자 판단으로 별도 이슈(#3412)로 분리하고 릴리즈는 계속한다.

### 부수 관찰 — headless 모드 WSL2 기동 실패

`npm run e2e:issue-2214` 의 `--mode=headless` 는 puppeteer 가 자체 Chrome 을 띄우는데, WSL2 에
`/sys/devices/system/cpu/cpu0/cpufreq/` 가 없어 브라우저 프로세스 기동 자체가 실패한다(테스트
코드 도달 전). `--mode=host` 로 실제 실행해 위 단언 실패를 확인했다. 같은 headless 인
`e2e:issue-2809`·`e2e:form-edit-escape`·`e2e:embed` 는 통과하므로 이 스위트만 다른 실행 경로를
탄다. #3412 에 부수 관찰로 기록했다.

## 4. 다음 단계

2단계 — 버전 갱신 10파일 + CHANGELOG 2종 + README·라이선스 점검. 작업지시자 승인 게이트.
