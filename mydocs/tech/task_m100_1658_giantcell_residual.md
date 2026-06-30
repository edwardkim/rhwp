> **현행화 안내(2026-07-01)**: 본 문서의 byeolpyo4 절대 수치(27쪽 / CLIP 1/27p 23.5px)는 라운드2 당시 devel 기준이다. 현재 PR head(devel 병합 후)에서는 **byeolpyo4 = 26쪽 / CLIP 6/26p max 90.7px** 로, devel 진화(다른 PR 병합)에 따라 변동했다. 본 PR의 수정(continuation ≤3 cut)은 byeolpyo4 절대값이 아니라 **controlset(75/92)·대형 오라클(442)·클리핑 게이트(controlset 92 회귀 0)** 로 검증된다. byeolpyo4 클리핑 자체는 deferral 비viable로 ceiling(별도 트랙) 유지.

# 분석 — #1658 별표4 잔여 Δ+2 (거대 셀 4줄 orphan, empty_spacer reset)

- 일자: 2026-06-30 / 대상: 산업통상부 별표4(rhwp 27 / 한글 25, Δ+2 잔여).
- 도구: dump-pages cut 시퀀스 + 한글 COM 행위치(hangul_row_heights/cell break 매핑).

## 1. 정밀 정렬 (rhwp cut vs 한글 COM break)
거대 셀(rhwp rows 31..32 = 한글 A32/row30, pages 7~16):

| 한글 break @line (COM) | rhwp cut (end_cut) | 판정 |
|----------------------|-------------------|------|
| 5 | P7=3 | 2줄 early |
| **40** | P8=37, **P9=41** | **이중 break → 37~41 (4줄 orphan)** |
| 75 | P10=75 | 일치 |
| 110 | P11=110 | 일치 |
| 145 | P12=144 | ±1 |
| 180 | P13=178 | ±2 |
| **215** | P14=212, **P15=216** | **이중 break → 212~216 (4줄 orphan)** |
| 250 | P16=250 | 일치 |

→ **대부분 정렬(일치/±1~2)**. 잔여 Δ+2 = **한글 break(40, 215) 지점의 4줄 orphan 2개**.

## 2. 근인 (디버그로 검증 — empty_spacer 가설 기각)
RHWP_CUT_DBG 로 P9(start=37) 유닛 확인:
```
u37 h=25.6 hard_break=false   u38 h=25.6   u39 h=25.6   u40 h=36.8(타 유닛보다 큼)
u41 h=28.8 hard_break=TRUE empty_spacer=FALSE   ← reset 은 정상 콘텐츠(spacer 아님)
avail=1005
```
- **empty_spacer 가설 기각**: u41 reset 은 empty_spacer=false 정상 콘텐츠.
- `≤4` 로 올리면 u41 break 흡수되어 P9 가 37→71(34유닛 full)로 합쳐지나 **별표4 27 불변**
  (= 페이지수 안 줄고 **재정렬만**). 한글 break @40 까지 건너뛰어 downstream 에 새 경계 생김.
- **진짜 원인**: P8 이 unit 40(36.8px, 타 유닛 25.6 보다 큼) 때문에 **37 에서 capacity-break**
  (한글은 40 에서). 즉 rhwp 페이지 경계가 한글과 **1~3유닛 어긋나는 capacity/정렬 결손**.
  행높이는 COM 상 일치하나, **셀 내부 어디서 끊는지(page-fitting)** 가 비균일 유닛(36.8px 등)에서
  1~3유닛 빗나감 → 누적되어 orphan/추가 페이지.

## 3. 수정 방향·리스크
- orphan 흡수(≤4)는 **shift 만 하고 페이지수 미감소**(한글 break 도 같이 건너뜀) → 부적합.
- 근본은 **page-fitting 을 한글 break 와 정렬**(비균일 유닛 경계에서 한글과 같은 지점에 끊기)인데,
  이는 cut 가드가 아니라 한글 break 지점(COM 으로 확보)을 **권위로 삼아 cut 을 스냅**하는 모델 필요.
  (앞서 reset-snap 32px 는 게이트 회귀 — 한글 break 권위 기반의 정밀 스냅이라야 안전.)
- 안전 ceiling: 별표4 Δ+2(33→27, 무회귀) 유지.

## 4. 한글-break 권위 cut 스냅 시도 — 페이지수 해결, 클리핑 유발(폐기)
continuation 에서 capacity-break 직전 다음 reset(=한글 break)이 ≤4 유닛 내면 거기까지 확장
(overflow ≤ VISIBLE_TAIL_TOL 120px) → **별표4 27→25(한글 완전 일치)**, 대형 오라클 442→**443**,
소형 75 / lib 2006 / roundtrip 4 / #1488 무회귀.

그러나 **detect_table_clipping 이 클리핑 regression 포착**:
- `법무부 별표4(18190781)`: pre-snap 클리핑 0 → 스냅 후 **33px 클리핑(시각 손실)**.
- 페이지수 게이트는 통과(443)하나, 120px overflow 가 일부 문서에서 본문 밖 렌더 → 클리핑.
→ **page-count ↔ clipping 트레이드오프 확정**(통합진단과 정합). 스냅 **폐기**(클리핑 무회귀 우선).

