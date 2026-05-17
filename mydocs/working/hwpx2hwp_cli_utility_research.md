# hwpx2hwp 엣지 케이스 진단 CLI 조사

## 1. 배경

`hwpx -> IR -> hwp save` 작업은 앞으로도 다양한 엣지 케이스가 계속 추가될 가능성이 높다.

최근 작업 흐름에서 반복된 수작업:

- HWPX 원본 `info`
- 한컴 정답 HWP `info`
- rhwp adapter 저장본 생성
- 저장본 재로드 `info`
- `ir-diff --summary`
- 필요 시 `export-svg --debug-overlay`
- 작업지시자 시각 판정용 output 경로 생성
- stage 문서에 결과 복사

이 반복을 줄이는 유틸리티 CLI가 필요하다.

## 2. 현재 CLI 자산

현재 `rhwp` CLI에 존재하는 유용한 단품 도구:

```text
info
dump
dump-pages
export-svg
export-png
export-pdf
ir-diff
diag
convert
```

### info

장점:

- 파일 형식/버전/페이지/구역/문단 수 확인 가능
- `BinData` type, storage id, extension, loaded bytes 출력
- 표/그림 요약 출력

한계:

- 사람 읽기용 텍스트 출력
- JSON 출력 없음
- 두 문서 비교 기능 없음

### ir-diff

장점:

- HWPX/HWP 파싱 결과 IR 차이를 자동 비교
- `--summary`, `--max-lines` 지원
- 문단/ParaShape/TabDef/LINE_SEG/표/그림 공통 속성 비교 가능

한계:

- `BinData` record attr/type/status/content 생존 여부를 직접 비교하지 않음
- `BorderFill`, 쪽 배경, master page, page definition 같은 DocInfo/section 리소스 비교가 제한적
- 출력은 텍스트 중심

### export-svg / dump-pages / dump

장점:

- 시각 판정 전 좌표/문단/표 식별에 매우 유용
- `--debug-overlay`로 문단/표 라벨 확인 가능

한계:

- 개별 명령을 사람이 직접 조합해야 함
- output 경로 표준화가 없음

## 3. #903에서 확인한 생산성 병목

`samples/hwpx/hwpx-h-01.hwpx` 진단 중 반복된 일:

```text
1. 원본 HWPX info
2. 한컴 정답 HWP info
3. adapter HWP 생성
4. native HWP 생성
5. adapter 저장본 재로드 info
6. native 저장본 재로드 info
7. 원본 SVG 출력
8. 원본/정답/저장본 ir-diff summary 3종
9. 결과를 stage 문서에 수동 기록
```

이 중 1~8은 CLI가 자동 생성할 수 있다.

## 4. 권장 CLI 1: hwpx2hwp-probe

가장 먼저 만들 가치가 높은 상위 명령.

```text
rhwp hwpx2hwp-probe <sample.hwpx> \
  --ref-hwp <hancom-reference.hwp> \
  --out output/poc/hwpx2hwp/task903/stage2_probe
```

생성 산출물:

```text
<out>/
  input.info.txt
  reference.info.txt
  adapter.hwp
  adapter.info.txt
  native.hwp
  native.info.txt
  source_svg/
  ir_diff_input_vs_ref.summary.txt
  ir_diff_input_vs_adapter.summary.txt
  ir_diff_ref_vs_adapter.summary.txt
  checks.json
  report.md
```

기본 체크:

```text
input parse ok
reference parse ok
adapter export ok
adapter reload ok
native export ok
native reload ok
page_count input/reference/adapter/native
section_count
paragraph_count
table_count
picture_count
border_fill_count
bin_data_record_count
bin_data_content_count
embedded BinData loaded bytes
page_def width/height/margins
```

장점:

- #903에서 만든 임시 example export 코드를 반복하지 않아도 됨
- 작업지시자 판정 파일이 항상 `output/` 아래 표준 구조로 생성됨
- stage 문서에 붙일 Markdown report를 자동 생성 가능

## 5. 권장 CLI 2: hwpx2hwp-check

CI/테스트 보조용. 파일을 만들기보다 invariant를 검사한다.

```text
rhwp hwpx2hwp-check <sample.hwpx> --ref-hwp <reference.hwp> --json
```

출력 예:

