# 최종 보고서 — #1658 페이지네이션 엔진 개선 (라운드 2)

- 마일스톤: M100 / 브랜치: local/task1658-r2 (upstream/devel) / PR: planet6897 → edwardkim/rhwp (round 2, 신규)
- 선행: 라운드 1 = PR #1670(거대 셀 cut), PDF baseline 인프라 = PR #1684(머지됨). 본 라운드는 그 후속.
- 일자: 2026-06-30 / 성격: 다세션 연구·개선 프로젝트 **라운드 2 완결**
- 오라클: OLE 한글(Windows+한컴) — 페이지수·줄 baseline·클리핑의 권위 기준.
- 검증 원칙: **모든 변경을 페이지수 게이트 + 클리핑 게이트 양쪽으로 검증, 회귀 0**.

## 1. LANDED 수정 (소스: table_layout.rs +11줄)
`advance_row_cut` `tiny_fragment_waste` 흡수 임계를 **continuation(start>0)에서 ≤3, fresh(start==0)
는 ≤2** 로 분기. 거대 셀이 페이지를 가로질러 분할될 때 rhwp 의 capacity-break 가 한글 break 보다
1~3줄 일찍 끊겨 reset 직전 1~3줄 orphan(낭비 페이지)을 만드는 것을 흡수. fresh 의 ≤2 유지로
#1488(가시 문단 사이 reset 3유닛 후 보존) 불변.

