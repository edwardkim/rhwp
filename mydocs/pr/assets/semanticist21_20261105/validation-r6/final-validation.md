# 통합 후보 최종 로컬 검증

기록 시각: 2026-11-06T11:08:53+09:00. 원 PR 18건·원 커밋 44개를 한 번씩 누적한 검토다.
고정 base `170b886f167b705b6e630c5b34bb638d92fae0a9`, Rust 제품 source
`86709ac16711820e61baa3b87e2c4ddaac23573e`, 최종 Studio 코드 `5feccb130e0f695de0046d994c060130fa3e928e`.
최종 문서·증적 commit은 생산 코드의 실행 source와 구분한다. 원격 CI·merge는 로컬 완료와 별도다.

## 전체 Rust의 실제 실행과 재사용

- 전체 실행 head `dab38dd4b786a71377f93d6baf90335fdc14b8b1`. 실제 55개 binary의 발견 12,785건 중 eligible 12,740건 모두 PASS,
  기존 ignored 45건, 누락·예상 밖 이름·중복 0건이다. 실제 종료 코드는 모두 0이다.
- 변경하지 않은 0.5초 성능 검사는 조용한 환경에서 1건 먼저 실행했다. 그 정확한 보집합을
  `-E 'not test(inflated_row_count_does_not_slow_down_parsing)'`로 실행해 12,739건 PASS를 확인했다.
  이 보집합의 skipped 46은 기존 ignored 45와 먼저 실행한 1건이다.
- 전체 회귀 이후 Rust 생산 코드·Cargo 입력은 같으며, 테스트 변경은 `note_separator_declared_width.rs`
  첫머리의 문서 링크뿐이다. 모듈 문서 주석을 제외한 내용이 같음을 대조했다.
- 후속 helper가 이전 파생 harness와 다른 suite를 선택해 한 차례 exit 4·0건 실행으로 멈췄다.
  이를 PASS로 처리하지 않고 `--prepare`로 정상화한 뒤 실제 2건을 실행해 PASS했다.
  전후 grouped 2,072개 원본 파일의 경로·SHA가 모두 같으며 추가·삭제·bytes 변경 0이다.
  검증 기대값·허용 오차를 바꾸지 않았다. 파생 harness·manifest는 PR에 stage하지 않는다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast \
  -E 'test(inflated_row_count_does_not_slow_down_parsing)'
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast \
  -E 'not test(inflated_row_count_does_not_slow_down_parsing)'
```

## 최소 글꼴과 공통 실행 환경

`CARGO_BUILD_JOBS=8`, 공용 `target/pr-review`, 테스트 스레드 12다. Cargo 명령은 순차 실행했다.
기본 증분 설정을 유지하고 `CARGO_INCREMENTAL=0`을 지정하지 않았다.
[최소 글꼴 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pr_review/local_validation.md#회귀-테스트의-최소-글꼴-환경)에
따라 bwrap에서 `/usr/share/fonts`·`/usr/local/share/fonts`를 숨기고 다음 공개 자원만 노출했다.
`RHWP_FONT_PATH`는 unset이며 `fc-list` 실제 결과도 아래 5개와 같다. 비공개 설치 OTF에 의존하지 않는다.

- `/home/tsjang/rhwp/ttfs/opensource/NotoSansKR-ExtraLight.ttf`
- `/home/tsjang/rhwp/ttfs/opensource/NotoSansKR-Regular.ttf`
- `/home/tsjang/rhwp/ttfs/opensource/NotoSansSymbols2-Regular.ttf`
- `/home/tsjang/rhwp/ttfs/opensource/RHWPOldHangul-Regular.otf`
- `/home/tsjang/rhwp/ttfs/opensource/RHWPPublicBatang-Regular.ttf`

## Cargo 검증 결과

아래는 실제 최종 유효 실행이다. 새 harness all-target lint도 추가 완료했다.
명령 앞의 최소 글꼴 wrapper는 위 namespace·fontconfig 조건을 적용하며 Cargo 옵션을 바꾸지 않는다.

| 검사 | 실제 종료 | 명령 |
| --- | ---: | --- |
| r7-format-check | 0 | `cargo fmt --all -- --check` |
| r7-native-clippy | 0 | `cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` |
| r7-wasm-clippy | 0 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` |
| r7-workspace-build | 0 | `cargo build --locked --workspace --target-dir /home/tsjang/rhwp/target/pr-review` |
| r7-workspace-all-target-clippy | 0 | `cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` |
| r7-native-skia-lib | 0 | `cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads=12` |
| r8-workspace-all-target-clippy | 0 | `cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` |
| r8-native-skia-placeholder | 0 | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` |
| r8-native-skia-direct-pdf | 0 | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` |
| r8-release-build | 0 | `cargo build --locked --release --target-dir /home/tsjang/rhwp/target/pr-review` |
| r8-release-lib | 0 | `cargo test --locked --release --lib --target-dir /home/tsjang/rhwp/target/pr-review -- --test-threads=12` |
| r8-doc-tests | 0 | `cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review` |
| r8-suite-manifest-check | 0 | `node scripts/rust-test-suite-manifest.mjs --check --base-ref 170b886f167b705b6e630c5b34bb638d92fae0a9` |
| r8-unit-tiers-tests | 0 | `node --test scripts/tests/rust-unit-test-tiers.test.mjs` |
| r8-unit-tiers-check | 0 | `node scripts/rust-unit-test-tiers.mjs --check --base-ref 170b886f167b705b6e630c5b34bb638d92fae0a9` |