```json
{
  "input": "samples/hwpx/hwpx-h-01.hwpx",
  "reference": "samples/hwpx/hancom-hwp/hwpx-h-01.hwp",
  "adapter_reload": {
    "parse_ok": true,
    "page_count": { "expected": 9, "actual": 9, "ok": true },
    "bin_data_content_count": { "expected": 5, "actual": 0, "ok": false }
  }
}
```

용도:

- RED/GREEN 테스트 전후 빠른 확인
- `cargo test`보다 사람이 보기 쉬운 실패 지점 제공
- batch 실행의 원자 체크 단위

## 6. 권장 CLI 3: ir-diff 보강

`ir-diff`는 계속 핵심 도구로 두되 비교 범위를 확장한다.

추가 후보:

```text
--focus bindata
--focus docinfo
--focus table
--focus page
--json
```

특히 `bindata` 비교 항목:

```text
bin_data_list.len
bin_data_content.len
record[i].attr
record[i].data_type
record[i].status
record[i].compression
record[i].storage_id
record[i].extension
content[id].loaded_bytes
content[id].sha256
```

이 보강이 있으면 #903의 핵심 문제가 `ir-diff --focus bindata`에서 바로 드러난다.

## 7. 권장 CLI 4: hwpx2hwp-batch

샘플이 많아질 때 필요한 batch runner.

```text
rhwp hwpx2hwp-batch --manifest mydocs/plans/hwpx2hwp_cases.toml --out output/poc/hwpx2hwp/batch/20260514
```

manifest 예:

```toml
[[case]]
id = "hwpx-h-01"
input = "samples/hwpx/hwpx-h-01.hwpx"
reference = "samples/hwpx/hancom-hwp/hwpx-h-01.hwp"
checks = ["page_count", "bindata", "table_count"]

[[case]]
id = "business_overview"
input = "samples/hwpx/business_overview.hwpx"
reference = "samples/hwpx/hancom-hwp/business_overview.hwp"
checks = ["cell_fill_pattern", "page_count"]
```

출력:

```text
summary.md
summary.json
cases/<id>/...
```

## 8. 구현 구조 제안

`src/main.rs`가 이미 커지고 있으므로 새 기능의 본체는 library 쪽에 두는 편이 좋다.

후보 모듈:

```text
src/diagnostics/hwpx2hwp_probe.rs
src/diagnostics/doc_summary.rs
src/diagnostics/ir_compare.rs
```

CLI는 얇게 유지:

```text
src/main.rs
  Some("hwpx2hwp-probe") => hwpx2hwp_probe(&args[2..])
  Some("hwpx2hwp-check") => hwpx2hwp_check(&args[2..])
```

핵심 타입:

```text
DocSummary
BinDataSummary
ExportProbeReport
ProbeCheck
ProbeOutcome
```

## 9. 우선순위

### 1순위: hwpx2hwp-probe

가장 즉시 효과가 크다.

완료 기준:

- 입력 HWPX + 정답 HWP + output 경로만 주면 표준 산출물 생성
- adapter/native HWP 생성
- info/ir-diff summary/report.md 자동 생성
- `output/` 규칙 준수

### 2순위: BinData-aware diff/check

#903 같은 문제를 자동으로 잡는다.

완료 기준:

- `bin_data_content_count` 불일치 감지
- `attr` low nibble 불일치 감지
- loaded bytes/sha256 비교

### 3순위: batch manifest

샘플 수가 충분히 늘어난 뒤 도입한다.

완료 기준:

- manifest 기반 N개 샘플 일괄 probe
- Markdown matrix 생성

## 10. #903에 대한 적용 제안

현재 #903 Stage 2 이전 기준으로는 다음 순서가 합리적이다.

```text
1. 이번 Stage 2는 RED 테스트를 GREEN으로 만드는 최소 수정 진행
2. 그 다음 별도 이슈로 hwpx2hwp-probe CLI 등록
3. hwpx2hwp-probe의 첫 기준 샘플로 hwpx-h-01 사용
4. 이후 business_overview, expense_report, mel-001 등 누적
```

즉 #903 본질 수정과 CLI 개발을 한 커밋에 섞지 않는 편이 안전하다.

다만 `BinData` 비교 보강은 #903의 직접적인 관찰 지점이므로,
Stage 2 이후 후속 이슈로 바로 등록할 가치가 높다.