## 5b. render-fidelity 클리핑 수정 착수 — 깊은 reconciliation 확인
- **클리핑 위치 코드**: `layout_partial_table`(table_partial.rs)의 **Task #993 행높이 오버라이드**
  (render==pagination 의도, cut 측정 `row_cut_content_height` 적용). 그러나 **순수 rowspan 연속
  행은 의도적으로 제외**(line 199, 259-261 `continue`) — rowspan 셀 높이는 `resolve_row_heights`
  가 spanned 행에 분배하므로 per-row cut 강제 시 분배가 깨짐. 즉 클리핑은 **pagination(cut 모델)
  ↔ render(rowspan 높이 분배) 정합** 문제 = 단순 오버라이드 확장 불가, 깊은 reconciliation 필요.
- **측정 재현성 블로커**: `detect_table_clipping` CLI 는 byeolpyo4 23.5px 를 **일관 보고(3/3)** 하나,
  동일 바이너리의 직접 `export-svg` 산출 SVG 를 raw/detect_in_svg 로 검사하면 **body 초과 text 없음**.
  같은 export 명령이 출력경로/환경에 따라 다른 결과 → **신뢰 가능한 row pinpoint 가 선결**.
  (detector tempdir export 와 수동 export 의 SVG 불일치 원인 규명 필요.)
- **리스크**: render 좌표/높이 변경은 **시각 회귀 게이트 부재**(render_page_gate 는 페이지수만 검사)로
  광범위 시각 회귀를 못 잡음 → blind 변경 부적절.
- **결론**: render-fidelity 클리핑은 **별도 전용 라운드** — (1) detector 재현성 규명 + row pinpoint,
  (2) 시각 diff 게이트 구축, (3) rowspan cut↔render 높이 분배 정합. 본 라운드는 **무회귀 안전
  ceiling(별표4 Δ+2) 유지**, blind render 변경 미실시.

## 5c. 클리핑 근본 원인 — 코드 레벨 확정 (advance_row_block_cut 누적 스택 결손)
선결 작업(재현성 규명+시각 게이트) 완료 후 정밀 진단:
- **재현성**: detector "flaky" 는 도구가 아니라 호출부 **MSYS `/c/` 경로 mangling**(python glob 무음
  실패) + 한글 NFC/NFD 였음. `norm_path` 강건화로 해소. byeolpyo4 23.5px 클리핑 **실재 확정**
  (PowerShell 네이티브 경로 재현, dA/dB MD5 동일).
- **pinpoint(PARTIAL_DBG)**: byeolpyo4 page2(블록 컷, 연속분) = **선행 행 10–13(249.6px) +
  split row 14(810.2px) = 1059.8 > col 1009 → overflow 50.8px → 클리핑 23.5px**.
- **근본 원인**: `advance_row_block_cut`(table_layout.rs:5248)이 블록의 **각 셀을 avail 에 독립
  검사**(행 stack 누적 미반영) → 블록 **Σ-per-row 가 avail 초과해도 각 행은 통과**. budget 은
  선행 행 reserve(`avail-consumed`)하나, 블록 **내부** 선행 행(10–13)은 row 14 셀 컷에 미반영.
