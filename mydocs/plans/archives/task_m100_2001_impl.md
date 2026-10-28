# 구현계획서 — Task M100 #2001: parse_paragraph_list 해체 (라운드 3)

- 이슈: #2001 / 수행계획서: `task_m100_2001.md` / 작성일: 2026-07-06
- 1단계 산출: 도구 소품 3건 커밋(`f57a7104`) + 아래 의존 실측.

## 1. 의존 실측 결과

컨트롤 코드 `match ch`의 **11개 arm이 동일한 mut 캐리오버 5종 세트를 공유**한다
(text_string/char_offsets/utf16_len/controls/ctrl_data_records + 문자 인덱스 `i` +
`hwp3_char_to_utf16_pos`). 함수 파라미터군(doc_* 5종 + body_cursor + 기하 3종)은 그대로
전달 가능. 후처리는 별도의 flow 상태(mut 12)를 가진다.

| 블록 | 크기 | 읽기 | mut | 판정 |
|---|---|---|---|---|
| `_` arm (GSO/개체: 표·글상자·수식·버튼) | 1,116줄 | 10+파라미터군 | 5+i | **추출 1** — 스캔 상태 struct로 정리 |
| 소형 arm 10개 (9/18..=21/22/23/24\|25/26/28/30\|31/7\|8/1) | ~330줄 | 7 내외 | 동일 5종 | **추출 2** — 같은 struct 재사용 |
| 후처리 (line_infos/스타일/wrap-zone 조립) | 694줄 | 20 | **12** | **이연** — v2 §6 임계(mut 10↑) 초과, `Hwp3FlowState` 설계 선행 필요 (다음 라운드 입력) |

## 2. 추출 설계

### 공유 상태 struct (신규, `src/parser/hwp3/` 내부)
```rust
/// [#2001] parse_paragraph_list 문자 스캔의 공유 가변 상태 — match ch 의 11개
/// arm 이 공유하는 캐리오버 묶음.
struct Hwp3CharScan<'a> {
    text_string: &'a mut String,
    char_offsets: &'a mut Vec<usize>,
    utf16_len: &'a mut usize,
    i: &'a mut usize,
    hwp3_char_to_utf16_pos: &'a mut [usize],   // (실측 타입 확인 후 확정)
    controls: &'a mut Vec<Control>,
    ctrl_data_records: &'a mut Vec<...>,       // (실측 타입 확인 후 확정)
}
```
(대안: 소유 이동이 어려우면 개별 `&mut` 파라미터 7개 — struct 실패 시 폴백. 어느 쪽이든
동작 불변 원칙 유지.)

### 추출 1 (수행계획 2단계) — `parse_object_control_char` (1,116줄)
- `_` arm 본문 통추출. 파라미터: `body_cursor`, doc_* 5종, `pic_name_to_id`, 기하 3종
  (body_left_hu 등), `ch`, `para_info`, + `Hwp3CharScan`.
- HWP3 격리 영역 내부 이동 — 공통 모듈 무접촉. 내부의 ch==10 표 블록(861줄) 2차 분해는
  통추출 후 별도 커밋으로 재평가(이번 라운드에서는 강행하지 않음).

### 추출 2 (수행계획 3단계) — 소형 arm 헬퍼화 (~330줄)
- 유사 성격 묶음 3~4개 함수로: 필드/북마크 계열(18..=21), 탭/공백 계열(9/30|31),
  하이픈·묶음 계열(24|25/26/28), 선/페이지 계열(7|8/22/23/1). 정확한 묶음은 arm 정독 후
  확정하되, 모두 `Hwp3CharScan` 재사용.

### 게이트 (매 추출)
fmt/clippy 0 · 전체 테스트 FAILED 0 · OVR 5샘플 "추가 변동 0"(기지 #1936발 3건 동일) ·
hwp5_roundtrip_baseline · 페이지 오라클 무변동.

## 3. 예상 효과

`parse_paragraph_list` 2,270 → **~800줄대**, CC 234 → 대폭 감소(분기의 대부분이 arm 내부).
전체 1위 함수 3연속 해소 (282→104, 288→226, 234→?).

## 4. 이연 기록 (다음 라운드 입력)

- 후처리 694줄: `Hwp3FlowState`(acc_section_vpos, active_wrap_zone/cs_sw,
  wrap_zone_end_vpos, prev_last_pgy, prev_para_had_flags_break) 설계 —
  `EndnoteFlowState`/`RunEmitState`와 같은 mut-묶음 패턴 3호.
- ch==10 표 블록 내부 재분해 (통추출 후 상태에서 재평가).
