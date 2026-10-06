# PR #1334 검토 - 단 구분선 중복 렌더 제거 (#1333)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1334
- 작성자: `planet6897`
- 상태: open / draft 아님
- base: `devel`
- head: `fix/1333-column-separator` (`785de686`)
- 관련 이슈: #1333

## 2. 변경 요약

문제:

- 페이지 전체가 단일 다단인 문서에서 단 구분선이 두 경로로 중복 렌더됨.
- page-level 경로는 body 전체 높이를 기준으로 구분선을 그려, 섹션 끝/부분 페이지에서 구분선이 과도하게 길어짐.

컨트리뷰터 수정:

- page-level `build_column_separators` 호출과 함수를 제거.
- `build_columns` 내부 `emit_zone_column_separators` 경로를 단 구분선의 단일 렌더 경로로 사용.
- `emit_zone_column_separators`에서 `y_end`를 body 영역 하단으로 cap하여 꽉 찬 페이지의 하단 초과 렌더를 방지.

메인테이너 정리:

- 제거된 `build_column_separators`를 기준으로 설명하던 주석 2곳을 현재 구조에 맞게 갱신.
- 메인테이너 시각 판정에서 3쪽 단 구분선이 본문 하단까지 그려지지 않는 회귀가 발견됨.
- `zone_layout=None`인 페이지 기본 다단은 body 전체 높이로 그리고, `zone_layout=Some`인 페이지 내부 zone 전환만 콘텐츠 높이를 쓰도록 분기 보강.

## 3. GitHub 상태

- PR head: `785de6863befb049da0aed28c3c9d50d81a9e81f`
- PR base SHA: `1b286dded93c63481b7ee6c736edca7fc8c1c41b`
- 현재 로컬 `devel`: `6f9ddbab`
- PR base가 현재 `devel`보다 오래되어, `local/devel..local/pr1334-upstream` 직접 diff는 최근 PR들의 변경을 되돌리는 것처럼 크게 보인다.
- 실제 검토는 현재 `local/devel`에서 `local/pr1334-merge-test` 브랜치를 만들고 PR 단일 커밋을 cherry-pick하여 진행했다.

GitHub Actions:

- Analyze JS/Python/Rust: success
- Build & Test: success
- Canvas visual diff: success
- CodeQL: success
- WASM Build: skipped

## 4. 로컬 적용 방식

검토 브랜치:

```text
local/pr1334-merge-test
```

적용 커밋:

```text
91c5cebe fix(layout): 단 구분선을 콘텐츠 높이로 단일 렌더 (#1333)
```

현재 `local/devel` 기준 실제 변경 범위:

- `src/renderer/layout.rs`
- contributor 작업 문서 7개
  - `mydocs/plans/task_m100_1333.md`
  - `mydocs/plans/task_m100_1333_impl.md`
  - `mydocs/plans/task_m100_1333_v2.md`
  - `mydocs/report/task_m100_1333_report.md`
  - `mydocs/working/task_m100_1333_stage1.md`
  - `mydocs/working/task_m100_1333_stage2.md`
  - `mydocs/working/task_m100_1333_v2_stage1.md`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --test issue_702
