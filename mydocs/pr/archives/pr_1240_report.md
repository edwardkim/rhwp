# PR #1240 처리 보고서

- **작성일**: 2026-06-02
- **PR**: #1240
- **제목**: `미주(해설) 다줄 문단 줄간격 간헐적 좁음 수정 (closes #1236)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1236
- **검증 브랜치**: `local/pr1240-verify`
- **기준 브랜치**: `local/devel` (`704f62d2`)
- **PR head**: `b6c9a4df7ec043be59d144b5909a993cabb2734f`
- **검증 병합 커밋**: `1b3fc307 Merge PR 1240 verification`

## 1. 처리 요약

PR #1240을 현재 `local/devel` 기준 검증 브랜치에 병합했다.

변경 핵심은 미주 가상 문단에서 다줄 문단의 마지막 줄 뒤 `line_spacing`을 조건부 복원하는 것이다. 다음 가상 미주 문단이 같은 원본 Endnote control의 연속 문단인 경우에만 trailing spacing을 적용해, 같은 해설 내부 문단 간격은 복원하고 다른 문제 경계의 중복 간격은 피하는 방향이다.

## 2. 반영 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout.rs` | `endnote_para_has_same_endnote_successor()` 추가 |
| `src/renderer/layout/paragraph_layout.rs` | 미주 다줄 문단 마지막 줄 간격 조건 보정 |
| `mydocs/plans/task_m100_1236*.md` | PR 작성자의 계획 문서 |
| `mydocs/working/task_m100_1236_stage*.md` | PR 작성자의 단계 기록 |
| `mydocs/tech/endnote_line_spacing_1236.md` | 미주 줄간격 조사 문서 |
| `mydocs/report/task_m100_1236_report.md` | PR 작성자의 완료 보고서 |

## 3. 자동 검증

| 항목 | 결과 |
|---|---|
| `git diff --check HEAD` | 통과 |
| `cargo fmt --all --check` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate` | 통과, 41 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 4 passed |
| `cargo test --test issue_1156_rowbreak_fragment_fit` | 통과, 3 passed |
| `cargo test --lib` | 통과, 1524 passed / 6 ignored |
| `cargo test --tests` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| `rhwp-studio npm run build` | 통과 |

`cargo test --tests`에서는 `issue_1139`, `issue_1189`, `issue_1209`, `issue_1156`, `svg_snapshot` 등 미주/페이지네이션/렌더링 회귀 테스트가 통과했다.

## 4. 시각 판정 자료

샘플:

```text
samples/3-11월_실전_통합_2022.hwpx
```

한컴 페이지 기준 10/11/12/14쪽을 CLI 0-index `-p 9/10/11/13`으로 내보냈다.

| 페이지 | SVG | 판정 |
|---|---|---|
| 10쪽 | `output/poc/pr1240-endnote-line-spacing/3-11월_실전_통합_2022_010.svg` | 통과 |
| 11쪽 | `output/poc/pr1240-endnote-line-spacing/3-11월_실전_통합_2022_011.svg` | 통과 |
| 12쪽 | `output/poc/pr1240-endnote-line-spacing/3-11월_실전_통합_2022_012.svg` | 통과 |
| 14쪽 | `output/poc/pr1240-endnote-line-spacing/3-11월_실전_통합_2022_014.svg` | 통과 |

디버그/격자 SVG:

| 페이지 | SVG |
|---|---|
| 10쪽 | `output/poc/pr1240-endnote-line-spacing-debug/3-11월_실전_통합_2022_010.svg` |
| 11쪽 | `output/poc/pr1240-endnote-line-spacing-debug/3-11월_실전_통합_2022_011.svg` |
| 12쪽 | `output/poc/pr1240-endnote-line-spacing-debug/3-11월_실전_통합_2022_012.svg` |
| 14쪽 | `output/poc/pr1240-endnote-line-spacing-debug/3-11월_실전_통합_2022_014.svg` |

## 5. 관찰 사항

11쪽 SVG 산출 중 다음 레이아웃 경고가 일반/디버그 SVG에서 동일하게 1회 발생했다.

```text
LAYOUT_OVERFLOW_DRAW: section=0 pi=537 line=0 y=2622.4 col_bottom=1092.3 overflow=1530.2px
LAYOUT_OVERFLOW: page=10, sec=0, col=1, para=537, type=FullParagraph, first=false, y=2622.4, bottom=1092.3, overflow=1530.2px
```

명령은 정상 종료했고 SVG도 생성되었다. 다만 미주/페이지네이션 영역의 잔여 경고이므로 메인테이너 시각 판정 시 11쪽도 반드시 확인해야 한다.

## 6. 현재 판정

자동 검증과 빌드는 통과했다.

메인테이너 SVG/웹 캔버스 시각 판정도 통과했다.

```text
2026-06-02 통과
```

## 7. 다음 절차

1. `local/devel`에 병합
2. 원격 `devel` push
3. PR #1240 및 이슈 #1236 종료 상태 확인