- **이미 존재하는 해결 함수**: `advance_row_block_cut_with_row_offsets`(행별 top offset 차감=누적
  반영). 단 게이팅 `rowbreak_hard_break_row == Some(b_start)` 로 **hard-break 가 블록 첫 행일 때만**
  적용(#1486). byeolpyo4 는 hard-break 가 **나중 행(14)**(#1105류)이라 plain 버전 사용 → 버그.
- **시도·결과**: 게이팅을 나중 행까지 완화 → 클리핑 **악화(23.5→176.4px)**. row-offset 함수의
  offset 값(`cut_row_h` 전체)이 **연속분(continuation)에 안 맞음**. → 되돌림(무회귀 복원).
- **남은 정공법**: 나중-행 hard-break + 선행 행 케이스용 **continuation-aware 누적 스택 컷**
  (offset 값을 연속분 행높이로 산정). 코어 변경, 별도 집중 작업. 양 게이트(페이지+클리핑) 준비됨.

## 5d. continuation-aware 누적 스택 컷 구현 시도 — 계약 충돌 + 타깃 무개선(되돌림)
`advance_row_block_cut` 에 누적 행-offset 추가(row_span==1 행 높이를 빼서 각 행 셀의 eff_avail 축소,
rowspan 셀은 full). 결과:
- **lib 1건 실패**: `test_block_cut_rowspan_giant_split` — 이 테스트는 **버그 자체를 assert**한다.
  (rowspan 라벨 + row0 셀(2줄=32px) 위에 row1 거대 셀, avail=80px → 기대 `[2,2,5]`=거대 5줄(80px)
  full. 그러면 열1 = 32+80 = 112px > 80 **오버플로우**.) 내 수정 `[2,2,3]`(거대 3줄=48px, 32+48=80)
  은 **기하학적으로 옳다**. → 테스트가 현행(버그) 동작을 고정.
- **그러나 byeolpyo4 클리핑 무변(여전히 23.5px)**: 즉 byeolpyo4 page2 의 거대 셀 컷은
  `advance_row_block_cut` 누적-스택 경로로 **재현 안 됨** → **다른 컷 경로**(per-row advance_row_cut
  또는 reset 강제 컷)에서 발생.
- **되돌림**(무회귀).

### 실제 컷 경로 식별 (정정)
컷 시퀀스(`P2: rows=10..15 start_cut=[1,1,1,2,1] end_cut=[28]`)를 보면 byeolpyo4 page2 거대 셀(row14)
은 **`end_cut=[28]` 단일값 = `advance_row_cut`(per-row)** 로 컷된다(5값 start_cut 은 블록 연속분).
즉 page2 = 선행 블록 행(10–13) 배치 후, **거대 셀 row14 를 per-row `advance_row_cut` 로 컷**.
- **버그 위치 추정**: row14 per-row 컷의 **budget 이 선행 블록 행(10–13, 249.6px)을 안 빼서** full
  avail(1009)을 받음 → row14 가 810px(reset unit28 까지)로 컷 → 열 누적 1059.8 > 1009 = overflow.
  (budget = avail - consumed 이 블록 행 높이를 consumed 에 미반영하는 지점.)
- **남은 정공법(정밀)**: typeset 의 row 배치 루프에서 **블록 행 배치 후 per-row 거대 셀 컷에 넘기는
  budget 이 선행 블록 행 높이를 차감**하도록 정정. (advance_row_block_cut 누적-스택 수정은 별건이며
  `test_block_cut_rowspan_giant_split` 계약 한글 검증 후 별도.)

## 5e. per-row 컷 budget 추적 — 최종 근본 원인(block-continuation 측정 불일치)
BUDGET_DBG/ROWTOTAL_DBG 로 byeolpyo4 page2 배치 결정을 추적:
```
r=14 cursor_row=10 consumed=178.7 budget=826.7 avail=1009.1 end_cut=[28] split_total=810.2 guard_exceed=false
```
- 가드(11294)는 `consumed(178.7) + split_total(810.2) = 988.9 ≤ avail 1009` → **통과**(row14 배치).
- **그러나 render 의 선행 행 10–13 = 249.6px**(PARTIAL_DBG: 8.4+184.7+36.7+19.8). 즉 pagination
  의 **`consumed`(178.7)가 render(249.6)를 70.9px 과소측정** → 가드 오판 → row14 배치 → overflow.
- 추적 결과: r=13 만 per-row 경로(19.8). **rows 10–12 는 또 다른(블록) 경로**로 158.9px 기여하나
  render 는 229.8px 로 그림. **render 는 선행 행을 full 로 그리고, pagination 은 cut remainder 로
  측정**하는 **block-continuation 컷 적용 불일치**가 핵심.

### 시도 — `row_block_content_height` max→max(max, Σ-stack) (되돌림)
이 함수가 단일 셀 `max_h` 만 반환(다중 행 블록 stacking 미반영) 발견 → `max(max_h, Σ per-row)` 로
정정. **모든 게이트 무회귀**(lib 2006/소형 75/대형 442/클리핑 baseline 0/roundtrip 4)이나 **byeolpyo4
무개선**(이 함수가 byeolpyo4 선행 행 경로 아님; stacked 도 cut remainder 라 158.9 동일) → 되돌림.

### 종합 — 클리핑은 5층 tangled, block-continuation 측정 불일치가 코어
줄높이→empty_spacer→advance_row_block_cut 누적스택→per-row budget→**block-continuation 측정 불일치**
순으로 5층을 벗겼다. 코어는 **연속분(continuation) 블록에서 render 와 pagination 이 선행 행 컷을
다르게 적용**(render full vs pagination remainder)하는 것. 단일 함수 수정으로 안 되고, 블록 연속분
컷 적용을 render·pagination 양쪽에서 단일화하는 **전용 아키텍처 작업**이 필요. 안전 ceiling 유지.

## 5. 의의·결론
- COM 매핑으로 잔여 원인을 **capacity/정렬 결손**으로 확정(empty_spacer·줄높이 가설 기각).
- 한글-break 스냅은 **페이지수는 정합하나 클리핑 유발** — **detect_table_clipping 인프라가
  페이지수 게이트가 놓친 시각 regression 을 포착**(인프라 가치 입증, 잘못된 ship 차단).
- 별표4 안전 ceiling = **Δ+2(27, 클리핑 무회귀)**. Δ0 은 **클리핑(cut↔render)을 먼저 해결**해야
  스냅이 안전 — render-fidelity 선결.
- 후속 정공법: (1) cut↔render 정합으로 클리핑 제거 → (2) 그 위에서 한글-break 스냅 = 클리핑-safe 정합.
