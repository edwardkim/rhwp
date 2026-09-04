# Task M100 #2185 Stage 4 완료보고서 — 광역 회귀 게이트와 결과 정리

## 목표

최신 `upstream/devel`을 반영한 최종 코드에서 전체 Rust, 정적 검사, WASM, Studio와
저장소 내 시각 정합 게이트를 다시 통과하는지 확인한다. 저장소에 없는 제3자 로컬
코퍼스는 통과로 간주하지 않고 실행 여부와 최종 범위 판단을 분명히 기록한다.

## 최신 upstream 동기화

- 최초 작업 기준: `upstream/devel@53a5093c`
- Stage 4 최초 동기화 기준: `upstream/devel@6f1bd284`
- PR 게시 전 최종 동기화 기준·merge-base: `upstream/devel@3077f96d`
- 최종 재검증 대상 HEAD: `1f467079`
- 최종 관계: `upstream/devel` 대비 `0 behind / 4 ahead`

첫 동기화에서 반영한 네 커밋은 PR #2188의 CanvasKit replay와 검토 문서 영역이었다.
PR 게시 직전에는 PR #2184의 NO_LS 셀 회계, PR #2191의 CanvasKit replay 보강과 관련
문서 여섯 커밋을 추가 반영했다. #2185의 composer·style resolver·전용 통합 테스트와
upstream 추가 소스·테스트 변경의 경로 교집합은 0개였다. `mydocs/orders/20260711.md`의
add/add 충돌만 양쪽 기록을 모두 보존해 해결했고 #2185 소스·테스트 diff는 바뀌지 않았다.

## 최종 코드 게이트

| 게이트 | 최종 결과 |
|--------|-----------|
| `cargo test --profile release-test --tests` | **3,042 passed / 0 failed / 22 ignored** (총 3,064개, 약 4분 9초) |
| #2185 전용 통합 테스트 | HWP/HWPX 순차 검증 통과, 전체 스위트 내 약 6.44초 |
| `cargo fmt --check` | 통과, 약 3.38초 |
| `cargo clippy --all-targets --all-features -- -D warnings` | 통과, 경고 0건, 약 33.15초 |
| `git diff --check` | 통과 |
| Docker WASM 빌드 | 통과, 최종 upstream 기준 새 `pkg/` 생성, 약 1분 45초 |
| `rhwp-studio npm test` | **185 passed / 0 failed** |
| `rhwp-studio npm run build` | 통과, 135 modules |
| `rhwp-studio npm run e2e:renderer-contract` | 통과 |

WASM 빌드를 위해 시작한 Colima는 검증 후 다시 중지해 원래 상태로 복구했다. Studio
프로덕션 빌드에는 기존 CanvasKit `fs`/`path` externalize 및 큰 chunk 경고만 있었고
실패나 새 경고는 없었다.

## 저장소 내 광역 정합 게이트

### 공식 PDF 쪽수 핀

`tests/issue_1891.rs`의 세 테스트가 독립 실행에서 모두 통과했고, 최신 upstream 동기화
후 전체 테스트에서도 다시 포함돼 통과했다. 다음 HWP/HWPX 쪽수 핀이 유지됐다.

| 문서 | 공식 PDF 기준 | 결과 |
|------|---------------|------|
| `76076_regulatory_analysis` | 82쪽 | HWP/HWPX 유지 |
| `80168_regulatory_analysis` | 157쪽 | HWP/HWPX 유지 |
| `80250_regulatory_analysis` | 17쪽 | HWP/HWPX 유지 |
| `86712_regulatory_analysis` | 65쪽 | HWP/HWPX 유지 |

### Canvas legacy/layer 정합

최신 upstream과 새 WASM 기준으로 저장소 루트에서도 서빙 가능한 여섯 픽스처의 첫 쪽을
headless Chrome에서 다시 비교했다.

| 픽스처 | 결과 |
|--------|------|
| `basic/KTX.hwp` | PASS, 116/889,746 pixels (0.01304%, 임계 0.05% 이내, max delta 84) |
| `biz_plan.hwp` | PASS, diff 0 |
| `footnote-01.hwp` | PASS, diff 0 |
| `kps-ai.hwp` | PASS, diff 0 |
| `shift-return.hwp` | PASS, diff 0 |
| `tac-case-001.hwp` | PASS, diff 0 |

11개 `ALL_FIXTURES` 일괄 실행은 첫 항목 `BlogForm_BookReview.hwp`를 `/samples`에서
가져오는 단계에서 HTTP 404로 중단됐다. 파일은 `rhwp-studio/public/samples`에는 있지만
저장소 루트 `samples`에는 없고, 현재 Vite `/samples` 미들웨어가 루트 샘플을 우선
처리하는 서빙 제약 때문이다. 같은 조건의 public 전용 픽스처는
`form-002.hwpx`, `number-bullet.hwp`, `oullim-01.hwp`, `para-head-num-2.hwp`까지
총 5개다. 렌더 결과의 불일치로 실패한 것은 아니지만 이들을 포함한 11개 전체
통과로도 기록하지 않는다.

## 외부 로컬 자산의 최종 분류

| 계획상 항목 | 현재 환경 | 최종 판정 |
|-------------|-----------|-----------|
| #2169 `kbu`/`kbu2` 통제 자료 | 픽스처·한컴 기대값·대조 절차가 저장소에 없음 | 미실행, 비차단 참고 항목 |
| 359 문서 목록·기준선·recount 도구 | 목록·기준선·재현 가능한 실행 자산이 저장소에 없음 | 미실행, 비차단 참고 항목 |
| Windows 대량 코퍼스 | 원본 문서와 결과가 이 환경 및 저장소에 없음 | 미실행, 비차단 참고 항목 |

구현계획은 이 자산이 없으면 완료를 선언하지 않도록 작성됐으나, 작업지시자가 세 항목이
이슈 작성자 `@planet6897`의 로컬 파일이며 현재 작업과 무관하다고 확인하고 비차단
처리를 승인했다. 따라서 최종 완료 범위는 다른 작업자가 보유한 비버전 로컬 자산이 아니라
저장소에서 제3자가 반복 실행할 수 있는 게이트로 조정했다.

위 세 항목에 대해서는 `REGRESSED=0`을 주장하지 않는다. 공개 또는 저장소 추적 자산으로
전환되면 별도 보강 게이트로 실행할 수 있다.

## 작업공간 확인

- #2185 추적 파일 외 생성·변경된 추적 파일은 없다.
- 기존 미추적 `scripts/frontend-metrics/`는 수정하거나 stage하지 않았다.
- Studio 개발 서버는 추가 확인을 위해 `127.0.0.1:7700`에서 계속 실행 중이다.

## 판정

최신 upstream 기준으로 반복 가능한 전체·표적·포맷·WASM·Studio·Canvas 게이트에서 새
회귀가 발견되지 않았다. 외부 로컬 코퍼스는 통과로 오인하지 않도록 미실행으로 남겼고,
작업지시자가 승인한 조정 범위에서는 Stage 4 완료 조건을 충족한다.
