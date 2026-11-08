# #7952 — 최종 로컬 검증과 PR 준비

Issue: [#7952](https://github.com/edwardkim/rhwp/issues/7952)

## 검증 대상

- 코드·테스트 commit: `9acd01745de69c09b0d173abd5aa80e1f9b3fd74`.
- 기준 devel: `552514f423ec91f1b7171f6e6b0882a4efe034f2`.
- clean 검증 worktree: `/home/tsjang/rhwp-7952-verify`.
- host: ubuntu-ted2 / Linux x86_64 / Rust 1.93.1 / 논리 CPU 16 / build jobs 8 / test threads 12.
- 공용 target: `/home/tsjang/rhwp/target/pr-review`. Cargo는 순차 실행했다.

## 최종 게이트

| 검사 | 결과 | 총 소요 초 |
| --- | --- | --- |
| `final-suite-prepare` | PASS | 2.441 |
| `final-fmt` | PASS | 40.541 |
| `final-focused-query` | PASS | 132.201 |
| `final-clippy-native` | PASS | 5.220 |
| `final-clippy-wasm32` | PASS | 1.804 |
| `final-build-workspace` | PASS | 74.557 |
| `final-clippy-workspace` | PASS | 29.415 |
| `final-suite-check` | PASS | 3.494 |
| `final-full-nextest` | PASS | 1029.194 |

**focused 227/227 PASS**, 전체 **10,459/10,459 PASS / 45 skipped**다.
전체 컴파일은 3분 49초, 실행은 792.891초다. 기존 ignored 45개를 유지했으며 새 skip은 없다.
최종 실행은 first run의 실패 검사도 모두 포함한다. 첫 실행과 보정 이유는 [stage 2](task_m100_7952_stage2.md)에 기록했다.

실행한 lint는 format, native Clippy, WASM32 lib Clippy, workspace build 및 workspace all-target Clippy다.
원본 명령·종료 코드·소요 시간은 `validation-results.json`, 전체 nextest 로그에 보존했다.

## 인터페이스·문서·산출물

- 실제 release 빌드 PASS. 172개 개별 도움말의 모든 설명 절과 파일 없는 실행을 확인했다.
- 닫힌 stdout 파이프는 초기 구현 exit 101을 재현하고 보정 후 exit 0을 확인했다.
- 기존 캐시의 CLI와 사용법 오류·공개 sample JSON 32건을 대조해 일치했다. 상세 도움말의 출력 변경은 의도한 기능이다.
- 매뉴얼 실제 HWP 예제 13건 및 HWPX 조회를 release로 실행했다. Windows 명령은 구문 안내이며 이번 host에서 실행하지 않았다.
- 호출 Node 18개·Python 44개, suite 정책 도구 Node 23개, 기존 Rust unit tier 정책을 통과했다.
  이 검증 뒤 `scripts/`, 제품 `src/`, Cargo 입력은 바꾸지 않았으며 9acd01745와 동일함을 git diff로 대조했다.
- 최종 Cargo metadata의 루트 bin은 `rhwp`, `rhwp-q`, `rhwp-agent`, `font-metric-gen` 4개, q 실행 파일은 1개다.
- release artifact: 15,044,584 bytes, SHA-256 `dac55f809ba5ed41490af30301b45660d8713c675dc33fccf0f84cf47d08df24`.
- 통합 dev artifact: 198,702,248 bytes. 기존 캐시 q 23개의 논리 합계는 7,798,918,392 bytes였다.
  이 비교는 통합과 dev debug 정보 축소 효과를 함께 포함하며 통제된 동일 소스 A/B 빌드가 아니다.
  실제 디스크 확보량 또는 전체 빌드 시간 단축률로 해석하지 않는다. 공용 캐시와 기존 실행 파일은 보존했다.

## 입력 커밋 확인

| 재사용 입력 | SHA-256 |
| --- | --- |
| `samples/form-01.hwp` | `75b6b8bb9121c6aeebf567aaa158ac7ec95caae597fd8a5ec7d4c84a25b927ac` |
| `samples/hwpx/form-01.hwpx` | `3bbd207b88fe61e802706de3ccf98abdb8b450493164eec657c9ee88a5aba87e` |
| `samples/pic-crop-01.hwp` | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` |
| `samples/equation-lim.hwp` | `ba168d388df614e7b9390c4685d368cf3019595c5fecbce0e681dc8fb9bea36d` |

위 4개는 9acd01745의 Git blob과 실행 파일 바이트가 동일하다. 새 HWP/HWPX/PDF fixture는 없다.

## 조판 원칙·시각 검증 적용 여부

**비해당**. 변경한 생산 경로는 CLI target 등록, argv 전달, 도움말과 호출 script다.
DocumentCore·원본 저장 모델·font 공급·renderer·paint·WASM 구현, 렌더링 golden·px/쪽수 기대값은 바꾸지 않았다.
새 검사는 라우팅·JSON 식별자·도움말·입력 전달·승인된 root binary 집합을 보호하며 배치 품질을 주장하지 않는다.

## 현재 승인과 후속 절차

2026-10-17 사용자가 목표를 “pr 준비되면 pr 하고 manual/pr_review_workflow.md, post merge md 준수.”로 갱신했다.
PR 전 최신 devel head를 확인하고 바뀐 경우에만 rebase·영향 검증한다. push·Open PR 뒤 실제 code head CI를 확인한다.
CI 성공 뒤 같은 PR에 archive review·오늘할일의 문서 trailing commit을 추가하고 최신 trailing CI도 확인한다.
merge SHA·duration 갱신·이슈·후속 comment·devel sync·작업 branch/worktree와 `output/` 전체 정리를 순서대로 수행한다.
user dirty `rhwp-studio/public/rhwp.js`와 다른 작업·공용 target은 보존한다.

로그 실행 경로는 ignored `output/pr-review/cli-build-scope-20261016/`다. merge 뒤 필요한 증거는
로컬 영구 경로 `/home/tsjang/rhwp/pdf/pr-review-evidence/issue7952-20261017/`로 보존한 뒤 output을 정리한다.
이 문서는 로컬 검증 결과이며 GitHub CI·PR review·merge 완료를 미리 선언하지 않는다.
