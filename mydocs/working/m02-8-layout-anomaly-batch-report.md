# M02-8 samples/ layout-anomaly 배치 리포트

M08 착수 근거 데이터. 레이아웃 버그를 고치지 않았다. `--batch`(#5371) 없이
devel 단건 `layout-anomaly --json` 을 파일마다 돌렸다.

## 재현

```bash
cargo build --release --bin rhwp
python tools/layout_anomaly_batch_report.py --root samples --top 20 --json-out mydocs/working/m02-8-layout-anomaly-batch-report.json --tsv-out mydocs/working/m02-8-layout-anomaly-batch-report.tsv --md-out mydocs/working/m02-8-layout-anomaly-batch-report.md
```

- 이슈: #5390
- 생성기: `tools/layout_anomaly_batch_report.py`
- CLI 계약: `per-file: rhwp layout-anomaly <file> --json (devel; --batch is #5371)`
- 바이너리: `target/debug/rhwp.exe` (rhwp v0.8.4)
- git: `0bc05ef81107ac61ec38d622f71b44a44d1b4821` (`feat/m02-8-anomaly-report`)
- 입력: `samples`  fileCount=694  limit=None
- timeout=180s  jobs=2
- `--batch` 지원: no
- off-canvas 필드: False
- text-overlap 필드: False
- 시각: 2026-08-18T06:48:41Z → 2026-08-18T06:56:24Z

## 헤드라인 카운트

| 항목 | 값 |
| --- | ---: |
| 스캔 | 694 |
| CLEAN | 425 |
| ANOMALY | 266 |
| ERROR | 3 |
| TIMEOUT | 0 |
| overflow 건수 (파일 수) | 2347 (245) |
| overlap 건수 (파일 수) | 346 (76) |
| empty_page 건수 (파일 수) | 114 (27) |
| off-canvas 건수 (파일 수) | 미지원 (devel / #5389 미병합) |
| text-overlap 건수 (파일 수) | 미지원 (devel / #5379 미병합) |

## Top overflow

| 순위 | 파일 | score | overflow | overlap | empty_page | 상태 |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 1 | `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp` | 271 | 271 | 0 | 1 | ANOMALY |
| 2 | `samples/2025 행정업무운영 편람(최종).hwpx` | 161 | 161 | 4 | 18 | ANOMALY |
| 3 | `samples/2025 행정업무운영 편람(최종).hwp` | 152 | 152 | 3 | 18 | ANOMALY |
| 4 | `samples/hwp3-sample10-hwp5.hwp` | 94 | 94 | 13 | 0 | ANOMALY |
| 5 | `samples/task2287/1342000_edu_curriculum_map.hwp` | 90 | 90 | 0 | 0 | ANOMALY |
| 6 | `samples/80168_regulatory_analysis.hwp` | 48 | 48 | 1 | 0 | ANOMALY |
| 7 | `samples/issue1891/80168_regulatory_analysis.hwpx` | 46 | 46 | 1 | 0 | ANOMALY |
| 8 | `samples/hwp3-sample16.hwp` | 43 | 43 | 5 | 0 | ANOMALY |
| 9 | `samples/hwpctl_API_v2.4.hwp` | 38 | 38 | 12 | 0 | ANOMALY |
| 10 | `samples/3-09월_교육_통합_2022.hwpx` | 37 | 37 | 2 | 0 | ANOMALY |
| 11 | `samples/hwp3-sample16-hwp5.hwpx` | 29 | 29 | 0 | 0 | ANOMALY |
| 12 | `samples/issue1891_external_bindata_link.hwpx` | 28 | 28 | 4 | 0 | ANOMALY |
| 13 | `samples/hwp3-sample5-hwp5-v2018.hwp` | 27 | 27 | 0 | 0 | ANOMALY |
| 14 | `samples/hwp3-sample5-hwp5-v2024.hwp` | 27 | 27 | 0 | 0 | ANOMALY |
| 15 | `samples/hwp3-sample5-hwp5.hwp` | 27 | 27 | 0 | 0 | ANOMALY |
| 16 | `samples/hwp3-sample10-hwpx.hwpx` | 26 | 26 | 13 | 0 | ANOMALY |
| 17 | `samples/issue2559/1341000_research_report_footnotes.hwp` | 25 | 25 | 0 | 0 | ANOMALY |
| 18 | `samples/synam-001.hwp` | 24 | 24 | 0 | 0 | ANOMALY |
| 19 | `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` | 23 | 23 | 0 | 0 | ANOMALY |
| 20 | `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 21 | 21 | 8 | 0 | ANOMALY |


## Top overlap

| 순위 | 파일 | score | overflow | overlap | empty_page | 상태 |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 1 | `samples/issue1858_paper_anchor_float_stack.hwpx` | 14 | 13 | 14 | 0 | ANOMALY |
| 2 | `samples/hwp3-sample10-hwp5.hwp` | 13 | 94 | 13 | 0 | ANOMALY |
| 3 | `samples/hwp3-sample10-hwpx.hwpx` | 13 | 26 | 13 | 0 | ANOMALY |
| 4 | `samples/hwp3-sample10.hwp` | 13 | 0 | 13 | 0 | ANOMALY |
| 5 | `samples/hwpctl_API_v2.4.hwp` | 12 | 38 | 12 | 0 | ANOMALY |
| 6 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx` | 11 | 21 | 11 | 0 | ANOMALY |
| 7 | `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwpx` | 10 | 8 | 10 | 0 | ANOMALY |
| 8 | `samples/issue4090/156492236_규제샌드박스_min.hwpx` | 10 | 19 | 10 | 0 | ANOMALY |
| 9 | `samples/3-09월_교육_통합_2023.hwp` | 9 | 2 | 9 | 0 | ANOMALY |
| 10 | `samples/3-09월_교육_통합_2023.hwpx` | 9 | 8 | 9 | 0 | ANOMALY |
| 11 | `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp` | 9 | 4 | 9 | 0 | ANOMALY |
| 12 | `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwpx` | 9 | 19 | 9 | 0 | ANOMALY |
| 13 | `samples/3-11월_실전_통합_2022.hwp` | 8 | 2 | 8 | 0 | ANOMALY |
| 14 | `samples/3-11월_실전_통합_2022.hwpx` | 8 | 9 | 8 | 0 | ANOMALY |
| 15 | `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp` | 8 | 4 | 8 | 0 | ANOMALY |
| 16 | `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwpx` | 8 | 4 | 8 | 0 | ANOMALY |
| 17 | `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp` | 8 | 9 | 8 | 0 | ANOMALY |
| 18 | `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 8 | 21 | 8 | 0 | ANOMALY |
| 19 | `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp` | 8 | 2 | 8 | 0 | ANOMALY |
| 20 | `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwpx` | 8 | 9 | 8 | 0 | ANOMALY |


## Top empty_page

| 순위 | 파일 | score | overflow | overlap | empty_page | 상태 |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 1 | `samples/2025 행정업무운영 편람(최종).hwp` | 18 | 152 | 3 | 18 | ANOMALY |
| 2 | `samples/2025 행정업무운영 편람(최종).hwpx` | 18 | 161 | 4 | 18 | ANOMALY |
| 3 | `samples/hwpspec.hwp` | 10 | 15 | 1 | 10 | ANOMALY |
| 4 | `samples/issue4514/sample1-repro.hwp` | 10 | 7 | 0 | 10 | ANOMALY |
| 5 | `samples/hwpx/issue2019_floating_form_74312.hwpx` | 9 | 7 | 0 | 9 | ANOMALY |
| 6 | `samples/table-ipc.hwp` | 8 | 0 | 0 | 8 | ANOMALY |
| 7 | `samples/hwp-3.0-HWPML.hwp` | 7 | 4 | 0 | 7 | ANOMALY |
| 8 | `samples/[2027] 온새미로 1 본교재.hwp` | 4 | 0 | 0 | 4 | ANOMALY |
| 9 | `samples/[2027] 온새미로 1 본교재.hwpx` | 4 | 0 | 0 | 4 | ANOMALY |
| 10 | `samples/issue2006/1790387_prep_final_report.hwpx` | 4 | 13 | 0 | 4 | ANOMALY |
| 11 | `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` | 3 | 0 | 0 | 3 | ANOMALY |
| 12 | `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp` | 3 | 0 | 0 | 3 | ANOMALY |
| 13 | `samples/table-complex.hwp` | 2 | 0 | 0 | 2 | ANOMALY |
| 14 | `samples/field-01-memo.hwp` | 1 | 0 | 0 | 1 | ANOMALY |
| 15 | `samples/field-01.hwp` | 1 | 0 | 0 | 1 | ANOMALY |
| 16 | `samples/hwp3-sample11-hwp5.hwp` | 1 | 15 | 0 | 1 | ANOMALY |
| 17 | `samples/hwp3-sample11-hwpx.hwpx` | 1 | 10 | 0 | 1 | ANOMALY |
| 18 | `samples/hwp3-sample11.hwp` | 1 | 20 | 0 | 1 | ANOMALY |
| 19 | `samples/hwpx/hwpx-02.hwpx` | 1 | 0 | 0 | 1 | ANOMALY |
| 20 | `samples/tac-img-02.hwp` | 1 | 19 | 0 | 1 | ANOMALY |


## Top off-canvas

devel 바이너리에 `offCanvasCount` 가 없다. #5389 병합 후 같은 명령으로 다시 돌린다.


## Top text-overlap

devel 바이너리에 `textOverlapCount` 가 없다. #5379 병합 후 같은 명령으로 다시 돌린다.


## Top 총 신호

| 순위 | 파일 | score | overflow | overlap | empty_page | 상태 |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 1 | `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp` | 272 | 271 | 0 | 1 | ANOMALY |
| 2 | `samples/2025 행정업무운영 편람(최종).hwpx` | 183 | 161 | 4 | 18 | ANOMALY |
| 3 | `samples/2025 행정업무운영 편람(최종).hwp` | 173 | 152 | 3 | 18 | ANOMALY |
| 4 | `samples/hwp3-sample10-hwp5.hwp` | 107 | 94 | 13 | 0 | ANOMALY |
| 5 | `samples/task2287/1342000_edu_curriculum_map.hwp` | 90 | 90 | 0 | 0 | ANOMALY |
| 6 | `samples/hwpctl_API_v2.4.hwp` | 50 | 38 | 12 | 0 | ANOMALY |
| 7 | `samples/80168_regulatory_analysis.hwp` | 49 | 48 | 1 | 0 | ANOMALY |
| 8 | `samples/hwp3-sample16.hwp` | 48 | 43 | 5 | 0 | ANOMALY |
| 9 | `samples/issue1891/80168_regulatory_analysis.hwpx` | 47 | 46 | 1 | 0 | ANOMALY |
| 10 | `samples/3-09월_교육_통합_2022.hwpx` | 39 | 37 | 2 | 0 | ANOMALY |
| 11 | `samples/hwp3-sample10-hwpx.hwpx` | 39 | 26 | 13 | 0 | ANOMALY |
| 12 | `samples/issue1891_external_bindata_link.hwpx` | 32 | 28 | 4 | 0 | ANOMALY |
| 13 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx` | 32 | 21 | 11 | 0 | ANOMALY |
| 14 | `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 29 | 21 | 8 | 0 | ANOMALY |
| 15 | `samples/hwp3-sample16-hwp5.hwpx` | 29 | 29 | 0 | 0 | ANOMALY |
| 16 | `samples/issue4090/156492236_규제샌드박스_min.hwpx` | 29 | 19 | 10 | 0 | ANOMALY |
| 17 | `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwpx` | 28 | 19 | 9 | 0 | ANOMALY |
| 18 | `samples/hwp3-sample5-hwp5-v2018.hwp` | 27 | 27 | 0 | 0 | ANOMALY |
| 19 | `samples/hwp3-sample5-hwp5-v2024.hwp` | 27 | 27 | 0 | 0 | ANOMALY |
| 20 | `samples/hwp3-sample5-hwp5.hwp` | 27 | 27 | 0 | 0 | ANOMALY |


## ERROR / TIMEOUT

| 파일 | 상태 | 오류 |
| --- | --- | --- |
| `samples/HWP3-password-123456.hwp` | ERROR | 오류: 문서 로드 실패 samples\HWP3-password-123456.hwp: InvalidFile("비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)") |
| `samples/hwp3-sample16-hwp5-2024-password-123456.hwp` | ERROR | 오류: 문서 로드 실패 samples\hwp3-sample16-hwp5-2024-password-123456.hwp: InvalidFile("비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 ... |
| `samples/HWP5-password-123456.hwpx` | ERROR | 오류: 문서 로드 실패 samples\HWP5-password-123456.hwpx: InvalidFile("비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)") |

## 메모

- resume mydocs/working/m02-8-layout-anomaly-batch-report.json: kept 218, rerun 476/694
- --batch 없음 (#5371). 단건 layout-anomaly --json 루프로 산출
- offCanvasCount 없음 — #5389 미병합. 카운트는 null
- textOverlapCount 없음 — #5379 미병합. 카운트는 null
- ERROR 3건은 암호 문서 로드 실패(비밀번호 필요). 레이아웃 스캔 실패가 아니다
- timeout=180s, TIMEOUT 0. 전수 제한 없이 694파일 완료
