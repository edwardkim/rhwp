# 단계별 완료보고서 — Task #1853 Stage 1

## 범위

`preceded_by_same_para_float` 술어 정정 + 회귀 게이트 추가 + 검증.

## 변경

### 소스 (`src/renderer/typeset.rs`)

`typeset_block_table` 의 이월 술어를 같은 문단의 진짜 flow 스택 float 으로 한정.
선행 항목의 소스 컨트롤을 `para.controls[control_index]` 로 조회하고
`is_para_topbottom_float`(`!tac && TopAndBottom && vert=Para`)인 표만 선행 float 로 센다.

```rust
let preceded_by_same_para_float = st.current_items.iter().any(|it| {
    let ci = match it {
        PageItem::Table { para_index, control_index }
        | PageItem::PartialTable { para_index, control_index, .. }
            if *para_index == para_idx => *control_index,
        _ => return false,
    };
    matches!(
        para.controls.get(ci),
        Some(Control::Table(t)) if is_para_topbottom_float(&t.common)
    )
});
```

### 게이트 (`samples/issue1853_caption_precedes_body_split.hwpx` + `tests/issue_1853.rs`)

실문서 78842(pi=371 = tac 캡션 ci=0 + 본체 자리차지 표 ci=1) fixture + 테스트 2건.
게이트 유효성: 구(PR#1844) 바이너리에서 캡션 쪽(44)에 본체 없음·총 53쪽 확인 → 버그 포착.

## 검증 결과

| 검증 | 결과 |
|---|---|
| 회귀 3건 페이지수 | 156767631 6→**5**, 78842 53→**52**, 3143097 4→**3** (전부 목표) |
| 별표4(2448877) 캘리브레이션 | 2쪽 불변 |
| float-stack-defer 캘리브레이션 | 2쪽 불변 |
| issue_1853 신규 테스트 | 2건 통과 |
| float/표 테스트 | issue_1156/1488/1510/1549/1639/1663/1748 통과 |
| lib 단위 테스트 | 2073 passed, 0 failed |
| clippy (lib+tests) | 0 warning |
| 통합 테스트 | svg_snapshot 5건 외 전부 통과 — 5건은 CRLF-only(내용 동일, 로컬 autocrlf 노이즈, 메모리 `svg-snapshot-crlf-local-noise`) |
| rustfmt (변경 파일) | 내용 diff 없음 |

## 판단

술어를 **좁히기만** 하여 과잉 이월 3건이 정상 분할로 복귀하고, 기존 정합(별표4·
float-stack-defer)과 전 테스트가 무회귀. Stage 1 완료.
