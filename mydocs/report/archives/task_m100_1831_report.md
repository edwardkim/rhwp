# Task #1831 최종 결과보고서 — 다쪽 표 continuation 페이지 상수 오프셋

- 마일스톤: M100 / 브랜치: `local/task1831`
- 계획서: `mydocs/plans/task_m100_1831.md` (v2 — 원인 재규명 반영)
- 재현물·실험 스크립트: `output/poc/task1831/` (로컬)
- **리뷰어 재현용(저장소 포함)**: 핵심 표본 2448877 = `samples/float-stack-defer.hwp`
  + 한글 2022 정답지 `pdf/float-stack-defer-2022.pdf` (`rhwp dump-pages` 로 p1/p2
  배치 비교), 셀 플래그 패처 `tools/patch_cell_flags.py`. rowbreak-problem-pages
  (18쪽, 한글 일치)는 기존 `samples/`·`pdf/` 자산으로 재현 가능.

## 1. 배경

hwpdocs 5,200 표본 조사(`report/survey_pipage_20260703.md`)에서 페이지·PI 는 MATCH
인데 continuation 페이지 콘텐츠가 통째로 30~54pt 위로 이동하는 위치 결함(지배
패턴)을 확인, 대표 표본 2448877(별표4)·3114781(한방 검토부표)로 본 타스크 개시.

## 2. 원인 재규명 (v1 오진 → v2 확정)

### v1 오진과 폐기

v1(오전)은 "한글이 continuation 상단에 반복 제목행을 렌더하는데 rhwp 가 누락"으로
진단하고 `leading_header_rows()` 에 repeat_header 폴백(제목셀 전무 시 row0 반복)을
구현했다(WIP 1a0f14b3). 그러나 판정 기준으로 쓴 r2.pdf 가 **한글 정답지가 아니라
rhwp 자체 출력**(PDF producer=`rhwp`)이었음을 확인했다. 진짜 한글 2022 정답지
2종(신규 COM 생성본 + survey `h_10.pdf`, 모두 producer=Hancom PDF)은 서로 일치하며
**반복 제목행이 없다**. 또한:

- 18151945 별표7(repeat_header=true, 제목셀 전무, 306행): 한글 PDF 39쪽 전수 스캔
  — continuation 38쪽 전부 반복 없음(행 경계 시작 페이지 포함). → 폴백은 한글과
  모순. 게이트로도 회귀 실증(18151945 39→40쪽, hwpx sample2 테스트 30≠29).
- 제목셀(hdr=true)이 실제로 있는 표(rowbreak 25×7)는 한글이 반복(p3·4·5)
  — 기존 `has_header_cells` 경로가 정합. → **WIP 폴백 폐기**.

### v2 확정 — 같은 문단 float 스택 이월 규칙

한글 2022 의 실제 동작(2448877): 같은 문단에 앵커된 표1 아래로 밀린 표2가 단
잔여(197px)에 통째로 안 들어가자 **첫 조각을 만들지 않고 표 전체를 p2 로** 민다.
rhwp 는 row0 한 줄을 p1 하단에 조각으로 남겨 p2 전체가 −54.3pt 위로 밀렸던 것.

판별 인자를 한글 COM 변형 실험으로 압축(재현물 `output/poc/task1831/`):

| 실험 | 결과 | 기각된 가설 |
|------|------|------------|
| RepeatHeader=0 저장(attr 0x06000002) | 여전히 밀기 | 제목반복 연동 |
| row1 축소 — 2행(35.9mm)이 잔여에 들어감 | 여전히 밀기 | 최소 2행 |
| 행 10개 추가 — 표 > 1쪽 | 밀고 p2 상단에서 분할 | 새 쪽 수용성 |
| 표1 삭제 + 텍스트 필러 스윕(잔여 133~880px) | 전 구간 분할 | 잔여 공간 임계 |
| 36387040 — 다른 문단 표 3개 선행 | 한글이 분할 | 페이지 위 임의 float |

→ **같은 앵커 문단의 float 스택 멤버만** 통째-이월 그룹으로 다뤄진다.

## 3. 구현 (커밋 53d7b3b3)

`src/renderer/typeset.rs` 분할 진입부:

1. **이월 규칙**: 같은 문단에 앵커된 선행 float(Table/PartialTable, 같은
   `para_index`) 이 현재 단에 있고, 다행 자리차지 표가 단 잔여에 통째로 안
   들어가면 `prefill_before_deferred_table` + `advance_column_or_new_page` 로
   표 전체를 이월(첫 조각 미생성).
