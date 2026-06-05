# PR #1247 처리 보고서 — 미주 between-notes min-gap

- **작성일**: 2026-06-02
- **PR**: #1247
- **제목**: `Task #1246: 미주 between-notes margin min-gap (HeightCursor) — closes #1238, #1246`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1238, #1246
- **검증 브랜치**: `local/pr1247-verify`
- **기준 브랜치**: `local/devel`
- **PR head**: `8e5493a096f3f28213c9cd69dbe1fd09237647b5`
- **검증 머지 커밋**: `58619373`

## 1. 처리 요약

PR #1247을 현재 `local/devel` 기준 검증 브랜치에 병합했다.

GitHub상 PR은 stale base 때문에 `CONFLICTING` 상태였고, 실제 충돌은 `src/renderer/layout.rs` 한 곳에서 발생했다.

충돌 해소 방향:

- PR #1240에서 추가된 `endnote_para_has_same_endnote_successor()` 유지
- PR #1247에서 추가된 `endnote_between_notes_hu` 필드와 `set_endnote_between_notes_hu()` 유지
- 미주 flow column 생성 시 `HeightCursor.endnote_between_notes_hu` 주입 유지

즉, 같은 코드 영역에 들어간 두 기능을 모두 보존했다.

## 2. 자동 검증

| 항목 | 결과 | 비고 |
|---|---|---|
| `git diff --check HEAD` | 통과 | whitespace/path 검증 |
| `cargo fmt --all --check` | 통과 | formatting |
| `cargo test renderer::height_cursor --lib` | 통과 | 29 passed |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 41 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과 | 4 passed |
| `cargo test --lib` | 통과 | 1527 passed, 6 ignored |
| `cargo test --tests` | 통과 | integration 전체 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 | WASM package 생성 |
| `npm run build` (`rhwp-studio`) | 통과 | Vite build 통과 |

`local/devel` 병합 후 재확인:

| 항목 | 결과 | 비고 |
|---|---|---|
| `cargo test renderer::height_cursor --lib` | 통과 | 29 passed |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 41 passed |

## 3. 시각 판정 산출물

메인테이너 시각 판정 결과:

| file | 목적 | 판정 |
|---|---|---|
| `output/poc/pr1247-endnote-min-gap/page14/3-11월_실전_통합_2022_014.svg` | 핵심 케이스, page14 문22 미주 간격 | 통과 |
| `output/poc/pr1247-endnote-min-gap/page14-debug/3-11월_실전_통합_2022_014.svg` | 핵심 케이스 debug/grid | 통과 |
| `output/poc/pr1247-endnote-min-gap/page10/3-11월_실전_통합_2022_010.svg` | 11월 회귀 가드 | 통과 |
| `output/poc/pr1247-endnote-min-gap/page17/3-11월_실전_통합_2022_017.svg` | 11월 회귀 가드 | 통과 |
| `output/poc/pr1247-endnote-min-gap/guard-sep-page10/3-09월_교육_통합_2022_010.svg` | 9월 회귀 가드 | 통과 |
| `output/poc/pr1247-endnote-min-gap/guard-oct-page11/3-10월_교육_통합_2022_011.svg` | 10월 회귀 가드 | 통과 |

참고: `guard-oct-page11` 산출 중 기존 레이아웃 overflow 진단 로그가 출력되었지만 SVG export는 정상 완료되었다.

메인테이너 시각 판정:

```text
2026-06-02 통과
```

## 4. 남은 절차

1. 본 보고서와 검토 문서를 커밋한다.
2. `local/pr1247-verify`를 `local/devel`에 병합한다.
3. `local/devel` 기준 테스트 후 `origin/devel`로 push한다.
4. PR #1247에 메인테이너 코멘트를 남기고, 연결 이슈 #1238/#1246 종료 상태를 확인한다.

## 5. 현재 결론

자동 검증과 메인테이너 시각 판정을 모두 통과했다.

PR #1247은 수용 가능하며, `local/devel` 반영 후 원격 `devel`로 push한다.
