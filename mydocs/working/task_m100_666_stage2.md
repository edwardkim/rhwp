# Task #666 Stage 2 — 4종 시험지 CLI 대표 검증

## 목적

#666 의 DoD 는 기존 visual sweep 도구 자체가 아니라, 시험지 분석 결과인
`ingest.json` 이 rhwp CLI 를 통해 HWPX 로 생성되고 다시 텍스트/구조/SVG 산출물로
확인되는지 검증하는 것이다. Stage 1 에서 최소 샘플 직렬화 실패를 복구했으므로,
Stage 2 에서는 4종 시험지 PDF 전체의 텍스트 레이어를 추출해 ingest 입력으로 만든 뒤
CLI 왕복을 확인했다.

## 검증 대상

| 구분 | 기준 파일 | PDF 페이지 수 | 비고 |
|---|---|---:|---|
| 국어 | `samples/2010-exam_kor.pdf` | 31 | Hancom PDF, A4 |
| 영어 | `samples/exam_eng-2010.pdf` | 14 | Hancom PDF, A3 |
| 수학 | `samples/exam_math.pdf` | 20 | 수식/기호가 PDF 텍스트 레이어에서 깨짐 |
| 교육 통합형 | `pdf/3-09월_교육_통합_2022.pdf` | 23 | Hwp 2024 산출 PDF, PUA 수식 기호 포함 |

## 수행 절차

각 PDF 전체를 `pdftotext -layout` 로 추출하고, 빈 줄을 제외한 텍스트 라인을
`stem_blocks` 로 구성해 대표 `ingest.json` 을 만들었다. 이후 다음 흐름을 4종 모두에
실행했다.

```bash
target/debug/rhwp build-from-ingest <ingest.json> -o <sample>.hwpx
target/debug/rhwp export-text <sample>.hwpx -o <text-dir>
target/debug/rhwp dump <sample>.hwpx > <sample>.dump.txt
target/debug/rhwp export-svg <sample>.hwpx -o <svg-dir>
```

## 결과

| 구분 | ingest 라인 | HWPX 생성 | export-text | dump | export-svg | ingest 텍스트 보존 | 산출 text/svg 쪽수 |
|---|---:|---|---|---|---|---|
| 국어 | 1993 | 통과 | 통과 | 통과 | 통과 | 1993/1993 | 49 |
| 영어 | 680 | 통과 | 통과 | 통과 | 통과 | 680/680 | 11 |
| 수학 | 408 | 통과 | 통과 | 통과 | 통과 | 408/408 | 8 |
| 교육 통합형 | 1995 | 통과 | 통과 | 통과 | 통과 | 1995/1995 | 49 |

생성된 HWPX 는 rhwp 기본 시험지 레이아웃으로 다시 페이지네이션되므로 원본 PDF 페이지 수와
산출 text/svg 파일 수는 다를 수 있다. 페이지별 텍스트 파일을 합산하면 ingest 텍스트는
4종 모두 100% 보존된다.

`export-svg` 통과는 생성된 HWPX 가 rhwp 렌더러에서 SVG 로 산출 가능한지를 확인하는
smoke test 이다. 원본 PDF 의 다단 배치, 문항 위치, 이미지, 수식 조판, 글꼴, 지면
밀도까지 맞춘다는 의미의 시각 정합성 통과가 아니다.

## 결함 분류

- hotfix 필요: 없음. Stage 1 의 DocInfo 기본 pool 보정 후 4종 대표 입력은 모두
  HWPX 생성과 재추출이 가능했다.
- follow-up 필요:
  - 수학/교육 통합형 PDF 의 수식은 `pdftotext` 단계에서 이미 의미 정보가 깨지거나
    PUA 문자로 추출된다. 이는 `build-from-ingest` 의 보존 문제가 아니라 ingest 생성
    전단계의 식별/구조화 문제다.
  - 수식/도형/이미지를 의미 단위로 보존하려면 Vision 분석 결과를 텍스트 라인만이
    아니라 `media` 또는 전용 수식/도형 구조로 분류하는 후속 설계가 필요하다.

## 추가 문서화

`build-from-ingest` 는 CLI 명령이므로 `mydocs/manual/cli_commands.md` 에 다음을 보강했다.

- PDF 직접 분석 명령이 아니라 ingest JSON 을 HWPX 로 조립하는 명령임을 명시
- `-o`, `--media-dir`, 최소 입력 필드 설명
- `build-from-ingest -> export-text -> dump -> export-svg` 검증 흐름 예시
- `export-svg` 는 원본 PDF 대조가 아니라 산출 HWPX 렌더 가능 여부 확인임을 명시
- 수식/도형/손글씨는 ingest 전단계에서 별도 구조로 분류해야 한다는 한계
