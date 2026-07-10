# PR #2160 검토 - per-pi oracle 미주 PageItem 정렬

- PR: https://github.com/edwardkim/rhwp/pull/2160
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/2152
- 작성자: `planet6897`
- base: `devel`
- head: `fix/2152-oracle-endnote-pi`
- PR head: `54c0ccb040ab15dd3eeb6be95460a0cb012a8b7d`
- merge commit: `c03fff7b3c551835d205da0c8e0ca4a95f9cba8c`
- merge 일시: 2026-07-10 16:47 KST
- 처리 방식: 옵션 2. 원 코드 PR merge 후 docs-only review 기록 PR 로 분리

## 결론

PR #2160은 merge 완료했다.

`dump-pages`에서 미주 PageItem의 `[미주]` 라벨을 `FullParagraph` 외 `PartialParagraph`,
`Table`, `PartialTable`, `Shape`까지 확장하고, `tools/verify_pi_page_vs_hangul.py`가
미주 PageItem을 본문 PI count/mapping에서 제외하도록 맞춘 방향은 PR 목적에 부합한다.

관련 이슈 #2152는 PR 본문 기준으로도 잔여 `PARA_COUNT` 축이 남아 있으므로 close 하지 않는다.

## 변경 범위

- `src/document_core/queries/rendering.rs`
  - 미주 PageItem kind 라벨 확장:
    - `PartialParagraph[미주]`
    - `Table[미주]`
    - `PartialTable[미주]`
    - `Shape[미주]`
- `tools/verify_pi_page_vs_hangul.py`
  - `[미주]` 라벨이 붙은 dump line을 rhwp 본문 PI mapping에서 제외
- 테스트 기대 문자열 갱신
  - `tests/issue_1139_inline_picture_duplicate.rs`
  - `tests/issue_1375_endnote_rewind_column_overflow.rs`
- PR 내부 계획/보고서 추가
  - `mydocs/plans/task_m100_2152.md`
  - `mydocs/report/task_m100_2152_report.md`

## 검증

- GitHub Actions 최신 head 기준 통과
  - CI / Build default-feature tests: pass
  - Native Skia tests: pass
  - Render Diff / Canvas visual diff: pass
  - CodeQL: pass
- 로컬 검증
  - `git diff --check upstream/devel...local/pr2160`: pass
  - `python3 -m py_compile tools/verify_pi_page_vs_hangul.py`: pass
  - `cargo fmt --check`: pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate --test issue_1375_endnote_rewind_column_overflow`: pass
- 실제 dump 확인
  - `samples/3-09월_교육_통합_2022.hwp`에서 `PartialParagraph[미주]`, `Table[미주]`,
    `Shape[미주]` 출력 확인
  - `samples/3-11월_실전_통합_2022.hwpx`: rhwp pages 21, body PI count 451
  - `samples/3-11월_실전_통합_2022.hwp`: rhwp pages 21, body PI count 451
  - `samples/endnote-01.hwp`: rhwp pages 5, body PI count 40
  - `samples/SO-SUEOP.hwpx`: rhwp pages 46, body PI count 1037
  - `samples/SO-SUEOP.hwp`: rhwp pages 46, body PI count 1037

## 잔여 참고

`tools/verify_pi_page_vs_hangul.py`의 미주 제외 조건은 현재 `if "[미주]" in ln:`으로
줄 전체를 검사한다. synthetic 입력에서 본문 preview 텍스트에 literal `[미주]`가 포함된
`FullParagraph pi=0` 줄이 rhwp start mapping에서 누락되는 것을 확인했다.

이번 PR의 본문 목적과 검증 샘플에서는 merge blocker로 보지 않았다. 다만 후속 개선 시에는
line 전체가 아니라 kind prefix, 예를 들어 `FullParagraph[미주]`, `PartialParagraph[미주]`,
`Table[미주]`, `PartialTable[미주]`, `Shape[미주]`만 제외하도록 좁히는 편이 더 안전하다.

## 후속

- #2152는 open 유지한다.
- 원 PR에는 merge 완료와 #2152 open 유지 사유를 후속 comment로 남긴다.
