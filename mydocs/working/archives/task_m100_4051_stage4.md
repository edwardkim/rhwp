# #4051 Stage 4 - 실제 CLI와 정적 검증 계획

## 목표

mock 계약뿐 아니라 실제 rhwp CLI의 feature-gate 종료 코드와 출력 생성 경계가 메인터너 보정에
정확히 연결됐는지 확인한다.

## 검증

1. `config.all-formats.json`과 native-skia 없는 rhwp 실행에서 PNG exit 2가 batch Failed/exit 1로
   반영되는지 확인한다.
2. 같은 stem의 HWP/HWPX 입력은 output root와 rhwp 호출 없이 거부되는지 확인한다.
3. `cargo fmt --check`, batch-convert clippy, `git diff --check`를 순차 실행한다.

## 테스트 결과

### 실제 rhwp feature-gate

`samples/field-01.hwp` 복사본 하나에 `config.all-formats.json`을 적용하고,
native-skia 없이 만든 `target/release-test/rhwp`를 `--rhwp-bin`으로 지정했다.

- `export-pdf`, `export-svg`, `export-text`는 산출물을 생성했다.
- `export-png`는 rhwp 종료 코드 2와 native-skia feature 필요 메시지를 반환했다.
- batch-convert는 PNG를 재시도하지 않았고, 최종 집계를 Successful 0, Failed 1, exit 1로
  반환했다. `collect_failed` 설정에 따라 원본은 `failed/field-01.hwp`로 복사됐다.

따라서 부분 산출물이 남았다는 사실만으로 성공을 보고하던 이전 동작은 제거됐다.

### 출력 충돌 사전 거부

동일 디렉터리에 `same.hwp`와 `same.hwpx`를 두고 mock rhwp를 지정했다.

- exit 1과 `출력 경로 충돌` 오류를 반환했다.
- mock rhwp 호출은 0회였다.
- output root는 생성되지 않았다.

### 회귀·정적 검사

| 검증 | 결과 |
| --- | --- |
| `CARGO_TARGET_DIR=target/review-kevin9327-4052-20260806 CARGO_INCREMENTAL=0 cargo test -p batch-convert` | 18개 통합 테스트 통과 |
| `cargo fmt --check` | 통과 |
| `CARGO_TARGET_DIR=target/review-kevin9327-4052-20260806 CARGO_INCREMENTAL=0 cargo clippy -p batch-convert --all-targets -- -D warnings` | 통과 |
| `git diff --check` | 통과 |

## 다음 단계

- 보정 코드·회귀 테스트·이 Stage 기록을 하나의 일반 commit으로 고정한다.
- 다음 문서 단계에서 PR #4052 검토 기록과 오늘할 일을 작성하고, 원격 source head·LFS 상태·push
  승인 조건을 다시 확인한다.