cargo test --test issue_874_ktx_toc_page_number_right_align
cargo test --test svg_snapshot
cargo test --lib
cargo clippy --all-targets -- -D warnings
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp' -o output/poc/pr1334-column-separator -p 2 --debug-overlay
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp' -o output/poc/pr1334-column-separator -p 21 --debug-overlay
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp' -o output/poc/pr1334-column-separator-fixed -p 2 --debug-overlay
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp' -o output/poc/pr1334-column-separator-fixed -p 21 --debug-overlay
```

결과:

- `cargo fmt --all -- --check`: pass
- `cargo test --test issue_702`: 2 passed
- `cargo test --test issue_874_ktx_toc_page_number_right_align`: 1 passed
- `cargo test --test svg_snapshot`: 8 passed
- `cargo test --lib`: 1615 passed, 0 failed, 6 ignored
- `cargo clippy --all-targets -- -D warnings`: pass

검증 산출물:

- `output/poc/pr1334-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_003.svg`
- `output/poc/pr1334-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_022.svg`
- `output/poc/pr1334-column-separator-fixed/3-09월_교육_통합_2024-구분선아래20구분선위20_003.svg`
- `output/poc/pr1334-column-separator-fixed/3-09월_교육_통합_2024-구분선아래20구분선위20_022.svg`

메모:

- 초기 산출물 3쪽에서 단 구분선이 `y=821.76`에서 끊겼다. 수정 후 `y=90.7067 -> 1092.2667`로 body 하단까지 렌더됨을 확인했다.
- 22쪽 SVG export 중 `LAYOUT_OVERFLOW` 로그가 일부 출력되었지만 export 자체는 성공했고, 수정 후 단 구분선은 body 하단에서 cap된다.

## 6. 검토 의견

핵심 방향은 타당하다.

- 단 구분선 SSOT를 `emit_zone_column_separators` 하나로 좁혀 중복 렌더 원인을 제거한다.
- 다만 `zone_layout.unwrap_or(layout)` 경로에 콘텐츠 높이 기준을 그대로 적용하면, 페이지 기본 다단에서도 구분선이 중간에서 끊긴다.
- 따라서 page layout fallback은 body 전체 높이를 유지하고, 실제 zone-specific layout만 콘텐츠 높이를 사용해야 한다.
- body bottom cap은 꽉 찬 페이지의 하단 초과를 제한하는 보수적인 방어로 유지한다.

주의점:

- PR upstream branch는 현재 `devel`보다 오래된 base 위에 있으므로 GitHub 기본 merge보다 maintainer-side cherry-pick/통합이 안전하다.
- contributor 작업 문서 7개는 `mydocs/plans`, `mydocs/report`, `mydocs/working` 정리 정책과 충돌할 수 있다. 최종 통합 시 코드 변경과 메인테이너 리뷰 문서만 남기고 contributor 작업 문서는 제외하는 방안을 권장한다.
- 렌더 영향 PR이므로 메인테이너 SVG/WASM 시각 판정을 수행했고 통과했다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. `local/devel` 기준 PR 코드 커밋 + 메인테이너 보정 커밋을 통합.
2. contributor 작업 문서는 최종 커밋에서 제외하거나 archive 정책에 맞춰 별도 판단.
3. 최종 검증 후 `devel` push.
4. PR #1334에 검증 결과와 수용 코멘트 작성.
5. 이슈 #1333 close 확인.

## 8. PR 코멘트 초안

```markdown
검토 완료했습니다.

단 구분선이 page-level 경로와 zone emit 경로에서 중복 렌더되던 원인을 `emit_zone_column_separators` 단일 경로로 정리한 방향은 타당하다고 판단했습니다. `y_end`를 body 하단으로 제한하는 처리도 부분 페이지에서 구분선이 과도하게 길어지는 문제를 줄이는 보수적인 수정으로 확인했습니다.

다만 메인테이너 시각 판정 중 대상 문서 3쪽에서 단 구분선이 본문 하단까지 그려지지 않는 회귀를 확인했습니다. 이 부분은 maintainer-side 보정으로 `zone_layout=None`인 페이지 기본 다단은 body 전체 높이로 그리고, `zone_layout=Some`인 페이지 내부 zone 전환만 콘텐츠 높이를 쓰도록 분기해 수정했습니다.

로컬 검증:

- `cargo fmt --all -- --check`
- `cargo test --test issue_702`
- `cargo test --test issue_874_ktx_toc_page_number_right_align`
- `cargo test --test svg_snapshot`
- `cargo test --lib`
- `cargo clippy --all-targets -- -D warnings`
- 대상 샘플 3쪽/22쪽 SVG export 및 fixed SVG 재검증

GitHub Actions도 Build & Test / Canvas visual diff / CodeQL 모두 success 상태임을 확인했습니다.

메인테이너 SVG/WASM 시각 판정도 통과했습니다. 수용 절차로 진행하겠습니다. 감사합니다.
```
