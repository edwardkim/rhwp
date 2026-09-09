# Task #1692 Stage 4 - SO-SUEOP HWP3 line box/미주 기준 보정

## 배경

- 기준 PDF `pdf/SO-SUEOP-2024.pdf`는 46쪽이다.
- Stage 3 이후 HWP3/HWPX 모두 페이지 수와 일부 시각 분포가 기준과 어긋났다.
- 특히 HWP3는 본문 문단 line box에 문단 여백이 반영되지 않아 5쪽 이후 본문 흐름이 기준 PDF/HWPX와 크게 달랐고, 43~46쪽 미주 영역도 페이지 분포가 맞지 않았다.

## 원인

- HWP3 파서가 HWP3 `LINE_SEG`의 전체 폭 값을 그대로 정규화하면서 `ParaShape`의 좌/우 여백과 내어쓰기 값을 line box에 반영하지 않았다.
- HWP3 미주 기본값이 HWPX와 달라 미주 번호 suffix, 미주 모양, 2단 미주 column 설정, "해답" 개요 번호가 기준 흐름과 달랐다.
- 비가시 미주 구분선인데 `separator_line_width != 0`만으로 구분선 높이를 잡아 pagination 흐름에 불필요한 높이가 섞였다.
- SO-SUEOP의 비가시 구분선/미주 사이 0 문서는 pagination 계산에는 조밀한 미주 흐름이 필요하지만, 같은 값을 렌더 line spacing에 직접 적용하면 203~215번 미주처럼 글자 겹침이 생긴다.

## 수정 내용

- HWP3 `ParaShape` 단위를 IR 문단 단위로 변환하는 helper를 추가하고 HWP3 line box 계산에 반영했다.
  - `column_start = margin_left + min(indent, 0)`
  - `segment_width = column_width - column_start - margin_right`
- HWP3 미주 후처리를 보강했다.
  - 미주 번호를 HWPX처럼 `숫자)` 형태로 고정
  - HWP3 기본 endnote shape/column definition 보정
  - "해답" 문단의 2단 미주 column definition 삽입
- HWP3 `Outline:` field 기반 개요 번호를 numbering style로 복원했다.
  - "25)해답" 번호가 `-해답` 또는 OLE placeholder처럼 남지 않도록 선행 `-`/U+FFFC를 제거
  - 기본 numbering format과 `HeadType::Number`를 복원
- 비가시 미주 구분선 판정을 `line_type != 0 && line_width != 0`로 좁혔다.
- 비가시 구분선/미주 사이 0인 미주는 pagination 계산용 line spacing 압축과 렌더용 line spacing을 분리했다.
  - pagination은 기존 46쪽 수렴을 유지하기 위해 20분의 1 음수 spacing 후보를 사용
  - 렌더에 저장하는 paragraph는 원본 line spacing을 유지

## 검증

```bash
cargo fmt
env CARGO_INCREMENTAL=0 cargo test --test issue_1692
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
./target/debug/rhwp export-pdf samples/SO-SUEOP.hwp -o tmp/visual-1692-stage4-render-original-spacing/pdf/SO-SUEOP-hwp-rhwp.pdf
./target/debug/rhwp export-pdf samples/SO-SUEOP.hwpx -o tmp/visual-1692-stage4-render-original-spacing/pdf/SO-SUEOP-hwpx-rhwp.pdf
pdfinfo tmp/visual-1692-stage4-render-original-spacing/pdf/SO-SUEOP-hwp-rhwp.pdf
pdfinfo tmp/visual-1692-stage4-render-original-spacing/pdf/SO-SUEOP-hwpx-rhwp.pdf
```

- `cargo fmt`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692`: 통과
- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과
- 기준 PDF `pdf/SO-SUEOP-2024.pdf`: 46쪽
- HWP export PDF: 46쪽
- HWPX export PDF: 46쪽
- HWP3 대표 line box 회귀 테스트 통과:
  - paragraph 57
  - paragraph 77
  - paragraph 1000
- HWP3 미주/개요 번호 회귀 테스트 통과:
  - 미주 번호/width/shape/column definition이 HWPX 기준과 일치
  - "25)해답" 번호 복원

## 시각 검증 결과

- 5쪽:
  - HWP/HWPX 모두 기준 PDF와 같은 본문 구간으로 수렴했다.
  - 다만 제목/페이지 번호/일부 굵기와 위치는 아직 기준 PDF와 차이가 있다.
- 43~46쪽:
  - HWP/HWPX 모두 전체 페이지 수는 46쪽으로 맞다.
  - 현재 후보의 46쪽 시작 미주는 HWP/HWPX 모두 188번이다.
  - 기준 PDF의 46쪽 시작 미주는 192번이다.
  - 현재 후보는 216번부터 오른쪽 단으로 넘어가지만, 기준 PDF는 223번 일부만 오른쪽 단 상단에 이어진다.
  - 203~215번 구간은 렌더 상 글자 겹침이 아직 남아 있다.

## Stage 5 과제

- 미주 pagination 높이와 render advance를 더 분리해 203~215번 겹침을 제거한다.
- 46쪽 시작 미주를 기준 PDF의 192번에 맞추고, 오른쪽 단 시작을 223번 tail 수준으로 늦춘다.
- HWP/HWPX 양쪽에서 같은 페이지 단위 시각 검증을 반복한다.

## 커밋 규칙

- 이번 커밋에는 이 stage 문서 하나만 포함한다.
- 이 커밋 이후 추가 분석/수정을 계속하려면 먼저 `mydocs/working/task_m100_1692_stage5.md`를 새로 만든다.
