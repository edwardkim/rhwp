# PR #2128 리뷰 문서

## 1. 메타

| 항목 | 내용 |
|---|---|
| PR | #2128 |
| 제목 | task 2099: U+F53A 아래아 렌더 복구 |
| 작성자 | jangster77 |
| base | devel |
| head | task/m100-2099-araea-pua |
| 관련 이슈 | Closes #2099 |
| 작성 시점 참고값 | mergeable=MERGEABLE, 초기 코드 검증 기준 SHA=dfdfaf689d7607bc77fe1b438f12bb858177a918 |
| 규모 | Stage 2 보강 포함, merge 전 최신 diff 재확인 필요 |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.
위 mergeable, SHA, 규모 값은 작성 시점 참고값이며 merge 전 재확인한다.

## 2. 관련 이슈 요약

#2099는 Linux 환경에서 `한글문서파일형식_5.0_revision1.3` 문서의 `아래아`가 포함된 `글` 표기가
깨지거나 보이지 않는 문제다.

로컬에서 대응 가능한 기준 자료는 다음과 같다.

- `samples/hwpspec.hwp`: 178쪽 HWP5 배포용 원본
- `pdf/hwpspec-2024.pdf`: 기존 178쪽 기준 PDF
- `samples/pua-test.hwp` / `samples/pua-test.pdf`: PUA 회귀 검증 fixture
- `samples/한글문서파일형식_5.0_revision1.3.hwp`: 사용자 재검증 요청 68쪽 배포본

## 3. 변경 범위

핵심 변경은 다음과 같다.

- `src/renderer/pua_oldhangul.rs`에서 U+F53A를 `ᄒᆞᆫ` 자모 시퀀스로 다시 매핑한다.
- 이전 #615의 "매핑 표 외 공백" 판단은 보존된 `samples/pua-test.pdf`와 hwpspec 실제 본문 확인 결과에 맞춰
  #2099 기준으로 정정한다.
- `tests/issue_2099_araea_pua.rs`를 추가해 `samples/hwpspec.hwp`와 `samples/pua-test.hwp` SVG 출력에
  raw U+F53A가 남지 않고 `ᄒᆞᆫ`으로 확장되는지 고정한다.
- 사용자 재검증 파일 `samples/한글문서파일형식_5.0_revision1.3.hwp`를 회귀 fixture로 포함하고,
  1쪽 제목의 `ᄒᆞᆫ` 클러스터가 `Source Han Serif K Old Hangul`을 font-family 최우선으로 쓰는지 고정한다.
- 브라우저 Canvas 경로에서도 옛한글 자모 클러스터만 `Source Han Serif K Old Hangul` 우선 폰트로 렌더한다.
- `rhwp-studio/src/core/font-loader.ts`에 `한컴산뜻돋움` 매핑을 추가해 사용자 재현 파일의 영문 라인 폭 겹침을 줄인다.
- `mydocs/working/task_m100_2099_stage1.md`, `mydocs/working/task_m100_2099_stage2.md`에 분석 및 검증 계획을 기록한다.

## 4. 렌더 영향 및 시각 검증

이 PR은 페이지 배치나 pagination이 아니라 PUA 옛한글 텍스트 확장 및 브라우저 폰트 매칭 경로를 바꾸는 PR이다.
따라서 HWP 2020 MCP로 새 기준 PDF를 만들지는 않았다. 기존 저장소 기준 PDF와 fixture PDF를 근거로 U+F53A가
실제 `` 글리프로 보여야 함을 확인했고, SVG 렌더 결과에서 raw PUA가 사라지는지 테스트로 고정했다.

사용자 재검증 요청에 따라 `samples/한글문서파일형식_5.0_revision1.3.hwp`도 확인했다.

- `target/debug/rhwp info`: 68쪽, distribution 문서 확인.
- `export-text`: 1쪽 제목 첫 글자가 raw U+F53A ``로 저장되어 있음을 확인.
- `export-svg`: raw U+F53A 미출력, `ᄒᆞᆫ` 출력, 해당 클러스터 font-family 최우선이
  `Source Han Serif K Old Hangul`임을 확인.
- `localhost:7700` 브라우저 검증: 68쪽 로드, `한컴산뜻돋움`/`Source Han Serif K Old Hangul` loaded,
  콘솔 error/warn 없음, 제목/영문 라인 시각 확인.
- 증적: `mydocs/pr/assets/pr_2128_issue2099_revision13_browser_title_crop.png`

검증 fixture SHA-256은 다음과 같다.

| 파일 | SHA-256 |
|---|---|
| `samples/hwpspec.hwp` | `64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d` |
| `samples/pua-test.hwp` | `592e00e3e8bc72afef829fd71a13cbc340bff3770425b9d1ee025afc1e1649fa` |
| `samples/한글문서파일형식_5.0_revision1.3.hwp` | `f21edf2138e134702366f2fb6a2ab082b05c6dcb42216ec4fa2575ed292efd1d` |

## 5. 검증 결과

최종 확인한 로컬 검증은 다음과 같다.

- `cargo test --test issue_2099_araea_pua` 통과
- `cargo test --lib pua` 통과
- `cargo test --test issue_1086` 통과
- `cargo fmt --check` 통과
- `git diff --check` 통과
- `cargo clippy --all-targets -- -D warnings` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `npm run build` (`rhwp-studio`) 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과
- `localhost:7700` Puppeteer headless 브라우저 검증 통과
  - `samples/한글문서파일형식_5.0_revision1.3.hwp` 로드 결과 68쪽
  - `한컴산뜻돋움` loaded
  - `Source Han Serif K Old Hangul` loaded
  - 콘솔 error/warn 0건

WASM build 후 추적 대상 `pkg/` 변경은 없었다.

## 6. 리스크 및 잔여 확인

- 이번 PR은 U+F53A 단일 코드포인트의 옛한글 확장 복구와 사용자 재검증 파일의 브라우저 폰트 매칭 보강에 한정한다.
- 한컴 PDF와 픽셀 단위 동일한 폰트 fidelity는 여전히 별도 축이다. 다만 사용자 재검증 파일의 대표 제목/영문 라인은
  브라우저 경로에서 겹침 없이 표시됨을 확인했다.
- merge 전 GitHub Actions 최신 head 결과를 재확인한다.

## 7. 최종 권고

로컬 집중 테스트, 관련 PUA 회귀 테스트, 기존 hwpspec 페이지 수 회귀, clippy, WASM build, 7700 브라우저 검증을
통과했고, #2099의 아래아 표시 누락 핵심 원인을 U+F53A raw PUA 출력 및 옛한글 폰트 우선순위 문제로 좁혀 수정했다.
PR #2128은 옵션 1 경로로 review 문서와 오늘할일 갱신을 같은 PR head에 포함한 뒤 GitHub Actions 통과와
작업지시자 승인 조건으로 merge 가능 후보로 본다.
