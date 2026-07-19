# Task #663/#666 Report — 시험지 ingest roundtrip 대표 검증

## 요약

#666 은 기존 visual sweep 도구의 중복 구현 이슈가 아니라, 시험지 분석 결과인
`ingest.json` 을 `rhwp build-from-ingest` 로 HWPX 화하고 다시 `export-text`/`dump`/
`export-svg` 로 산출 가능 여부를 확인하는 e2e 성격의 후속 작업이다.

Stage 1 에서 `build-from-ingest` 의 HWPX 직렬화 실패를 수정했다. Stage 2 에서는 4종
시험지 PDF 전체 텍스트 레이어를 ingest 로 구성해 CLI 왕복 검증을 수행했다.

## 대표 검증 결과

| 구분 | 파일 | ingest 라인 | CLI 생성/검증 | 텍스트 보존 | 산출 text/svg 쪽수 |
|---|---|---:|---|---|
| 국어 | `samples/2010-exam_kor.pdf` | 1993 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 1993/1993 | 49 |
| 영어 | `samples/exam_eng-2010.pdf` | 680 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 680/680 | 11 |
| 수학 | `samples/exam_math.pdf` | 408 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 408/408 | 8 |
| 교육 통합형 | `pdf/3-09월_교육_통합_2022.pdf` | 1995 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 1995/1995 | 49 |

## 검증 명령

대표 ingest JSON 생성 후 각 샘플에 아래 흐름을 적용했다.

```bash
target/debug/rhwp build-from-ingest <ingest.json> -o <sample>.hwpx
target/debug/rhwp export-text <sample>.hwpx -o <text-dir>
target/debug/rhwp dump <sample>.hwpx > <sample>.dump.txt
target/debug/rhwp export-svg <sample>.hwpx -o <svg-dir>
```

추가로 다음 검증을 수행했다.

```bash
cargo test document_core::builders::exam_paper --quiet
cargo build --bin rhwp --quiet
cargo fmt --check
git diff --check
```

## 판단

- 4종 PDF 전체 텍스트 레이어 기준으로, `ingest.json -> HWPX -> export-text` 경로의 텍스트 보존은
  통과했다.
- `dump` 와 `export-svg` 도 모두 성공해 기본 구조 확인과 SVG 산출 경로가 막히지 않음을
  확인했다.
- 이번 검증은 원본 PDF 와 산출 HWPX 의 시각 정합 검증이 아니다. ingest 입력은 PDF 전체
  텍스트 레이어를 줄 단위 `stem_blocks` 로 구성했으므로, 원본 시험지의 다단 배치, 문항
  위치, 이미지, 수식 조판, 글꼴, 지면 밀도는 보존 대상이 아니다.
- 수학/교육 통합형은 원본 PDF 텍스트 레이어부터 수식 의미가 깨지거나 PUA 문자로
  추출된다. 따라서 수식 의미 보존 문제는 `build-from-ingest` 생성기 결함이 아니라
  ingest 생성 단계의 구조화 한계로 분류한다.

## 후속

- Vision 기반 ingest 산출물까지 확보되면 동일 CLI 흐름으로 전수 비교한다.
- 수식/도형/이미지 문제는 텍스트 라인 보존과 별도로 `media` 또는 전용 구조를 ingest
  스키마에 반영하는 후속 이슈로 다룬다.
- 원본 PDF 와의 시각 정합이 목표라면 `mydocs/manual/verification/visual_sweep_guide.md` 기준으로
  별도 visual sweep 을 수행하고, 그 결과는 텍스트 보존 검증과 분리해 판정한다.
