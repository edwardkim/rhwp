# PR #1250 처리 보고서 — 어울림 그림 및 미주 수식 배치 보정

- **작성일**: 2026-06-02
- **PR**: #1250
- **제목**: `task 1245: 어울림 그림 및 미주 수식 배치 보정`
- **컨트리뷰터**: @jangster77
- **연결 이슈**: #1245
- **검증 브랜치**: `local/pr1250-verify`
- **기준 브랜치**: `local/devel`
- **PR head**: `c5f5442934c64bf2d9d03cc0de365274383e0693`
- **검증 머지 커밋**: `855be98f`

## 1. 처리 요약

PR #1250을 현재 `local/devel` 기준 검증 브랜치에 병합했다.

GitHub 상태는 `MERGEABLE`이었고, 로컬 병합도 충돌 없이 완료되었다.

이번 PR의 핵심 보정:

- `Square/어울림` 그림의 line segment 기반 세로 위치를 첫 줄 대비 상대 delta로 계산
- 본문/미주 수식-only TAC 문단에서 TAC가 없는 선행 guide 줄을 배치 후보와 y advance에서 제외
- 본문/미주 수식-only 문단과 셀 내부 수식-only 문단의 정렬 기준 분리
- 미주 선두 번호가 prefix `TextRun`으로 이미 렌더된 경우 같은 위치의 `FootnoteMarker` 중복 생성 억제

## 2. 자동 검증

| 항목 | 결과 | 비고 |
|---|---|---|
| PR head CI `Build & Test` | 통과 | GitHub Actions |
| PR head CI `Render Diff` | 통과 | GitHub Actions |
| PR head CI `CodeQL` | 통과 | GitHub Actions |
| `git diff --check HEAD` | 통과 | whitespace/path 검증 |
| `cargo fmt --all --check` | 통과 | formatting |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 43 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과 | 4 passed |
| `cargo test --lib renderer::height_cursor` | 통과 | 29 passed |
| `cargo test --lib renderer::layout` | 통과 | 143 passed, 1 ignored |
| `cargo test --tests` | 통과 | 전체 integration 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 | WASM package 생성 |
| `npm run build` (`rhwp-studio`) | 통과 | Vite build 통과 |

`local/devel` 병합 후 재확인:

| 항목 | 결과 | 비고 |
|---|---|---|
| `git diff --check HEAD` | 통과 | whitespace/path 검증 |
| `cargo fmt --all --check` | 통과 | formatting |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 43 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과 | 4 passed |

## 3. 시각 판정 산출물

메인테이너 시각 판정 후보 SVG:

| file | 목적 | 판정 |
|---|---|---|
| `output/poc/pr1250-visual/page7/3-09월_교육_통합_2022_007.svg` | 2022년 9월 7쪽 `문25)`/`문28)` 어울림 그림 위치 | 통과 |
| `output/poc/pr1250-visual/page10/3-09월_교육_통합_2022_010.svg` | 2022년 9월 10쪽 `문12)` 우측 단 수식 위치 | 통과 |
| `output/poc/pr1250-visual/2023-page4/3-09월_교육_통합_2023_004.svg` | 2023년 9월 4쪽 `문26)` 중복 표시 제거 | 통과 |

웹 캔버스 판정용 WASM/Studio 빌드도 완료했다.

메인테이너 시각 판정:

```text
2026-06-02 통과
```

## 4. 현재 결론

자동 검증, WASM/Studio 빌드, 메인테이너 시각 판정을 모두 통과했다.

PR #1250은 수용 가능하며, `local/devel` 반영 후 원격 `devel`로 push한다.

## 5. 남은 절차

1. 본 보고서의 병합 후 재검증 결과를 커밋한다.
2. `origin/devel`로 push한다.
3. PR #1250에 메인테이너 코멘트를 남기고, 연결 이슈 #1245 종료 상태를 확인한다.