2. **전체-배치 2px 허용**: 이월 후 단 상단에서 표 전체가 ≤2px 오차로만 넘치면
   통째 배치 — 행높이 측정 드리프트 흡수(2448877 표2 = 가용 941.1px vs 942.9px).
   전체가 들어갈 때만 적용되어 분할 경계 산정에는 무영향.

`src/model/table.rs`: WIP 폴백 제거(원상 복구). dump `hdr=` 진단(main.rs)은 유지.

## 4. 게이트 결과

| 게이트 | 결과 |
|--------|------|
| 2448877 p2 Δbaseline (vs h_10.pdf) | **−54.3pt → −2.27pt** (p1 −1.0pt) |
| oracle 452 (`render_page_oracle_1658.tsv`) | 파일 단위 쪽수 변경 **0건** (442 일치 유지) |
| 통제셋 92 | 75/14/3 분포 동일 |
| 클리핑 게이트 92 | 신규 0 · 회귀 0 |
| rowbreak-problem-pages | **18쪽, 한글 PDF(-2024) 와 일치** (hwp/hwpx 모두, 클리핑 0) |
| 18151945 / byeolpyo4 / byeolpyo1 | 39 / 26 / 4쪽 유지 |
| cargo test (`--no-fail-fast`, 182 스위트) | 통과. 유일 실패 svg_snapshot 7건은 로컬 autocrlf CRLF 노이즈(내용 diff 없음, CI 무관) |

검증 기준 2종 모두 수행:

- **순수 upstream/devel 단독**: 본 변경만 적용한 대조 실험 — 대조군(미적용 devel)
  대비 oracle 452 파일 단위 쪽수 변경 0건(양쪽 442/−1:7/+1:3 동일), 대조군에서
  결함 재현(float-stack-defer p1 하단 row0 조각) 확인, 적용 후 해소. rowbreak 는
  양쪽 모두 18쪽(무영향).
- **제출 열린 PR 선적용 스택(#1823~#1828 등) 기준**: 상동 무회귀. rowbreak 는
  스택 기준 19쪽이 본 변경으로 18쪽(한글 일치) 회복 — 스택 내 상호작용 상쇄로,
  순수 devel 기준 개선 아님에 유의.

## 5. 잔여 항목 조사 (커밋 a53cc1dc)

1. **3114781 재분류**: p2 −33.9pt 는 continuation 오프셋이 아니라 별도 전면 1×1
   서식 표의 **셀 내부 인라인 개체 라인높이 재계산 드리프트**(p[0] '작성요령'
   라인 저장 lh=3401HU 무시 → 갭 51pt→18pt 압축 등). → **이슈 #1842 등록**.
2. **b7 셀 플래그**: 한글 2022 는 셀 LIST_HEADER bytes6-7 상위 바이트를 렌더링·
   COM UI 모두 무시, 재저장 시 원값 보존만(플립 실험). 레이아웃 불활성 결론 —
   `tech/hwp_spec_errata.md` §10 반영, 코드 변경 불요.
3. **조사보고서 정정**: `report/survey_pipage_20260703.md` §4 에 정정 절 추가
   (실원인·재분류·오진 경위·재발 방지 절차).

## 6. 재발 방지

- 시각 판정 전 PDF metadata **producer 확인**(=Hancom PDF 만 정답지) 절차화 —
  오진의 직접 원인이 rhwp 출력물을 정답지로 오인한 것.
- 압축 스트림 패치 실험 시 deflate 뒤 꼬리 바이트 보존(`patch_flags.py`).

## 7. 커밋 이력 (local/task1831)

| 커밋 | 내용 |
|------|------|
| 1a0f14b3 | (WIP) repeat_header 폴백 + dump hdr 진단 — 본 보고서에서 폴백 폐기 |
| 53d7b3b3 | 같은 문단 float 스택 이월 규칙 + 2px 허용 + 폴백 제거 + 계획서 v2 |
| a53cc1dc | 잔여 조사 — 3114781 재분류·b7 불활성 결론·조사보고서 정정 |

## 8. 후속 (별건)

- #1842 셀 내부 인라인 개체 라인높이 드리프트 (3114781 p2, 후속 타스크)
- 행높이 측정 미세 드리프트(2448877 잔여 −2.3pt)는 #1827 계열 폰트 메트릭
  정합 트랙에서 계속.
