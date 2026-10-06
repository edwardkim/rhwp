# #5874 Stage 3: 확장 검증

- Issue: #5874
- 구현 commit: `ce67652fe`.
- Stage 2의 실제 재현 문서와 focused 7건은 통과했다.
- 이 단계는 native/WASM/workspace lint와 전체 기본 nextest를 순차 수행한다.
  새 sample의 IR sweep/overflow-cell 검사는 전체 suite에 포함한다.
- 사용자 지시에 따라 검증 raw log와 실행 summary는 로컬 `output/issue5874/`에만 보관하고
  커밋하지 않는다. 문서와 이후 PR 본문에는 검증 항목과 통과 사실만 기재한다.
  Cargo는 공유 `target/pr-review`에서 동시에 실행하지 않으며, 이 호스트의
  16GiB 메모리를 고려해 컴파일 job은 2개로 제한한다. test threads는 표준 12개다.
- 직접 수정하지 않은 Skia feature 전용/브라우저 WASM 시각 검증과 GitHub CI 결과를
  기본 PDF 경계의 로컬 검증 결과와 혼동하지 않는다.

## 검증 결과

아래 항목은 모두 최종 exit code 0을 확인했다. 검증 중 로그가 잠시 assets에 생성된 경로를
즉시 `output/issue5874/`로 옮겼으며 이후 실행도 해당 로컬 경로를 사용하도록 수정했다.

| 항목 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| native Clippy, `-D warnings` | 통과 |
| WASM lib Clippy, `-D warnings` | 통과 |
| workspace build | 통과 |
| workspace/all-targets Clippy, `-D warnings` | 통과 |
| 파생 suite manifest `--check` | 통과 |
| 전체 기본 nextest | 9,051 통과, 46 skip, 실패 0; 409.994초 |

전체 회귀 명령은 다음과 같다. 위 시간은 테스트 구간이며, 컴파일을 포함한 전체 명령은
1,074.144초였다. 공유 `target/pr-review`의 산출물은 삭제하지 않았다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
```

- 새 계약 7건 외에 기존 PDF fallback, subset isolation, bold, subSVG 계약도 전체 회귀에 포함됐다.
- overflow-cell 16개 파티션이 통과했고, 새 dump를 합친 결과는 기존 기준선과 정확히 일치했다.
- IR field sweep은 통과했다. 실제 250행은 기준선 568행의 부분집합으로 신규/증가 0건,
  누락 318행이며 정확히 동일한 기준선이라고 주장하지 않는다. 새 `issue5874` 입력의 위반은 0건이다.
  이번 PDF 경계 수정과 무관한 기존 기준선은 변경하지 않았다.
- 설치된 nextest가 `profile.ci-duration-observation.junit.report-skipped` 설정을 인식하지
  않는 경고는 있었지만, 최종 테스트 summary와 exit code로 성공을 판정했다.
- 최초 확장 검증 시점에는 Native-Skia feature 전용 검증, wasm-pack/browser 시각 검증 및
  GitHub CI를 수행하지 않았다. PR 생성 승인 뒤 추가한 제출 검증은 아래에 따로 기록한다.

## PR 생성 승인 뒤 추가 제출 검증

동일한 source/test에서 다음 명령을 순차 실행해 모두 exit code 0을 확인했다.

| 항목 | 결과 |
| --- | --- |
| Native Skia `--lib` | 4,112 통과, 13 ignore, 실패 0 |
| `issue_2225_missing_picture_placeholder`, Native Skia | 2 통과 |
| `render_p37_direct_pdf_export`, Native Skia | 4 통과 |
| host WASM `--no-opt` 진단 빌드 | 통과, wasm-bindgen 출력 생성 |

```bash
CARGO_BUILD_JOBS=2 cargo test --locked --profile release-test --target-dir target/pr-review \
  --features native-skia --lib -- --test-threads 12
CARGO_BUILD_JOBS=2 node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_BUILD_JOBS=2 node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh \
  --target web --out-dir output/issue5874/wasm-pkg --no-opt
```

이 호스트에는 Docker가 없어 개발 환경 안내의 host 대체 진단 경로를 사용했다. 기존 `pkg/`는
덮어쓰지 않고 ignored output에 생성했다. 이는 최적화된 배포 빌드나 브라우저 시각 검증의
통과를 뜻하지 않는다. 변경 코드는 native-only 기본 PDF 경계이며, Native Skia 3종은 기존
경로의 회귀 검증일 뿐 direct PDF의 합성 기울임을 새로 지원했다는 뜻이 아니다.

`upstream/devel@bb42e5790`과의 merge simulation은 충돌 없이 통과했다. 해당 base 전진은 이번
PDF source/test와 Cargo 파일을 바꾸지 않았으므로 검증한 code history를 리베이스하지 않았다.

## 커밋 대상 정리

- 미공개 Stage 1/2 커밋에서도 raw log, 임시 JSON/SVG/개별 PNG, 중복 PDF를 제외했다.
  정리 전 검증한 버전과 현재 버전의 `src`, `tests`, `Cargo.toml`, `Cargo.lock`은 동일하다.
- assets에는 코멘트 본문에 표시할 `before-after-review.png`, `reporter-hancom.png`만 남겼다.
  원본 최소 HWPX와 비교용 `before.pdf`/`after.pdf`는 재현 및 코멘트 다운로드 링크용이다.
- 이후 PR/코멘트에는 검증 통과 사실과 필요한 이미지/입력/PDF 링크만 사용한다.
  로그나 임시 PNG/SVG/JSON 링크는 추가하지 않는다.
- 사용자가 PR 생성과 push를 승인했다. 최초 PR에는 오늘할일을 제외하고, PR 번호가 확정된 뒤
  self-review와 오늘할일 두 문서만 trailing commit으로 같은 source branch에 추가한다.
  GitHub CI 통과와 merge, 코멘트 게시, 이슈 종료는 아직 완료한 작업이 아니다.
