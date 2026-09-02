# Stage 7 완료 보고 — Task M100 #3789: 최신 기준 전체 회귀

- **일자**: 2026-08-28 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **기준**: `upstream/devel@5645e1f5b`
- **시작 head**: `a76e88085`
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **문서 성격**: Stage 7 종료 시점에 작성한 contemporaneous 보고

## 승인과 명령 기준

Stage 6의 재최신화·focused 결과를 공유한 뒤 작업지시자가 Stage 7 진행을 별도로 승인했다. 최신
upstream에는 착수 계획에서 사용한 `scripts/release-test.mjs`가 존재하지 않는다. 해당 호출은 테스트를
시작하기 전에 `MODULE_NOT_FOUND`로 종료됐다. 이를 제품 테스트 실패로 계산하지 않고 현재 권위 문서인
`mydocs/manual/pr_review/local_validation.md`의 직접 nextest 명령으로 교체했다.

```bash
cargo nextest run --locked \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --no-fail-fast

cargo clippy --locked --all-targets \
  --target-dir target/pr-review -- -D warnings
```

같은 checkout과 `target/pr-review`를 쓰는 두 Cargo 작업은 병렬 실행하지 않았다.

## 전체 회귀 결과

| 검증 | 결과 |
| --- | --- |
| release-test compile | 2분 19초 |
| nextest | 8,473/8,473 통과 |
| skip | 43 |
| slow | 10 |
| test 실행 시간 | 301.555초 |
| 필수 clippy | 통과, 경고 0 |

장시간 sample roundtrip, security corpus, injection scan, IR field sweep와 convert/verify corpus ratchet까지
모두 통과했다. `--no-fail-fast` 전체 실행에서 실패는 없었다.

## 추가 진단과 필수 게이트 판정

착수 구현계획에는 필수 범위보다 넓은
`cargo clippy --locked --workspace --all-targets --all-features ... -D warnings`가 적혀 있었다. 이 명령은
첫 실행에서 sandbox DNS 차단 때문에 `skia-bindings`를 받지 못했고, 네트워크를 허용해 다시 실행하자
최신 upstream의 GPU feature 조합에서 `vello 0.10::Scene`과 `vello_svg`가 사용하는
`vello 0.9::Scene` 타입 불일치로 컴파일되지 않았다.

#3789 diff에는 `Cargo.toml`, `Cargo.lock`, `src/renderer/gpu.rs` 변경이 없다. 현재 권위 문서가 요구하는
필수 명령은 `--all-targets`이며 `--all-features`가 아니다. 따라서 다음처럼 판정한다.

- #3789 필수 clippy 게이트: 통과
- 추가 all-features GPU 조합: upstream 별도 컴파일 문제 관찰, #3789 범위에서 수정하지 않음
- 삭제된 wrapper 호출: runner drift, canonical 직접 명령으로 교체

## 종료 판단과 다음 승인 게이트

최신 기준의 focused 검증과 전체 회귀, 필수 clippy가 모두 통과해 Stage 7을 완료로 판정한다. 기존
generated integration suite는 ignored 상태이며 제출 대상에 포함하지 않는다. 다음 단계는 remote push와
PR 생성이며 작업지시자의 별도 승인 전에는 수행하지 않는다.
