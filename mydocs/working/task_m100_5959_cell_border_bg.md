# Task M100 — #5959 잔여: 셀 테두리/배경 다이얼로그 역연산화 판정

- 일자: 2026-08-24
- 상위: #5959 잔여 과제 (#5769 원장의 "⑤ cell-border-bg-dialog — 후순위, 스냅샷 진입점의 1.2%")
- 선행 규약: #5769 참인 역연산 판정(변경 실체가 속성 대입뿐이어야 한다) + 저장 바이트 왕복 동일성

## 대상

`CellBorderBgDialog`(ui/cell-border-bg-dialog.ts)의 적용 경로 3종 — 모두
`ih.executeOperation({ kind:'snapshot', operationType:'objectProps' })` 로 기록되어
적용 1회당 스냅샷 슬롯 1개를 쓴다.

| 진입 | 적용 네이티브 | 대상 |
|---|---|---|
| 각 셀마다 적용(each) | `setCellProperties` × N (`runInBatch`, #4118) | 선택 셀 전부 |
| 하나의 셀처럼 적용(asOne) | `setCellZoneProperties` | 셀 zone |
| 단일 셀 | `setCellProperties` | 커서 셀 |

## 변경 실체 분석 (선결 규약 판정)

### set_cell_properties_native (table_ops.rs:1335)

테두리/배경 키(borderLeft~ / fillType / diagonal* / centerLine)가 들어오면:

1. `create_border_fill_from_json`(html_table_import.rs:785) — JSON 으로 BorderFill
   조립 후 **기존 테이블에서 동일 항목 dedupe**(border_fills_equal), 없으면 append.
   raw_data 는 비운다(직렬화 필드 경로 강제).
2. 대상 셀 `border_fill_id = 새 id`.
3. `update_neighbor_borders`(table_ops.rs:1819) — 공유 엣지를 가진 **이웃 셀마다**
   이웃의 BorderFill 을 복제해 해당 방향만 교체 → dedupe/append → 이웃 id 재배정.
4. `doc_info.raw_stream_dirty = true`(append 시), 섹션 `raw_stream = None`,
   rebuild_resolved_styles, 재조판.

→ 변경 실체는 **셀 1개의 id 대입이 아니라 "대상+이웃 집단의 id 재배정 + 문서 스타일
테이블 성장"**이다. z 순서(moves[] 선례)와 달리 영향 집단이 Rust 내부에서만 결정된다.

### set_cell_zone_properties_native (table_ops.rs:1746)

bf 생성(dedupe) → zone 신설 또는 기존 zone id 교체(table.zones.push 포함) → raw 무효화.

## 판정 — 3개의 장벽

1. **영향 집단의 Rust 전용 결정**: 이웃 셀 목록/zone 은 코어가 계산한다. TS 가
   before 를 미리 모은다(all-undo 선례)로는 부족하고, z-order 의 moves[] 처럼
   **응답에 self-describing 기록**(affected cells/zones 의 before/after id)을 얻어야 한다.
2. **BorderFill 테이블 성장 vs 저장 바이트 수렴**: serializer/doc_info.rs:212 는 테이블
   전체를 쓴다. apply 가 append 하면 undo(id 복원) 후에도 고아 항목이 남아 원본 바이트와
   어긋난다. 현재 스냅샷은 saveSnapshot 이 doc_info 를 통째로 복원하므로 지금은 수렴한다 —
   역연산화가 오히려 수렴을 깨는 역전 지점.
3. **DocInfo passthrough**: append 는 `raw_stream_dirty` 를 세운다(#2555). undo 가
   수렴하려면 이 플래그까지 원상 복구돼야 한다.

## 설계안

Stage A — **Rust 응답 확장**: 두 네이티브가 적용 직전 before 를 스스로 캡처해
`changes:[{ppi,ci,cellIdx?,zone?,beforeId,afterId}]` + `createdBorderFills:[id…]`
(이번 호출로 push 된 항목만)를 반환한다. 무변경(dedupe 후 동일 id)은 changes 공백.

Stage B — **TS 커맨드**: `SetCellBorderFillCommand` — execute 는 네이티브 응답으로
pairs 확보(z-order 선례), undo 는 **직접 id 대입 네이티브**(신규,
`apply_cell_border_fill_ids`)로 before 복원 → 구역 raw 캡처·복원 재사용.
redo 는 execute 재실행.

Stage C — **고아 GC**: `discard_unused_border_fill_tails(ids)` — created 항목 중
"꼬리(tail)이면서 참조 0"인 것만 LIFO 절단. 중간 항목은 절단 불가(id shift 위험).
undo 마다 호출, 절단 못한 고아가 남으면 게이트가 잡는다.

Stage D — **게이트**: `issue_5959_cell_borderfill_inverse_convergence.rs` —
실제 표 문서에서 테두리/배경 적용 → undo → 저장 바이트 왕복 동일성 + border_fills
길이 원복 + 이웃 셀 id 원복. e2e 소스 가드(배선 핀) 추가.

## 리스크

| 리스크 | 대응 |
|---|---|
| redo 가 dedupe 로 다른 id 재사용 | after 도 기록해 redo 는 after 직접 대입(execute 재실행 아님) |
| 중간 삽입 편집으로 꼬리 절단 불가 | 저널 계약(캡처~복원 사이 편집 금지)과 TS 선형 히스토리로 대부분 방어, 잔존 시 게이트 실패로 드러냄 |
| '각 셀마다' N셀 × 이웃 폭발 | changes 배열은 1회 배치로 모음 — 여전히 슬롯 0 |

## 예상 규모

Rust 응답 확장+신규 네이티브 2건(~200줄), TS 커맨드+배선(~150줄), 게이트 .rs+e2e 가드.
followup2(section-all)급 — 단, Stage C 가 serializer 계약(#2555)과 맞닿으므로
구현 전 이 문서 기준으로 착수 승인을 받는다.
