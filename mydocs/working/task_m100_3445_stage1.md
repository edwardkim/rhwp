# v0.8.2 핫픽스 1단계 — 기준선 고정과 코드 검증

Issue: #3445
브랜치: `task/3445-release-v0.8.2`
기준선: `origin/devel` = **`732147a30`** (PR #3446 merge 완료)

## 1. 기준선

v0.8.1 과 달리 이번에는 한 번에 확정됐다. 착수 전 #3433 수정(PR #3446)을 먼저 merge 해
릴리즈 범위에 포함시켰고, 작업지시자가 그 시점의 `devel` HEAD 로 고정을 확정했다.

`v0.8.1..732147a30` 범위의 실질 변경은 두 건이다.

| 항목 | 내용 |
|---|---|
| #3433 | 확장 인쇄 복구 — `print.html` 빌드 복사 + 필수 산출물 게이트 |
| #3396 | TAC 인라인 표 x-원점 outMargin 배선 (Rust 렌더러) |

## 2. 검증 결과

`local_validation` 규약대로 `CARGO_INCREMENTAL=0` 을 적용하고 Cargo 명령을 순차 실행했다.

| 검증 | 결과 |
|---|---|
| `cargo build` | 통과 (38.7s) |
| `cargo test --profile release-test --tests` | **4160 passed / 0 failed** |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | **경고 0** |
| Docker WASM 빌드 | 성공 (4m31s) |
| studio `npm run build` | 성공 |
| studio `npm test` | **641 pass / 0 fail** |

테스트 수가 v0.8.1 의 4159 에서 4160 으로 늘었다. #3396 렌더 정정이 추가한 테스트다.

### 확장 빌드 — 이번 핫픽스의 본체

| 확장 | 결과 |
|---|---|
| `rhwp-chrome` | 빌드 exit 0, `dist/print.html` **1436 bytes** |
| `rhwp-firefox` | 빌드 exit 0, `dist/print.html` **1436 bytes** |

#3433 에서 추가한 필수 산출물 게이트가 함께 동작한다.

### E2E

| 스위트 | 결과 |
|---|---|
| `e2e:undo` | 통과 |
| `e2e:renderer-contract` | 통과 |
| `print-pdf-issue3126` | **83 PASS / 4 FAIL** → [#3450](https://github.com/edwardkim/rhwp/issues/3450) |

## 3. E2E 실패 처리 — #3450

`print-pdf-issue3126` 에서 PDF 안내 모달 관련 4건이 실패했다.

```
FAIL: PDF 안내 모달 표시
FAIL: PDF 모달의 명시적 확인 버튼
```

**인쇄 surface 자체는 정상이다.** 같은 실행에서 다음이 통과했다.

```
PASS: 전용 print.html surface
PASS: same-origin print iframe
PASS: 모달 확인 뒤 print() 자동 1회 호출
PASS: 네이티브 인쇄창 호출 전에 PDF 모달 제거
```

### 이번 릴리즈가 만든 실패가 아니다

- 테스트 파일 `e2e/print-pdf-issue3126.test.mjs` 와 소스 `src/command/commands/file.ts`
  모두 마지막 변경이 `14ded0fc5`(2026-07-24)다.
- **v0.8.1 태그와 현재 HEAD 사이에 두 파일 모두 무변경**이다.
- v0.8.2 범위의 변경은 `build.mjs` 2개와 Rust 렌더러뿐으로 studio 런타임을 건드리지 않는다.

v0.8.1 1단계에서 이 스위트를 실행하지 않아 발견되지 않았다. 근인 진단과 v0.8.1 태그 대조는
수행하지 않았고, 그 사실을 #3450 에 명시했다.

작업지시자 판단 없이 릴리즈를 중단하지 않고, #3412 때와 같은 방식으로 별도 이슈로 분리한다.

## 4. 다음 단계

2단계 — 버전 갱신 9파일 + CHANGELOG 3종 + 스토어 문서 4종. 작업지시자 승인 게이트.
