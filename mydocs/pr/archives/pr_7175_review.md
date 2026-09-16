---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7175_review.md
last_verified: 2026-09-16
---

# PR #7175 검토

## 메인터너 통합 보정 이후 확인

CLI 자기서술 보완 범위의 승인을 유지한다. #7141·#7178 보정 `6cdca9464`의 전체 검증을
통과했으며 #7141 추가 증적 `4670dce74`는 실행 동작을 바꾸지 않는다.
[최종 보정 기록](../../working/task_m100_7095_6946_maintainer_stage2.md).


## 최종 판정

**승인**. 검토 대상은 아래 원 PR 및 누적 통합 code head다.
이 판정은 GitHub approve·push·통합 PR 생성·merge 완료를 뜻하지 않는다.
CLI 자기서술 보완 범위에서 차단 결함 없음. 통합 batch의 다른 PR 보류는 별도다.

## 대상과 적용

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR | [#7175](https://github.com/edwardkim/rhwp/pull/7175) |
| 제목 | 수정(진단): layout-anomaly 자기서술에 stored-line-escape 축과 옵션을 싣는다 (#7061) |
| 작성자 / reviewer | planet6897, 기존 기여자 / jangster77 요청 및 API 확인 |
| 원 head | `2cf37aba8d7e08f6433f8d10b87cac6b043f2137` |
| base / 규모 | devel / 8 files, +77/-23 |
| 통합 base | `263b61a64a77a0679e9d8679c5be2e1d180cee1a` |
| 통합 code head | `21164e71a8a84c6204edcad58723f154256587da` |
| branch | `codex/pr7141-7175-7178-20260916` |
| 원 PR 상태 | OPEN / non-draft / MERGEABLE / CLEAN |
| 원 head CI | [Full CI success](https://github.com/edwardkim/rhwp/actions/runs/34991806960) — run head SHA 일치 확인 |

순서는 #7141 → #7175 → #7178이며 기능·test·문서 12개 commit을 `-x`로 체리픽했다.
#7141의 devel merge commit은 제외했으며 원 head와 최신 base의 merge tree가 해당 체리픽 결과와
바이트 동일함을 확인했다. 세 PR 모두 텍스트 충돌은 없었다. [실행 기록](pr_7175_review_impl.md)에
source/local SHA를 남겼다. 각 원 PR 최신 head의 CI 성공은 통합 head의 CI 성공을 대신하지 않는다.

base route: `collaborator_external_pr.md` (기본 작업공간의 devel 기반 누적 체리픽).
loaded: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_external_pr.md`,
`intake_and_review.md`, `multi_pr_update_branch.md`, `local_validation.md`, `visual_fixture_evidence.md`.
시각 판정에는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 적용했다.

## 변경과 검토 결과

`layout-anomaly`의 기존 stored-line-escape 진단을 help, capabilities, MCP schema와 문서에 노출한다.
MCP의 `storedLineTolerance`는 `--stored-line-tolerance`로 연결되고,
성공/배치 오류 봉투에 `storedLineTolerancePx` 및 빠졌던 count 필드를 맞춘다.
진단 알고리즘이나 renderer 배치는 변경하지 않는다.

새 CLI로 capabilities 및 `capabilities --mcp`를 직접 열어 flag·recordFields·optionalArgs 매핑을 확인했다.
`--stored-line-tolerance 0`, `0.5`, `2`는 exit 0이고 요청 값이 JSON에 그대로 나타났다.
문자열 `nope`는 exit 2였다. 기존 parser가 음수도 받는 동작은 그대로이며, MCP schema는 minimum 0이다.
단건 존재하지 않는 파일은 stdout JSON 대신 기존 stderr 오류 경로를 쓰므로,
배치 전용 오류 envelope 변경과 혼동하지 않았다. `layout_anomaly_contract` 14개가 모두 통과했다.

원 PR의 `Refs #7061`은 진단 전체 신규 구현을 뜻하지 않는다. 이 PR은 자기서술 보완 범위로 수용한다.
렌더 변경·독립 PDF 비교·조판 규칙·baseline 변경은 비해당이다. 발견한 차단 결함은 없다.

## 실제 검증

- 전용 target: `target/pr7141-7175-7178-20260916`. 기존 target과 실행 중 Cargo/Rust 작업을
  확인하고 별도 경로를 사용했다. Cargo 작업은 순차 실행했다.
- `cargo build --locked --profile release-test --target-dir target/pr7141-7175-7178-20260916`: exit 0.
- 관련 9개 case를 manifest에서 실제 suite로 해석해 `cargo nextest run --locked --cargo-profile
  release-test --target-dir target/pr7141-7175-7178-20260916 --test <해석된 suite> --no-fail-fast
  -E <case별 OR 필터>`로 실행: **51 passed / 1522 filtered skips**, exit 0.
  #7095 3, #4771 6, #6924 1, #7086 4, #6946 4, #6795 5, #6981 8, layout_anomaly_contract 14,
  security_corpus_regression 6이다. security 입력은 새 44529 fixture 1개다.
- 첫 실행의 #6981 terminal-cut 테스트에 LEAK 표시 1건이 있었다. 해당 테스트만 같은 target에서
  재실행하여 **1 passed / leak 표시 없음 / exit 0**을 확인했다. 첫 표시를 숨기거나 실패로 바꾸지 않는다.
- `cargo fmt --all -- --check`, manifest 및 unit-tier `--check --base-ref 263b61a64a77a0679e9d8679c5be2e1d180cee1a`: 모두 exit 0.
- Docker daemon 연결 불가를 확인했다. `CARGO_TARGET_DIR=target/pr7141-7175-7178-20260916
  scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/pr7141-7175-7178-wasm --no-opt`: exit 0.
  Chrome에서 새 패키지의 SVG와 render tree를 직접 얻었다. 최적화된 Docker 배포 빌드 통과로 표기하지 않는다.
- 전체 Rust/Native Skia 회귀는 사용자의 CI 중복 실행 생략 지시를 따랐다. 원 head의 정확한
  Full CI·별도 checks 성공을 확인했으며 **새 통합 head CI는 아직 실행하지 않았다**.
  이번 단계는 PR 검토다. 통합 PR 제출 전 Rust lint 3종 및 적용 게이트를 통과해야 한다.
- code/test/fixture/baseline 메인터너 보정은 추가하지 않았다. 탐색용 임시 IR probe는 정상 한컴
  입력 계약·수정 전 음성 대조가 확정되지 않아 수용 또는 신규 회귀 증거에서 제외했다.

## 공통 조판 원칙 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 측정·배치 일관성 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 분할·이어받기 계약 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 줄 소속과 점유 높이 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 사례와 증거의 독립성 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 기준값 변경 | 비해당 | 기존 진단의 자기서술·봉투 필드 보완이며 renderer·조판 알고리즘을 바꾸지 않음 |
| 주장과 검증 범위 | 충족 | 실제 CLI와 14개 계약 테스트, metadata/schema 매핑 확인 |

## 검증 입력 커밋 확인

**충족** — 아래 실행 파일은 통합 code commit의 blob과 byte 단위 동일하다.
기존 입력을 재사용했고 같은 내용의 HWP/HWPX/PDF 사본을 추가하지 않았다.
테스트가 메모리/임시 파일로 생성한 계약 입력은 해당 커밋의 test source로 재현한다.

| 경로 | SHA-256 |
| --- | --- |
| [samples/hwp3-sample.hwp](../../../samples/hwp3-sample.hwp) | `645525c8cd5ec11b1742ba7cfc759f68622861916233b5e982385cdb12f0ced2` |
| [samples/table_giant_cell_overfill.hwpx](../../../samples/table_giant_cell_overfill.hwpx) | `5d7eb4a21e46d9ad01a0f631eea2b1f2ec8a71750b4d448868e944e1b95042f4` |

## Merge 후 contributor PR comment 계획

통합 PR이 실제 merge된 뒤 원 PR에 통합 PR/merge SHA, 자기서술 보완 범위, 14개 계약 테스트와 최신 CI를 기록한다. #7061 전체 종료를 이 PR만으로 추정하지 않는다.

원격 comment는 merge 승인과 실제 병합 후 UTF-8 본문 파일을 `--body-file`로 게시하고 API로 재조회한다. 이번 검토에서는 reviewer 지정 외 원격 변경·push·comment·close·merge를 수행하지 않았다.
