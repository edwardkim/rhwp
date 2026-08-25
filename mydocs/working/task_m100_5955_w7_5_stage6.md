---
kind: report
status: active
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-6 제품 0-delta·전체 검증 결과

- **이슈**: #5955
- **상위 tracker**: #4960
- **작업 브랜치**: `task_m100_5955`
- **검증 head**: `270e321c8`
- **단계**: Stage W7.5-6
- **상태**: 전체 검증 완료, 결과 승인 대기
- **작성일**: 2026-08-24 KST

## 1. 판정

Stage W7.5-6의 제품 0-delta와 local validation 사다리는 모두 통과했다. canonical v2 registry는
830 active/0 retired와 projection별 171/67/281/153/158을 유지했고, 다섯 projection semantic hash는
Stage W7.5-3에서 대사한 값과 같다.

동일 head에서 만든 native release binary와 Docker optimized WASM의 공개 SVG는 W1 7문서 167페이지와 W2
대표 6문서 6페이지에서 byte mismatch 0이었다. runtime trace·renderer output과 제품 mapping의 회귀는
관측되지 않았다.

## 2. 검증 환경과 review 경계

- WSL2 Ubuntu 24, 논리 CPU 16개, RAM 약 31 GiB
- Docker Server 29.7.2
- cargo-nextest 0.9.137; 저장소 권고 0.9.140 안내는 비차단 warning
- Cargo 고정 target: `/home/edward/mygithub/rhwp/target/pr-review`
- detached review worktree에서 integration suite를 prepare·check하고 Cargo 전체 검증을 실행했다.
- nextest는 host 기본 동시성을 사용했다. 다른 Cargo·Rust 작업을 겹쳐 실행하지 않았다.
- review worktree는 검증 뒤 clean 상태에서 제거했다. generated suite·manifest는 source branch에
  복사하거나 stage하지 않았다.

prepared integration inventory는 다음과 같다.

```text
893 sources / 4,166 static test attrs
32 generated suites + 9 exceptions = 41/48 integration targets
nextest minimum 6,559 cases
machine-readable inventory 8,249 tests: 8,208 runnable + 41 skipped
```

## 3. schema·registry·projection

| gate | 결과 |
| --- | --- |
| v1 registry check | 통과 |
| v2 lifecycle registry check | 통과 |
| projection generator check | 통과 |
| pre-migration projection baseline check | 통과 |
| `scripts/tests/font_rule_*.test.mjs` | 92/92 통과 |
| Rust unit-tier | 4,221 tests / 299 modules, drift 없음 |
| W7 projection focused integration | 3/3 통과 |
| W2 public trace focused integration | 4/4 통과 |

canonical artifact 수치는 다음과 같다.

| 항목 | 값 |
| --- | --- |
| v2 registry raw SHA-256 | `fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16` |
| v2 rules SHA-256 | `bd9469aa16156a16ea262f608015cb0b78e925700ae7df69c38602ba6670c029` |
| projection semantic bundle | `090b4403832a739b7e2928fdc83741126a5cb7e05b4d3ae3fc8be17833e863a6` |
| generated content bundle | `3ba1d6c14b7514143bff42d5e1c690b4d87f41a09ef04424395f3327f772fcaa` |

| projection | active rules | semantic SHA-256 |
| --- | ---: | --- |
| `rust-layout-name` | 171 | `595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee` |
| `rust-layout-metric` | 67 | `c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2` |
| `canvas2d-paint` | 281 | `c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4` |
| `canvas2d-webfont` | 153 | `730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee` |
| `canvaskit-sfnt` | 158 | `d9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668` |

## 4. Rust 전체 검증

문서가 지정한 대형 복합 변경 순서로 실행했다.

