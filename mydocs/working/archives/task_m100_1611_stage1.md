# Stage 1 완료보고서 — Task #1611 (요인 B 근본 메커니즘 확정)

**단계**: 근본 메커니즘 확정 (조사, 코드 수정 없음) · **브랜치**: `local/task1611`

## 결론 (확정)

잔여 −1쪽의 **지배 메커니즘**: 발신명의 footer 블록(`VertRelTo::Page` + `valign=Bottom` +
`wrap=TopAndBottom`, 비-TAC)을 **TypesetEngine 이 stored vpos 에 동기화하지 않고 본문 흐름
위치(flowed `current_height`)에 배치**해, page-fit 판정이 ~30~62px 과소되어 1쪽에 흡수.

## 증거 — `used` vs `vpos하단` 3수치 분해 (36387725)

| 수치 | 값 | 의미 |
|------|----|------|
| footer stored vpos | 48053 HU = **640.7px** | 한컴이 기록한 footer top |
| footer 선언 높이 | 26353 HU = **351.4px** | 정상 측정 |
| vpos 기준 하단 | 640.7+351.4 = **992.1px** | > body 990.2 → 분할되어야 함 |
| rhwp `used` | **929.8px** | footer 를 flowed cur_h(≈578.4)에 배치 → 578.4+351.4 |
| 차이 | 992.1−929.8 = **62.3px** | = 640.7−578.4 (cur_h 가 vpos 보다 62.3px 낮음) |

→ rhwp 가 footer 의 stored vpos(640.7)를 무시하고 본문 흐름 cur_h(578.4)에 배치 → fit
판정 929.4 ≤ 990.2 → 1쪽. vpos(640.7) 동기화 시 992.1 > 990.2 → 분할(한글 2쪽 일치).

## 코드 경로 (root)

`src/renderer/typeset.rs:10434-10468` — TopAndBottom 블록의 `current_height` vpos 동기화는
**`VertRelTo::Paper` 만** 처리(절대좌표 jump). `VertRelTo::Page` 는 누락 → generic fit
체크(`typeset.rs:10503` `current_height + table_total <= available`)가 flowed cur_h 로 판정.

- `VertRelTo::Paper`(10440): cur_h 를 target_y(=stored vpos)로 점프, 표 advance=0(절대).
- `VertRelTo::Para` + RowBreak(10471): 별도 처리.
- **`VertRelTo::Page` + Bottom: 처리부 없음** ← 본 버그.

> Paginator(`pagination/engine.rs:1556` "vert=Page/Paper + Bottom/Center")는 처리가 있으나
> `RHWP_USE_PAGINATOR=1` fallback 전용. 기본 엔진은 TypesetEngine.

## 일반화 — 스폿체크 (잔여 21건)

| 케이스 | footer | vpos | flowed cur_h | diff | 동일 메커니즘 |
|--------|--------|------|------|------|------|
| 36387725 | 1x1 351.4px Page/Bottom | 640.7 | 578.4 | 62.3 | ✓ |
| 36390093 | 1x1 357.2px Page/Bottom | 607.3 | 548.8 | 58.5 | ✓ |
| 36392061 | 1x1 351.4px Page/Bottom | 595.6 | 563.0 | 32.6 | ✓ |
| 36398709 | **tac 대형 표** (hwp_used≈3950 vs rhwp 649) | — | — | — | ✗ (별 요인) |

→ **다수가 footer Page+Bottom 메커니즘**. 단 36398709 처럼 대형 tac 표 과소측정 등 별
요인도 잔존 → 본 수정의 기대 해소는 21건 전부가 아닌 footer 군집.

## 수정 방향 (Stage 2/3, 승인 필요)

`VertRelTo::Page`(+`valign=Bottom`) TopAndBottom 비-TAC 블록의 `current_height` 를 stored
vpos 에 동기화(Paper 핸들러 10443-10468 패턴 확장). **단 Paper 와 달리 페이지네이션에
참여**해야 하므로 advance=0(절대)이 아니라 정상 `table_total` 로 fit 체크 → 초과 시 분할.

설계 골자:
- `is_page_bottom_topbottom_block` = 비-TAC + TopAndBottom + `VertRelTo::Page` + `valign=Bottom`.
- 해당 시 `current_height = max(current_height, target_y)`(stored vpos 존중) 후 기존 fit 체크.
- valign=Bottom 의 페이지 하단 배치(렌더 위치)는 layout.rs Task #295 경로 확인(Stage 3).

## 회귀 위험 (매우 높음)

vpos 동기화를 Page+Bottom 전반에 적용 → 현재 정상 1쪽 문서가 +1쪽으로 회귀할 수 있음.
통제셋 게이트(해소−회귀>0) 필수. Stage 2 RED → Stage 3 수정 → Stage 4 게이트.

## 판정

요인 B 는 요인 A 와 달리 **단일 메커니즘(footer vpos 미동기화)으로 상당수 설명 가능** —
Task #1600 의 "단일 surgical fix 불가" 판정을 부분 정정(footer 군집은 surgical 가능,
대형 tac 표 등 잔여는 별도). Stage 2 진행 승인 요청.