### 검증
| 항목 | before | after |
|------|--------|-------|
| 별표4(산업통상부 LPG) | 28쪽(Δ+3) | **27쪽(Δ+2)** |
| 별표1(국토부 주차장법) | 4쪽 일치 | 4쪽 일치 |
| 대형 오라클(랜덤 452) | 441 | **442**(over +1: 2→1) |
| 소형 controlset(92) | 75 | **75 무회귀** |
| lib / hwpx_roundtrip | — | **2006 / 4 passed**(#1488 보존) |

**누적(라운드1+2): 별표4 33→27 (Δ+8→Δ+2), 별표1 5→4 일치.**

## 2. 돌파구 — 한글 행높이 COM 추출 (차단 블로커 해소)
라운드1에서 차단됐던 한글 표 행높이 레퍼런스를 COM 으로 해소:
- 실패 원인: 캐럿이 표 **밖**이라 셀 진입 실패. 해법: `HeadCtrl` 순회로 표(tbl) 찾아
  `GetAnchorPos(0)`→`SetPosBySet`→`FindCtrl`→`ShapeObjTableSelCell`→`get_row_height`/`TableLowerCell`.
- 도구: `tools/hangul_row_heights.py` (한글 행높이 mm→px vs rhwp cut_row_h).
- **확정 발견**: 행높이 fidelity 정상 — byeolpyo1 총높이 diff +6.4px, byeolpyo4 +14.4px(~0.1%).
  → 잔여 over/클리핑은 **줄높이 measurement 가 아니라 cut/배치 로직** (가설 정정·기각).

## 3. 시각 검증 인프라 — 재현성 강건화 + 회귀 게이트
- **detector 재현성 규명**: `detect_table_clipping` 의 "flaky" 는 도구가 아니라 호출부의
  **MSYS `/c/` 경로 mangling**(python glob 무음 실패) + 한글 NFC/NFD 였음. `norm_path()` 강건화로
  해소(이후 경로 형식 무관 신뢰). byeolpyo4 23.5px 클리핑 **실재 확정**(PowerShell 재현, MD5 동일).
- **시각 회귀 게이트**: `tools/clipping_gate.py` + `tests/fixtures/clipping_baseline.tsv`
  (controlset 92문서, 현재 클리핑 0). render 변경 후 **클리핑 증가를 회귀 판정**(페이지수 게이트가
  못 잡는 시각 회귀 차단). 검증: byeolpyo4 23.5px 를 0-baseline 대조 시 회귀 1건 정확 검출.

## 4. 클리핑 근본 원인 — 5층 정밀 진단 (block-continuation 측정 불일치)
가설을 데이터로 한 층씩 벗김(전부 게이트 검증):
1. ~~줄높이 drift~~ (COM 으로 기각) → 2. ~~empty_spacer reset~~ (RHWP_CUT_DBG: reset 은 정상 콘텐츠,
기각) → 3. ~~advance_row_block_cut 누적스택~~ (계약테스트 충돌 + 타깃 무개선) → 4. ~~per-row budget 산식~~
(budget 은 선행 행 이미 차감) → 5. **block-continuation 측정 불일치(코어)**.

**최종 근본 원인**(BUDGET_DBG/PARTIAL_DBG 추적): byeolpyo4 page2 = 선행 행 10–13(render 249.6px) +
split row14(810.2px) = 1059.8 > col 1009 → overflow 50.8px → 클리핑 23.5px. 가드(typeset)는
`consumed(178.7) + split_total ≤ avail` 로 통과시키나, **pagination 의 consumed(178.7)가 render(249.6)를
70.9px 과소측정**. 코어는 **연속분(continuation) 블록에서 render(선행 행 full)와 pagination(cut
remainder)이 컷을 다르게 적용**하는 구조적 불일치 → 단일 함수 수정 불가, 양쪽 컷 적용 단일화하는
**전용 아키텍처 작업** 필요.

## 5. 배제·되돌림 (게이트가 차단한 회귀)
| 시도 | 결과 | 차단 |
|------|------|------|
| 한글-break 권위 cut 스냅 | 별표4 27→25(페이지 일치) | **클리핑 게이트가 18190781 33px 유발 포착** → 폐기 |
| continuation ≤4 + empty_spacer | 무효(reset 강제 컷) | 되돌림 |
| advance_row_block_cut 누적 offset | lib `test_block_cut_rowspan_giant_split` 실패(버그 assert) | 되돌림 |
| row_block_content_height max→Σstack | 전 게이트 무회귀나 byeolpyo4 무개선 | 되돌림(검증효과 0) |
- **클리핑 게이트가 페이지수 게이트가 놓친 시각 회귀(스냅 33px)를 정확히 차단**(인프라 가치 입증).

## 6. PDF 질문 정리 (라운드 보조)
- PDF export = `PrintToPDF`+`HPrint`(`PrintMethod=4`=모아찍기 → 별표4 25→13쪽 압축). 출력경로 미해결.
- `save_as PDF` 는 doc 인쇄설정 honor, 세션 override 무효 → 배율 100% 강제 불가.
- **결론**: PDF 경로는 미해결 + redundant. COM 이 편집기 행높이를 직접 주는 올바른 레퍼런스.

## 7. 잔여 (후속 라운드)
- **별표4 Δ+2(클리핑 23.5px)**: 코어 = block-continuation 컷 적용 render/pagination 단일화(전용 작업).
  양 게이트 준비 완료.
- 안전 ceiling = 별표4 Δ+2, 클리핑 무회귀. blind 코어 변경 미실시.

## 8. 결론
- 법령 표 거대 셀 분할 over 를 **무회귀로 추가 개선**(별표4 Δ+3→Δ+2, 대형 오라클 441→442).
- 차단 블로커(한글 행높이)를 **COM 으로 해소**, 클리핑 측정 **재현성 규명 + 시각 회귀 게이트** 구축.
- 클리핑을 **5층 정밀 진단**해 코어(block-continuation 측정 불일치)까지 규명. **#1658 존속**,
  본 라운드 landed·무회귀.

## 검증 재현 (샘플)
검증용 HWP 를 `samples/` 에 추가 — 리뷰어가 직접 재현 가능:
- `samples/byeolpyo4.hwp` (산업통상부 별표4, 액화석유가스, 원본 law.go.kr 문서ID 17889615): 거대 셀
  분할 Δ+2 / 클리핑 케이스. `rhwp dump-pages samples/byeolpyo4.hwp` → **27쪽**,
  `python tools/detect_table_clipping.py samples/byeolpyo4.hwp --exe <rhwp>` → **CLIP 1/27p 23.5px**.
- `samples/byeolpyo1.hwp` (국토부 별표1, 거대 셀 표): `rhwp info samples/byeolpyo1.hwp` → **4쪽**(일치).

## 9. 산출물
- 소스: `src/renderer/layout/table_layout.rs`(+11, continuation ≤3)
- 검증 샘플: `samples/byeolpyo1.hwp`, `samples/byeolpyo4.hwp`
- 도구: `tools/{hangul_row_heights,clipping_gate,hangul_pdf_baseline}.py`,
  `tools/detect_table_clipping.py`(norm_path 강건화)
- 픽스처: `tests/fixtures/clipping_baseline.tsv`(controlset 92)
- 문서: `mydocs/tech/investigations/issue-1658/task_m100_1658_{com_rowheight,giantcell_residual,capacity,clipping_capacity_unified}.md`,
  `manual/{visual_clipping_detector,hangul_pdf_baseline}.md`, 본 보고서