| gate | 결과 | 시간 |
| --- | --- | ---: |
| production `cargo build --locked --release` | 통과 | 8분 25초 |
| release library | 4,071 pass / 13 ignore | 6분 53초 |
| release-test 전체 nextest | 8,208 pass / 41 skip | 5분 50초 |
| Native Skia library | 4,128 pass / 13 ignore | 2분 21초 |
| Native Skia missing picture | 2/2 | 통과 |
| Native Skia direct PDF | 4/4 | 통과 |
| `cargo fmt --all -- --check` | 통과 | — |
| `git diff --check` | 통과 | — |
| Clippy all-targets `-D warnings` | 통과 | 1분 10초 |
| rustdoc | 8 pass / 3 ignore | 57초 |

nextest는 `--no-fail-fast --status-level fail --final-status-level fail`로 실행해 실패 본문 중심으로 출력했다.
exit code 0 뒤 같은 prepared inventory를 JSON으로 조회해 8,249개 중 default-run 대상 8,208개와 skip 41개를
대사했다.

## 5. Studio·WASM·runtime trace

| gate | 결과 |
| --- | --- |
| Studio `npx tsc --noEmit` | 통과 |
| Studio·editor Node test | 1,070 pass / 1 skip |
| Studio production build | 통과, 223 modules |
| Docker optimized WASM | 통과, 5분 57초 |
| fresh WASM Decision Trace E2E | 3/3 통과 |

Docker 표준 경로로 다시 만든 `pkg/rhwp_bg.wasm`의 SHA-256은
`65eb836f5f0e192751e9e256d98f9e4dcd87f25b3524e09d6edfbe95f403a43e`이다. native와 WASM은 모두
`0.8.4`를 보고했고, public trace는 exact/missing/substFont 계보, key·font enumeration 결정성, 4,096
상한과 backend 미지원 fail-closed를 유지했다.

## 6. native/WASM public byte parity

동일 `270e321c8` source에서 만든
`target/pr-review/release/rhwp`와 Docker `pkg`를 사용했다. native binary SHA-256은
`7d2eea4e1ad2a5c5f62d0844a669c4a7c79ff1ba26fd965ebcc8e99be8ebc21a`이다.

| 묶음 | 문서 | 비교 페이지 | mismatch |
| --- | ---: | ---: | ---: |
| W1 공개 HWP | 7 | 167 | 0 |
| W2 공개 trace 대표 HWP/HWPX | 6 | 6 | 0 |

일치 SVG는 하네스 정책대로 보존하지 않았고, aggregate 임시 report도 수치 대사 뒤 휴지통으로 정리했다.
private 10k corpus, Hyper-V Oracle, host 절대경로 자료와 로컬 font bytes는 사용하지 않았다.

## 7. 보호 불변식 판정

- v1 봉인 artifact와 v2 canonical registry에 검증 중 write하지 않았다.
- v2 population은 830 active/0 retired이고 five-plane population도 그대로다.
- Stage W7.5-3에서 고정한 다섯 semantic projection hash가 모두 같다.
- selection·trace·renderer 결과의 회귀가 focused/full/public parity gate에서 발견되지 않았다.
- generated integration suite·manifest는 review 증적에만 사용하고 source branch에 남기지 않았다.
- current runtime trace envelope와 renderer API를 확장하지 않았다.
- private corpus·새 Oracle 수집은 계획대로 재실행하지 않았다.
- W7.5 검증은 #4967의 실제 mapping change set이나 rank 8 qualification을 승인하지 않는다.

## 8. 현재 상태와 다음 경계

검증 종료 뒤 main worktree는 `task_m100_5955` head `270e321c8`에서 clean이다. fresh Docker `pkg`와 Studio
`dist`는 gitignore된 로컬 검증·개발 산출물이며 source diff가 아니다.

결과 승인을 받으면 Stage W7.5-6 보고서와 계획 상태만 단계 경계 커밋으로 고정한다. 다음 Stage W7.5-7은
self-review, 최종 보고서, 최신 `upstream/devel` merge simulation과 PR 준비를 수행한다. remote push와 PR
생성은 각각 별도 승인 대상이다.
