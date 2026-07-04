# PR #1441 리뷰 기록

## PR 정보

- PR: https://github.com/edwardkim/rhwp/pull/1441
- 제목: `task 1440: 온새미로 그림 어울림 및 문단 테두리 정합`
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1440 (`Closes #1440`)
- 작성자: `jangster77`
- base: `edwardkim/rhwp:devel`
- head: `edwardkim/rhwp:task_m100_1440`
- 상태: Open, Draft 아님
- mergeable: `MERGEABLE`
- merge state: `BLOCKED` (GitHub Actions 대기)
- 작성 시점: 2026-06-19 KST
- 변경 규모: 29 files, +9342 / -6

## 변경 범위

- `samples/[2027] 온새미로 1 본교재.hwp` 35쪽에서 그림 주위 본문 감싸기 LineSeg를 반영해 텍스트가 그림 영역을 침범하지 않도록 보정했다.
- 35쪽 보정이 6쪽 문단 테두리 박스와 #547 passage box를 wrap zone으로 오인하지 않도록, anchor 없는 fallback을 문단 내부 혼합 LineSeg 폭 케이스로 제한했다.
- 6쪽 문단 테두리 박스의 비실선 테두리가 실선으로 렌더링되지 않도록 dash 렌더 경로를 보존했다.
- HWP/HWPX 문단 테두리 연결(`ParaShape.attr1 bit 28`)과 문단 여백 무시(`bit 29`)를 Studio 문단 모양 UI, 수정 명령, HWPX 저장 경로에 연결했다.
- 한컴 PDF/HWPX 비교 자료와 시각 검증 PNG/SVG 자료를 PR에 포함했다.

## 로컬 검토 결과

Blocking finding 없음.

검토 포인트:

- 35쪽 대상 문단은 앞 7줄이 `column_start=850`, `segment_width=20999`, 뒤쪽 줄이 `segment_width=36568`이라 실제 precomputed picture-wrap 흐름으로 판단된다.
- 6쪽 문단 테두리 박스와 #547 passage box는 줄 폭이 단일 패턴이라 LineSeg `cs/sw` 단독 판정 fallback에서 제외된다.
- 문단 테두리 연결과 문단 여백 무시 속성은 HWP attr1 bit와 HWPX `<hh:border connect ignoreMargin>` 양쪽에서 round-trip 가능한 경로로 연결됐다.
- 비실선 문단 테두리는 `Rectangle` 최적화를 타지 않고 면별 line 렌더를 유지해 dash 스타일이 보존된다.

## 로컬 검증

통과 확인:

```text
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
npm --prefix rhwp-studio run build
cargo test --test issue_1440_onsamiro_picture_wrap
cargo test --release --lib renderer::layout::integration_tests::tests::test_547_passage_text_inset_match_pdf_p4
```

세부 결과:

- `cargo test --release --lib`: `1842 passed; 0 failed; 6 ignored`
- `cargo test --test issue_1440_onsamiro_picture_wrap`: `4 passed`
- `cargo clippy --all-targets -- -D warnings`: warning 없이 통과

GitHub Actions 확인:

- 문서 커밋을 PR head에 포함해 push한 뒤 required checks 재실행 완료를 확인한다.
- 문서 커밋 이후 CI 통과 여부만 추가하려고 새 문서 커밋을 다시 push하지 않는다.

## 시각 검증 자료

- 35쪽 한컴 PDF: `mydocs/report/assets/task_m100_1440/hancom_pdf/page-35.png`
- 35쪽 rhwp HWP: `mydocs/report/assets/task_m100_1440/rhwp_hwp_p35.png`
- 35쪽 rhwp HWPX: `mydocs/report/assets/task_m100_1440/rhwp_hwpx_p35.png`
- 6쪽 한컴 PDF: `mydocs/report/assets/task_m100_1440/stage2/hancom_pdf/page-06.png`
- 6쪽 rhwp HWP: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwp_p06.png`
- 6쪽 rhwp HWPX: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwpx_p06.png`

## 리뷰 결론

PR #1441은 #1440의 핵심 증상인 그림 어울림 침범을 보정하면서, PR 준비 중 발견된 문단 테두리 박스 회귀까지 함께 차단한다. 로컬 필수 검증과 targeted 회귀 테스트가 통과했으므로, 리뷰 문서/오늘할일 커밋을 PR head에 포함해 GitHub Actions 재확인 후 merge 가능으로 판단한다.
