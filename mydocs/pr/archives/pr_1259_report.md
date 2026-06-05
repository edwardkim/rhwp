# PR #1259 처리 보고서 - 미주 답안 제목 between-notes 7mm 갭 복원

- **작성일**: 2026-06-03
- **PR**: #1259
- **연결 이슈**: #1256
- **컨트리뷰터**: @planet6897
- **검증 브랜치**: `local/pr1259-current`
- **PR head**: `9b8d1d90c3467d7158fabe42ec97fd65f8bfda76`
- **검증 적용 커밋**: `04e9165c`
- **기준 devel**: `5137c07f`

## 1. 처리 요약

PR #1259를 현재 `devel` 기준 검증 브랜치에 체리픽해 검증했다. 충돌은 없었다.

수용 범위는 다음이다.

- 미주 답안 `문N)` 제목 위의 `between-notes` 7mm 간격이 `height_cursor`의 vpos 되감김 보정에 의해 사라지는 문제 보정
- 단일 줄 prev + injected between-notes + 미주 제목 + 실제 cram 상황에서만 `y_offset` 유지
- 후속 미주 항목 desync 방지를 위한 active vpos base 이동
- 관련 height_cursor 단위 테스트 2건 추가
- `tests/issue_1139_inline_picture_duplicate.rs`의 문12 위치 단언을 #1256 PDF 정합 기준으로 갱신

## 2. 자동 검증 결과

| 항목 | 명령 | 결과 |
|---|---|---|
| whitespace | `git diff --check devel..HEAD` | 통과 |
| Rust fmt | `cargo fmt --all --check` | 통과 |
| 신규 height_cursor 테스트 | `cargo test compact_endnote_between_notes --lib` | 통과 |
| #1256 통합 테스트 | `cargo test issue_1256_2022_sep_page10_question12_keeps_between_notes_gap --test issue_1139_inline_picture_duplicate` | 통과 |
| 전체 integration | `cargo test --tests` | 통과 |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 통과 |
| WASM build | `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| Studio build | `npm run build --prefix rhwp-studio` | 통과 |

## 3. 시각 판정

메인테이너가 다음 후보 SVG와 rhwp-studio WASM 산출물을 기준으로 시각 판정했고 통과했다.

| 대상 | page | 판정 |
|---|---:|---|
| `samples/3-09월_교육_통합_2022.hwp` | 9, 10, 13, 14, 18 | 통과 |
| `samples/3-09월_교육_통합_2022.hwpx` | 9, 10, 13, 14, 18 | 통과 |
| rhwp-studio WASM 산출물 | 해당 없음 | 통과 |

생성 파일:

```text
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_009.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_010.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_013.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_014.svg
output/poc/pr1259-visual/hwp/3-09월_교육_통합_2022_018.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_009.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_010.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_013.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_014.svg
output/poc/pr1259-visual/hwpx/3-09월_교육_통합_2022_018.svg
```

판정:

```text
2026-06-03 통과
```

## 4. WASM 동기화

Docker WASM 빌드 후 `pkg/` 산출물을 `rhwp-studio/public/`에 동기화했다.

`rhwp.js`, `rhwp_bg.wasm`, `rhwp.d.ts`는 `pkg/`와 `rhwp-studio/public/` 해시가 일치하고 Git diff는 없었다.

`rhwp-studio/public/rhwp_bg.wasm.d.ts`는 이전 public 파일이 stale 상태였기 때문에 최신 wasm-bindgen export 목록으로 갱신되었다.

## 5. 잔여 및 후속

PR 본문이 명시한 것처럼 컬럼 하단 제목과 일부 mid-column 부분 갭은 본 PR 범위 밖이며 후속 #1257에서 다룬다.

## 6. devel 병합 후 재검증

`local/pr1259-current`를 `devel`에 `--no-ff`로 병합했다.

| 항목 | 명령 | 결과 |
|---|---|---|
| Rust fmt | `cargo fmt --all --check` | 통과 |
| 신규 height_cursor 테스트 | `cargo test compact_endnote_between_notes --lib` | 통과 |
| #1256 통합 테스트 | `cargo test issue_1256_2022_sep_page10_question12_keeps_between_notes_gap --test issue_1139_inline_picture_duplicate` | 통과 |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 통과 |

## 7. 최종 판단

자동 검증, WASM/Studio build, 메인테이너 시각 판정이 모두 통과했으므로 PR #1259를 수용한다.

남은 절차:

1. `local/pr1259-current` 변경 커밋 — 완료
2. `devel` 병합 — 완료
3. `devel` 기준 검증 재확인 — 완료
4. `origin/devel` push — 진행 예정
5. PR #1259 및 이슈 #1256 종료 처리 — 진행 예정
