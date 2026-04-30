# Task #471 최종 결과 보고서

## 이슈

- 번호: #471
- 제목: Task #468 cross-column 검출이 stroke_sig 머지와 불일치 — bf_id 비교로 partial_end 미설정
- 마일스톤: M100
- 브랜치: `local/task471`

## 증상

Task #470 적용 후 발견. `samples/21_언어_기출_편집가능본.hwp` 페이지 1 좌측 단 (가) 본문 박스(pi=7~9) 하단에 가로선 그려짐. PDF 는 cross-column 으로 우측 단 pi=10~12 로 이어지므로 하단 미그려야 함.

`/tmp/p21_post/...svg` line 94:
```xml
<rect x=128.5 y=558.3 w=402.5 h=879.9 fill="none" stroke="#000000" stroke-width="0.5"/>
```
4면 stroke 단일 Rectangle (bottom 포함).

## 근본 원인

`src/renderer/layout.rs` paragraph border 머지 (Task #321 v6) 와 cross-column 검출 (Task #468) 의 비교 기준 불일치:

- **머지**: `stroke_sig` (line_type/width/color) 기준 — bf_id 가 달라도 visual 동일하면 한 그룹.
- **Task #468**: `bf_id` 동등 비교 — 머지 후 그룹의 `g.0` 은 첫 range 의 bf_id 만 보존.

본 케이스 디버그:
```
group bf=7 first_pi=6 last_pi=9 next_bf=4   ← 4 != 7 → partial_end 미설정 ❌
group bf=4 first_pi=10 last_pi=12 prev_bf=4 ← 4 == 4 → partial_start 정상 ✓
```

좌측 단 그룹: pi=6(bf=7, "(빈)") + pi=7~9(bf=4, 본문) 머지. `g.0=7`. composed[10].bf_id=4. bf_id 비교로 4 != 7 → partial_end 미설정 → `skip_bottom=false` → 4면 단일 Rectangle (L1745) 로 그려져 하단 가로선 발생.

## 수정

`src/renderer/layout.rs:1670-1699` Task #468 검출을 stroke_sig 비교로 변경:

```rust
let group_sig = stroke_sig(bf_id);
let para_bf = |pi: usize| -> u16 { ... };

if !g.7 && first_pi > 0 {
    let prev_sig = stroke_sig(para_bf(first_pi - 1));
    if prev_sig.is_some() && prev_sig == group_sig {
        g.7 = true;
    }
}
if !g.8 {
    let next_sig = stroke_sig(para_bf(last_pi + 1));
    if next_sig.is_some() && next_sig == group_sig {
        g.8 = true;
    }
}
```

`stroke_sig` 헬퍼 (이미 L1621 에 정의됨, Task #321 v6 머지에서도 사용) 를 재활용. 그룹의 첫 range bf_id 가 다르더라도, visual stroke 가 같은 인접 paragraph 와 정확히 매칭.

## 추가된 테스트

`src/renderer/layout/integration_tests.rs::test_471_cross_column_box_no_bottom_line_in_col0`

- `samples/21_언어_기출_편집가능본.hwp` 페이지 1 SVG 의 좌측 단 영역 (x ∈ [120, 542]) 에서 stroke 가 있는 4면 rect 의 bottom_y > 1300 부재 검증
- 수정 전: FAIL (rect ends_y ≈ 1438) → 수정 후: PASS

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 단위 테스트 | PASS |
| 전체 cargo test (1123건) | 1123 / 1123 PASS |
| 21_언어_기출 OVERFLOW (10) | 변동 없음 |
| exam_kor / exam_eng / hwpspec / 등 OVERFLOW | 변동 없음 |
| 좌측 단 (가) 박스 시각 확인 | 하단 가로선 미렌더 ✓ |
| 우측 단 (가) 박스 시각 확인 | 상단 가로선 미렌더 (변동 없음) |

## 영향 범위

- paragraph border 머지로 다른 bf_id 가 같은 stroke_sig 로 묶인 그룹의 cross-column/cross-page 인접 검출 정확도 개선.
- 단일 컬럼/페이지 내 paragraph border 는 영향 없음 (그룹 머지 후 단일 rect 그대로).
- Task #468 이후 cross-column 박스에서 본 회귀 가능성 잠재. 본 task 가 정확한 visual sig 비교로 정정.

## 산출물

- `src/renderer/layout.rs` (Task #468 검출 블록 stroke_sig 비교로 변경)
- `src/renderer/layout/integration_tests.rs` (테스트 1건 추가)
- `mydocs/plans/task_m100_471{,_impl}.md`
- `mydocs/working/task_m100_471_stage{1,2,3}.md`
- `mydocs/report/task_m100_471_report.md`
