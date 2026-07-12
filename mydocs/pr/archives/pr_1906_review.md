# PR #1906 Self Review — 시험지 ingest roundtrip 검증 복구

## 메타

| 항목 | 값 |
|------|----|
| PR | https://github.com/edwardkim/rhwp/pull/1906 |
| 제목 | task 666: 시험지 ingest roundtrip 검증 복구 |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task/m100-666-ingest-roundtrip` |
| 관련 이슈 | #666 |
| PR 생성 직후 head | `7bfd89ed6127661405435ebbd690131e405318e1` |
| 규모 | +355 / -1, 6 files |
| 처리 경로 | `mydocs/manual/pr_review_workflow.md` 옵션 1 |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 변경 범위

- `build-from-ingest` 산출 HWPX 직렬화 실패를 수정했다.
  - 기본 `DocInfo` pool 없이 문단의 `char_shape_id=0`, `para_shape_id=0`, `style_id=0` 을 참조하던 문제를 보정했다.
  - ingest `default_font` 기반 FontFace 7개 그룹, `BorderFill`, `TabDef`, `CharShape`, `ParaShape`, `Style` 기본 항목을 등록한다.
  - `CharShape`/`ParaShape`/쪽 테두리의 `borderFillIDRef` 는 등록된 1-based id `1` 을 참조한다.
  - ingest `page_size` 를 section `PageDef` 에 반영한다.
- `mydocs/manual/cli_commands.md` 에 `build-from-ingest` 사용법과 검증 흐름을 보강했다.
- `mydocs/plans/task_m100_666.md`, `mydocs/working/task_m100_666_stage1.md`,
  `mydocs/working/task_m100_666_stage2.md`, `mydocs/report/task_m100_663_report.md` 로
  #666 검증 범위와 결과를 기록했다.

## 이슈 대응 판단

#666 은 기존 visual sweep 도구의 중복 구현 이슈가 아니라, `Vision/ingest.json -> build-from-ingest ->
dump/export-text/export-svg` 경로에서 시험지 ingest roundtrip 성격을 확인하는 후속 검증 이슈다.

이번 PR 은 `build-from-ingest` 자체가 HWPX 직렬화에 실패하던 blocker 를 제거하고, 4종 시험지 PDF 전체
텍스트 레이어를 ingest 로 구성해 CLI 왕복을 검증했다.

## 렌더 영향 및 visual sweep 판정

visual sweep 대상이 아니다.

- 이번 PR 의 검증 대상은 원본 PDF 와 생성 HWPX 의 시각 정합이 아니라, ingest 텍스트가 HWPX 생성과
  `export-text` 재추출을 거쳐 보존되는지다.
- `export-svg` 는 산출 HWPX 가 rhwp 렌더러에서 SVG 로 변환 가능한지 확인하는 smoke test 로만 사용했다.
- 현재 임시 ingest 는 PDF 텍스트 레이어를 줄 단위 `stem_blocks` 로 구성하므로, 원본 시험지의 다단 배치,
  문항 위치, 이미지, 수식 조판, 글꼴, 지면 밀도는 보존 대상이 아니다.
- 실제 원본 PDF 시각 정합은 Vision 기반 구조화 ingest 산출물과 스키마 확장이 확보된 뒤 별도 visual sweep 으로
  판정해야 한다.

## 로컬 검증

Focused 검증:

- `cargo test document_core::builders::exam_paper --quiet`
- `cargo build --bin rhwp --quiet`
- `cargo fmt --check`
- `git diff --check`

4종 시험지 전체 PDF 텍스트 레이어 기반 CLI 왕복:

| 구분 | 파일 | ingest 라인 | CLI 생성/검증 | 텍스트 보존 |
|---|---|---:|---|---|
| 국어 | `samples/2010-exam_kor.pdf` | 1993 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 1993/1993 |
| 영어 | `samples/exam_eng-2010.pdf` | 680 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 680/680 |
| 수학 | `samples/exam_math.pdf` | 408 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 408/408 |
| 교육 통합형 | `pdf/3-09월_교육_통합_2022.pdf` | 1995 | `build-from-ingest`, `export-text`, `dump`, `export-svg` 통과 | 1995/1995 |

## 리스크

- 현재 ingest schema v1 은 원본 시험지의 좌표 기반 다단 레이아웃, 수식 의미 구조, 이미지 위치를 충분히 표현하지
  않는다. 따라서 산출 HWPX 는 원본 PDF 와 시각적으로 크게 다를 수 있으며, 이것은 이번 PR 의 실패 조건이 아니다.
- 수학/교육 통합형 PDF 는 `pdftotext` 단계에서 수식이 깨지거나 PUA 문자로 추출된다. 이는 `build-from-ingest`
  생성기 결함이 아니라 Vision/ingest 생성 단계의 구조화 한계다.

## 결론

merge 후보로 판단한다. 최종 merge 전에는 PR head 최신 커밋 기준 GitHub Actions 통과 여부를 확인한다.

## 옵션 1 기록

- review 문서: `mydocs/pr/archives/pr_1906_review.md`
- 처리 계획 문서: `mydocs/pr/archives/pr_1906_review_impl.md`
- 오늘할일: `mydocs/orders/20260704.md`
- visual asset: 없음. visual sweep 대상이 아니며, `export-svg` 는 산출 가능 여부 smoke test 로만 사용했다.

