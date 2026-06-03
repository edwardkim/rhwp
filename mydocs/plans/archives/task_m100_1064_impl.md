# Task M100-1064 — el-school-001.hwpx 저장 (구현 계획서)

- 이슈: [#1064](https://github.com/edwardkim/rhwp/issues/1064)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1064`
- 일시: 2026-05-22
- 수행 계획서: [`task_m100_1064.md`](task_m100_1064.md)

## 1. 본질 식별 (사전 조사 결과)

### 1.1 record-level diff

| Tag | 정답지 | 저장본 | 본질 |
|-----|--------|--------|------|
| **87 (HWPTAG_CTRL_DATA)** | **2** | **0** | **누락** |

다른 record 모두 정합 (66/67/68/69/71/72/76/82 등).

### 1.2 누락 CTRL_DATA 의 위치 + 구조

위치: 표 control (`' lbt'` = 'tbl ') 직후 level=2 (CTRL_HEADER 자식 record).

payload (두 표 동일, 104 byte) — ParameterSet (hwplib `ForParameterSet.java` 정합):

```
1B 02         outer_ps.id = 0x021B
01 00         outer_count = 1
00 00         skip 2 byte
42 02         outer_item.id = 0x0242
00 80         outer_item.type = 0x8000 (ParameterSet)
  42 02         inner_ps.id = 0x0242
  0B 00         inner_count = 11
  00 00         skip 2 byte
  ----- 11 items (각 8 byte = 4 byte header + 4 byte value) -----
  00 40 04 00   item[0]: id=0x4000 type=0x0004 (Integer4)
  F2 0E 00 00   value = 0x0EF2 (3826)
  01 40 04 00   item[1]: id=0x4001 type=0x0004
  18 04 00 00   value = 0x0418 (1048)
  ... (id=0x4002~0x400A, 모두 Integer4) ...
```

11 parameters (id 0x4000~0x400A) — 표 의 페이지 위치 / 크기 / 셀 그리드 메타데이터.
**두 표 동일 값** — 같은 페이지 구조의 표가 동일 ParameterSet 가짐.

### 1.3 어댑터 현재 상태

- `src/document_core/converters/hwpx_to_hwp.rs::adapt_paragraph` 의 `Control::Table` arm
  → `adapt_table(table, report)`
- `adapt_table` 은 `table.raw_ctrl_data` 합성 + 셀 처리만 — **CTRL_DATA 자식 record
  (ctrl_data_records 슬롯) 미처리**
- 직렬화기는 `ctrl_data_record` 가 Some 일 때만 CTRL_DATA 출력
  (`src/serializer/control.rs:181-194`)

### 1.4 본 task 의 본질 가설

- **표 컨트롤의 ParameterSet ctrl_data 누락** → 한컴이 표 메타데이터 손실로 로딩 실패
- 같은 페이지 구조의 표끼리 동일 ParameterSet — **셀 안 도형 포함 표** 의 한컴 contract

작업지시자 보고 증상 (2)(3) (rhwp-studio 자체 렌더링 결함) 은 어댑터 영역과 별개 가능
— Stage 1 후 한컴 시각 판정 후 본 task 분리 여부 결정.

## 2. 단계 구성 (4 단계)

### Stage 1 — 진단 도구 + ParameterSet 정밀 분석

**파일**:
- `examples/dump_table_ctrl_data.rs` (신규) — 정답지/저장본 표 CTRL_DATA payload 비교
- `examples/repro_1064_save.rs` (신규) — HWPX → HWP 저장 reproduce

**검증**: 두 표 의 ParameterSet 11 parameters (id 0x4000~0x400A) 값을 정답지에서 추출
→ payload 의 의미 식별 (페이지 위치 / 크기 / 셀 메타 중 어느 것인지).

### Stage 2 — HWPX 어댑터 표 CTRL_DATA 합성

**파일**: `src/document_core/converters/hwpx_to_hwp.rs`

- `AdapterReport` 신규 필드: `table_ctrl_data_synthesized: u32`
- `adapt_paragraph` 의 `Control::Table` arm 에서 `ctrl_data_records[idx]` 슬롯에 합성 payload 작성
- 신규 함수 `adapt_table_ctrl_data(table, ctrl_data_slot, report)`:
  - 표 의 페이지 위치 / 크기 / 셀 메타 → 11 Integer4 parameter 합성
  - 또는 **간단 접근**: 정답지 hardcoded payload 만 적용 (모든 HWPX 출처 표 동일)

가능 후보:
- (a) HWPX `<hp:tbl>` 의 attribute 에서 11 parameter 값 추출
- (b) 표 의 `common.width`/`height`/`vertical_offset` 등에서 계산
- (c) 정답지 hardcoded (다른 fixture sweep 회귀 점검 필수)

Stage 1 분석 결과에 따라 (a)/(b)/(c) 결정.

### Stage 3 — 회귀 가드 + sweep + 작업지시자 시각 판정 게이트

**파일**: `tests/issue_1064_table_ctrl_data.rs` (신규)

- `issue_1064_table_ctrl_data_emitted` — 표 control 직후 CTRL_DATA record 존재
- `issue_1064_table_ctrl_data_count_matches_oracle` — el-school-001.hwpx 변환 시 2개
- `issue_1064_table_ctrl_data_payload_size` — 정답지 정합 (104 byte)
- 기타 검증

**sweep**: 12 fixtures (HWP/HWPX) 회귀 부재 확인 — 본 영역이 다른 표 fixture 의 라운드트립
에 영향 없는지 확인.

**작업지시자 시각 판정**: `output/poc/issue_1064/repro.hwp` 한컴 한글 2020 로딩 정상 + 본문 표시 정상 확인.

### Stage 4 — WASM Docker 빌드 + 최종 보고서 + 트러블슈팅

- WASM Docker 빌드 → pkg → rhwp-studio/public 동기화
- rhwp-studio 동작 판정 (작업지시자)
- 증상 (2)(3) (rhwp-studio 셀 이미지/도형 렌더링) — 작업지시자 보고 후 분리 task 또는 본 task 포함 결정
- 최종 보고서 + 트러블슈팅 + spec errata 갱신 후보 (필요 시)
- commit/merge/push + 이슈 close

## 3. 본 task 의 위험 + 결정 지점

| 위험 | 영향 | 완화 |
|------|------|------|
| ParameterSet payload 의 11 parameter 값이 표마다 다름 | 정답지만 동일 (우연일 수 있음) | Stage 1 의 정밀 분석 + 다른 fixture sweep |
| HWPX → 어댑터 변환 시 11 parameter 의 값 계산 방식 모름 | 한컴 호환 불가 | hwplib `ForCtrlData` 추가 분석 + 정답지 패턴 분석 |
| 모든 표에 동일 ParameterSet 적용 시 셀 안 도형 없는 표 회귀 | 회귀 발생 가능 | 셀 안 도형 있는 표만 적용 (조건 가드) |
| 증상 (2)(3) 가 본 영역과 별개 | 작업지시자 시각 판정 후 분리 | Stage 4 의 분리 결정 |

## 4. 산출물

- 진단 도구: `examples/dump_table_ctrl_data.rs`
- reproduce: `examples/repro_1064_save.rs`
- 어댑터 정정: `src/document_core/converters/hwpx_to_hwp.rs`
- 회귀 가드: `tests/issue_1064_table_ctrl_data.rs`
- 단계별 보고서: `mydocs/working/task_m100_1064_stage{1..4}.md`
- 최종 보고서: `mydocs/report/task_m100_1064_report.md`
- 트러블슈팅: `mydocs/troubleshootings/hwpx_table_ctrl_data_*.md`
- 산출물 hwp: `output/poc/issue_1064/`

## 5. 작업지시자 승인 요청

1. 본 구현 계획 (4 단계) 승인 여부
2. Stage 2 의 정정 후보 (a/b/c) — Stage 1 결과에 따라 결정 권장 수용 여부
3. 증상 (2)(3) (rhwp-studio 자체 렌더링) — Stage 4 시각 판정 후 분리 결정 권장 수용 여부
