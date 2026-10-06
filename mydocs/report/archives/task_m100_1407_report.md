# Task M100 #1407 최종 보고서 — newNum 슬롯 위치 변위 + 143E RT 페이지 수 1→2

- 이슈: #1407 "HWPX serializer: newNum 슬롯 위치 변위 + 143E RT 페이지 수 1→2 — 복합 컨트롤 문단 잔존 결함"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1407`
- 작성일: 2026-06-14

## 1. 개요

`143E433F503322BD33.hwpx` roundtrip 의 잔존 결함 2건(#1382 4단계 분리)을 처리했다.
조사 결과 **두 증상은 별개 원인**이었고, 작업지시자 판단으로 #1407 내 통합 정정했다.

| 증상 | 원인 | 해소 |
|------|------|------|
| ① newNum 슬롯 위치 변위 (char_offsets[3] 27→35) | post-char fieldEnd 방출이 expected 미진행 → 텍스트-끝 슬롯이 fieldEnd 갭 가로챔 | 1단계 — expected+=8 |
| ② RT 페이지 수 1→2 | 본문 colPr 이 IR(2단) 아닌 템플릿 colCount=1 방출 → 2단 손실 | 2단계 — colPr IR 치환 |

## 2. 증상 ① — newNum 슬롯 (1단계)

문단 0.14("김영훈 기자…", 머리말+하이퍼링크 필드+newNum 복합)에서 newNum 이 fieldEnd
뒤·텍스트 앞으로 이동 — char_offsets[3] 27→35.

실측 추적(`RHWP_DBG_1407`) 결과 근본 원인은 `render_runs` post-char fieldEnd 방출
(`section.rs:525~`)이 `expected_utf16_pos` 를 +8 진행하지 않은 것. 다음 idx 에서 텍스트-끝
슬롯(newNum)이 그 8유닛 갭을 가로챘다. **fieldEnd 방출 직후 `expected += 8`** 1줄로 정정
→ newNum 은 남은 슬롯 일괄 방출로 텍스트 끝 배치(원본 XML 동형).

(구현계획서는 pre-char 가드를 가정했으나 실측으로 본질이 한 idx 앞임을 확인 — stage1 정정.)

## 3. 증상 ② — 본문 colPr (2단계)

①을 정정해도 페이지 1→2 불변. `dump-pages` 대조로 **2단(컬럼) 손실**이 원인임을 확정:
원본 `colCount="2" sameGap="2268"` vs RT `colCount="1"`. 본문(depth 0) ColumnDef 가
`render_runs` 인라인 슬롯에서 제외되고(#1379), 템플릿 `empty_section0.xml` 의 하드코딩
colPr 만 방출됐다. **#1388(secPr 여백 템플릿) 동형**.

해소: `write_section` 에서 첫 문단 ColumnDef IR 을 템플릿 colPr anchor 에 치환
(`render_col_pr_ctrl` 재사용 — colCount/sameGap/구분선 IR 재현). RT colCount=2 복원,
**페이지 수 1→1**.

## 4. 검증

- 143E ir-diff 전체 **차이 0** (문단 0.14 char_offsets 정상, colPr colCount=2).
- 143E RT 페이지 수 **1=1** (증상 ② 해소).
- 단위 테스트 3건:
  - `task1407_field_end_not_stolen_by_newnum_slot` (①)
  - `task1407_body_col_pr_reflects_ir_column_def` (②)
  - `task1407_single_column_doc_unaffected` (② 회귀 가드)
- `serializer::hwpx::section` 42 passed (기존 #1298/#1378/#1321/#1379 회귀 없음).
- 전수 배치 PASS 53 / IR_DIFF 0 / SERIALIZE_FAIL 0 / ROUND2 0.
- baseline 4 passed — **B=0 유지**.
- CI급: `cargo test --profile release-test --tests` + fmt + clippy (수치 커밋 시점 그린).

## 5. 게이트 사각 (기록)

143E 는 ir-diff·baseline IR diff 가 0인데도 페이지 수가 달랐다 — 본문 colPr **방출 손실**
은 재파싱 시 colCount=1 로 읽혀 양쪽 IR 이 같아 보이는 사각이었다(IR→XML 손실, IR↔IR
비교로는 미검출). 방출 정정으로 표면화는 종결. 본문 템플릿 하드코딩 계열(secPr=#1388·
colPr=#1407)이 동형 패턴이므로, 향후 본문 템플릿 고정값 추가 시 IR 치환 점검 필요.

## 6. 시각 판정

143E 는 PDF 정답지 부재 샘플. 본 정정은 IR 정합 축(슬롯 위치·colPr) + 페이지 수 회복으로,
원본 HWPX 와 동형 방출을 확인했다. 한컴 편집기 판정이 필요하면 작업지시자 환경에서 추가 확인.

## 7. 산출물

- 수행계획서: `mydocs/plans/task_m100_1407.md`
- 구현계획서: `mydocs/plans/task_m100_1407_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1407_stage{1,2}.md`
- 본 최종 보고서
- 트러블슈팅: `mydocs/troubleshootings/hwpx_newnum_slot_after_text.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md`
- 검증 자료: `output/poc/task1407/` (143E rt + batch)
