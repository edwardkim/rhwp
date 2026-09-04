# Task #1692 Stage 5 - SO-SUEOP 미주 시각 정합 보정

## 시작 상태

- 직전 커밋: `1a8a88fe7 task 1692: SO-SUEOP HWP3 line box와 미주 흐름 보정`
- Stage 4에서 HWP/HWPX export PDF 전체 페이지 수는 기준 PDF와 같이 46쪽으로 맞았다.
- Stage 4에서 HWP3 본문 line box, 미주 번호/shape/column definition, "25)해답" 개요 번호는 회귀 테스트로 고정했다.

## 남은 문제

- 기준 PDF `pdf/SO-SUEOP-2024.pdf` 46쪽 시작 미주: 192번
- 현재 후보 HWP/HWPX 46쪽 시작 미주: 188번
- 기준 PDF는 223번 일부만 오른쪽 단 상단에 이어지지만, 현재 후보는 216번부터 오른쪽 단으로 넘어간다.
- 현재 후보는 203~215번 구간에서 글자 겹침이 남아 있다.
- 페이지 5는 같은 본문 구간으로 수렴했지만, 제목/페이지 번호/일부 굵기와 위치 차이가 남아 있다.

## 분석 방향

- 미주 PageItem 렌더 경로에서 pagination용 advance와 실제 render advance를 분리할 수 있는지 확인한다.
- 비가시 구분선/미주 사이 0 조건에서만 적용되는 좁은 보정으로 제한한다.
- HWP3 전용 분기는 parser/hwp3 내부에 두고, renderer에는 HWP3 전용 조건을 넣지 않는다.
- HWP/HWPX를 항상 같이 검증한다.

## 검증 계획

```bash
cargo fmt
env CARGO_INCREMENTAL=0 cargo test --test issue_1692
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
./target/debug/rhwp export-pdf samples/SO-SUEOP.hwp -o tmp/visual-1692-stage5/pdf/SO-SUEOP-hwp-rhwp.pdf
./target/debug/rhwp export-pdf samples/SO-SUEOP.hwpx -o tmp/visual-1692-stage5/pdf/SO-SUEOP-hwpx-rhwp.pdf
pdfinfo tmp/visual-1692-stage5/pdf/SO-SUEOP-hwp-rhwp.pdf
pdfinfo tmp/visual-1692-stage5/pdf/SO-SUEOP-hwpx-rhwp.pdf
pdftoppm -png -f 5 -l 5 -r 120 pdf/SO-SUEOP-2024.pdf tmp/visual-1692-stage5/render/ref_p5
pdftoppm -png -f 43 -l 46 -r 120 pdf/SO-SUEOP-2024.pdf tmp/visual-1692-stage5/render/ref_p43_46
```

## 구현 결과

- HWP3/HWPX note 내부에서 후속 줄 `vertical_pos=0`이 실제 단/쪽 리셋이 아닌 연속줄인 경우, note 문단 내부에서 이전 줄 advance 기준으로 복원했다.
- HWP3 endnote 22, HWPX endnote 161을 회귀 테스트로 고정했다.
- 비가시 구분선 + 기본 미주 간격 프로필에서 pagination advance와 실제 render line advance 차이를 분리했다.
- SO-SUEOP 미주 하단 tail 경계 보정:
  - p43: 25/26은 왼쪽 단, 27부터 오른쪽 단, 58은 첫 줄만 p43에 남도록 보정.
  - p44: 58의 나머지부터 129까지 유지하고, HWPX 130 하단 잘림을 제거.
  - p45: 130부터 191까지 유지하고, HWPX 161/162 겹침을 제거.
  - p46: 192부터 시작하고, 223은 왼쪽 단 첫 줄 + 오른쪽 단 continuation으로 분리.

## 검증 결과

```bash
cargo fmt
env CARGO_INCREMENTAL=0 cargo test --test issue_1692
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
```

- `issue_1692` 테스트 5개 통과.
- `samples/SO-SUEOP.hwp` export PDF: 46쪽.
- `samples/SO-SUEOP.hwpx` export PDF: 46쪽.
- 기준 PDF `pdf/SO-SUEOP-2024.pdf`: 46쪽.
- 시각 검증 산출물: `tmp/visual-1692-stage5-final14/`.
- p43-46 미주 페이지 경계는 HWP/HWPX 모두 기준 PDF의 주요 흐름과 맞음.
- p5는 본문 내용이 같은 페이지에 남지만, 기준 PDF 대비 헤더 우측 텍스트, 푸터 페이지 번호, 일부 상단 위치 차이가 남아 다음 stage에서 별도 분석 대상이다.
- 전체 회귀 `env CARGO_INCREMENTAL=0 cargo test --all-targets`는 `tests/issue_1116.rs`의 기존 기준선 실패 2건에서 중단됨.
  - `sample16_hwp3_page3_heading_positions_follow_hancom_grid`
  - `sample16_hwp5_page3_dump_pages_summary_uses_lineseg_spacing`
  - 별도 임시 worktree에서 직전 커밋 `1a8a88fe7` 기준 동일 2건이 동일하게 실패함을 확인했으므로 이번 변경의 신규 회귀로 보지 않는다.

## 커밋 규칙

- 이 stage에서 새 커밋을 만들 때 포함하는 stage 문서는 이 파일 하나만 둔다.
- 커밋 후 계속 진행하면 다음 stage 문서를 먼저 만든다.
