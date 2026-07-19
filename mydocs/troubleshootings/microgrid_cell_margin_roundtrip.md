---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# Issue #1785 조사 보고서 — cell.apply_inner_margin 파스 불일치의 근본 원인

## 요약

#1772 수정 후 잔여 9.25px 이동 군집의 원인을 확정했다. **HWPX 파서 결함이 아니라,
HWPX→HWP 어댑터의 micro-grid 셀 계약 물질화(`materialize_cell_list_header_contract`)가
LIST_HEADER width_ref bit 0 을 강제 세팅하면서 렌더 의미(apply_inner_margin)까지 바꾸는
것**이 원인이다. 시각 불변식(라운드트립 렌더 보존)을 record 계약 물질화가 깨는 구조.

## 인과 사슬 (재현: 36381023 = samples/task1772/table_outer_margin_common_sync.hwpx)

1. 대상 표: 발신명의 결재란, 9행×**45열** micro-grid, `table.padding=(0,0,0,0)`,
   셀들은 `hasMargin="0"`(aim=false) + 자체 `pad=(140,...)` 보유.
2. **HWPX 파스**: `hasMargin=0` → `apply_inner_margin=false` → 렌더 셀 패딩 =
   `table.padding` = **0** → 표 높이 244.9px (선언 18270HU=243.6px 와 정합).
3. **어댑터**: `table_requires_cell_width_ref_contract` (col_count≥30 micro-grid) 가
   true → `materialize_cell_list_header_contract` 가 모든 셀에
   `list_header_width_ref |= 0x0001` 강제 (한컴 저장본 record 정합용 계약).
4. **HWP5 재파스**: `apply_inner_margin = width_ref bit0` (parser/control.rs:374) →
   **true 로 반전** → 렌더 셀 패딩 = `cell.padding` = **140** → 표 높이 254.2px (+9.3px)
   → 쪽 하단 고정(valign=Bottom)이라 표 상단이 위로 밀리며 본문과의 차이 9.25px 발생.

## 검증 실험 (RHWP_1785_SYNC_PADDING, 원복 완료)

어댑터에서 bit0 을 켤 때 **원래 aim=false 였던 셀의 유효 패딩(table.padding)을
cell.padding 으로 함께 물질화**하면:
- seoul_071 render-diff 변위 9.25px → **3.73px** (흐름 레벨 이동 해소)
- 잔여 3.73px 는 셀 내부 TextLine 배치(Table4/Cell5/TextLine1 등)의 별개 소결함 —
  후속 조사 필요.

## 수정 방향 제안 (승인 요청)

`materialize_cell_list_header_contract` 에서 bit0 을 세팅할 때(use_width_ref 계약 경로),
`cell.apply_inner_margin == false` 인 셀은 유효 패딩을 함께 물질화:
```
cell.padding = table.padding;   // 유효 패딩 보존 (시각 불변)
cell.apply_inner_margin = true; // bit0 과 IR 의미 일치
```
- record 계약(bit0)은 유지하면서 렌더 의미 보존 — 시각 불변식 회복.
- 직렬화 시 padding 필드도 함께 기록되므로 재파스 렌더가 원본과 일치.
- 주의: 한컴 저장본과의 hwp5-inventory-diff 계약(셀 패딩 바이트 값)이 변할 수 있음 —
  oracle 대조(hwp5-inventory-diff) 재확인 필요.

## 잔여/후속

1. 셀 내부 TextLine 3.73px — 셀 패딩 변화와 무관한 별개 소결함, 본 수정 후 재조사.
2. 코퍼스 정량화: 수정 적용 시 OVER 55건 중 몇 건이 해소되는지 배치 재실행.

## 재현 명령

```
rhwp render-diff samples/task1772/table_outer_margin_common_sync.hwpx --via hwp  # 9.25px OVER
rhwp dump <파일> -s 0 -p 5   # 표 padding=(0,..) vs 셀 pad=(140,..), aim=false
rhwp convert <파일> out.hwp && rhwp dump out.hwp -s 0 -p 5   # aim=true 반전 확인
```