Native Skia lib 2건·누락 그림 2건·직접 PDF 22건 PASS다. doctest는 4개 crate 요약에서 총 8건 PASS·3건 ignored·실패 0건이다. 현재 base의 기존 lib 검사는
integration suite로 이관되어 있으며 과거 lib 검사 수를 현재 실행으로 재표기하지 않았다.

## Studio·WASM·실제 backend

최종 Studio 전체 Node 2,744건 PASS·실패/취소/skip 0, TypeScript·Vite production build exit 0이다.
기존 최적화 WASM bytes를 실제 공개 API 12그룹과 각 브라우저 network response에서 확인했다.
실제 30개 저장 HWP/HWPX와 독립 CFB·필드 문맥·타입 631개×2·소비 10건의 근거는
[동작 검사](runtime-checks.md)에 연결한다.

실제 Canvas2D·CanvasKit compatibility/default × screen/print 12캡처, 웹폰트 실제 브라우저
7시나리오, 최소 글꼴 실제 PageRenderer 관계 검사도 각각 exit 0이다. 겹침·덧말 같은 원본 1쪽의
Studio Canvas2D raw는 두 문서 모두 100.00%이며 [최종 review·overlay](../studio-symbol/README.md)를 보존했다.
새 글꼴 파일·다운로드 URL·초기 필수 공급 목록을 추가하지 않았다.

## 전쪽 시각·새 base·보호 PR

- Native/fresh WASM 각 128입력·301쪽, 전쪽 90% 이상·누락 0·독립 PDF 쪽수 일치.
  Native 최저 91.81401%, fresh WASM 최저 91.17272%. [원본·PDF SHA·전체 TSV](measurements.md).
- 같은 base와 각 301쪽을 대조해 raw 하락 자체가 0쪽이다. [전쪽 base 대조](base-comparison.md).
- 공식 Native 편람 HWPX 383쪽 gate: min 99.96%, 공개 기준 하락 0·exit 0.
  `5172f2d55c`에서 실행했으며 이후 Rust 생산 코드·입력·PDF·측정 도구·binary가 같음을 확인해 재사용했다.
- 보호 PR #8389·#8397: 각 17개 고유 문서 1770쪽, 공개 기준의 중복 행 포함 2153행을 대조해
  표시값 하락·누락·오류 0. [현재 전체 TSV 34개·대조](protected/README.md).
- 시각 검증은 전체 시스템 1,906 face와 승인 대체를 공급했다. 최소 글꼴 회귀 환경을 시각 점수 비교에 쓰지 않았다.
- 원본·PDF·측정 도구·기대값을 조정해 점수를 올리지 않았다. 실제 field 파일명 재평가 차이는
  [PR #8564 문맥](runtime-checks.md)에 별도로 설명한다. 점수 100%를 RGB 동일이나 내용 전수 동일로 표현하지 않는다.

## 제출·후속 처리 경계

최종 대표 PNG·전체 원문 TSV·검증 HWP/HWPX·독립 PDF를 정식 경로에 보존했다.
raw build/test 로그와 metric/run/MCP JSON은 output의 중간 자료이며 커밋하지 않는다.
원 PR 18개 리뷰는 실제 주장과 같은 문서·페이지·TSV·review/overlay를 각각 연결한다.
원 PR close와 Issue #8019·#8628·#7635는 통합 CI·merge·devel 포함과 해결 범위를 확인한 뒤 처리한다.
