# Stage 1 완료보고서 — 혼재 케이스 사전 조사 (M100 #1333)

구현계획서: `mydocs/plans/task_m100_1333_impl.md`

## 1. 조사 목적

`build_columns` 의 zone 구분선 emit 을 `col_content.zone_layout.is_some()` 로 한정할 때,
`has_zone_specific_layout = true` 페이지에 `zone_layout = None` 인 다단 cc 가 섞여
구분선이 누락되는 혼재 케이스가 실제 존재하는지 확인.

## 2. 활성 페이지네이터 확인

`src/document_core/queries/rendering.rs:2202-2206` — 기본값 `RHWP_USE_PAGINATOR != 1` →
**TypesetEngine(`src/renderer/typeset.rs`) 가 main pagination**. (pagination engine 은 fallback)

## 3. `current_zone_layout` 생애주기 (typeset.rs)

| 위치 | 동작 |
|------|------|
| `typeset.rs:651` | 초기 `None` |
| `typeset.rs:728,750` (state.rs:91,111) | ColumnContent 확정 시 `zone_layout: current_zone_layout.clone()` |
| `typeset.rs:1191` / `:6295` | **mid-doc 단나누기/구역전환** 으로 새 ColumnDef zone 생성 시 `Some(new_layout)` |
| `typeset.rs:802-810` `reset_for_new_page` | **매 새 페이지에서 `None` 으로 리셋** (단, `self.layout` 은 유지) |

→ `zone_layout = Some` 은 **문서 중간 단나누기/구역전환으로 생성된 zone** 에만 부여.
초기 ColumnDef(문서 첫 단정의) 및 zone 의 **연속 페이지** 는 `None`.

## 4. 실증 데이터

### 4.1 대상 문서 (`3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`)

- `문단 0.0` 의 단정의 2단이 **구역 전체 초기 적용**(단나누기/구역전환 없음).
- 전 페이지 `zone_layout = None`, `self.layout = 2단`.
- 23쪽 모두 page-level(전체높이 90.7→1092.3) + build_columns emit(콘텐츠높이) **이중 렌더**.

### 4.2 shortcut.hwp (`samples/basic/shortcut.hwp`, 단정의 45개 — #874 회귀 대상)

현재 빌드 렌더 결과 (페이지 높이 ≈ 793px, 가로):

| 페이지 | 세로 구분선 | y 범위 / 높이 |
|--------|-------------|----------------|
| 1 | 1개 | y 194.7→474.7 (h=280, **부분**) |
| 2 | 1개 | y 467.3→707.3 (h=240, **부분**) |
| 3 | 2개 | 132.1→244.1, 370.2→556.9 (**서로 다른 zone**, 부분) |
| 4 | 1개 | 548.4→688.4 (h=140, 부분) |
| 5 | 1개 | 170.0→330.0 (h=160, 부분) |
| 6 | 2개 | 118.9→278.9, 490.9→630.9 (**서로 다른 zone**, 부분) |
| 7 | 1개 | 56.7→196.7 (h=140, 부분) |

→ **모든 구분선이 부분 높이** = 전부 **zone emit(`zone_layout = Some`) 출처**.
shortcut.hwp 에서 page-level `build_column_separators` 는 **아무것도 그리지 않음**.
즉 shortcut.hwp 는 현재 **이중 렌더 없음** (가드 정상 작동). 페이지 3·6 의 2개 선은
같은 x(561.3) 이나 y범위가 겹치지 않는 **별개 zone** 으로 정상.

## 5. 결론 — 혼재 케이스

- 대상 문서: 전부 `None` (pure-None page). page-level 이 정답.
- shortcut.hwp: 모든 다단 구분선이 `Some`. page-level 무관.
- **혼재(None 다단 cc + Some cc 동일 페이지) 케이스는 현재 샘플에 존재하지 않음.**
  - 이론상 "2단 zone(Some) 이 다음 페이지로 연속(→ reset 로 None) + 동일 페이지에서
    또 mid-page 단나누기(Some)" 시 발생 가능하나 실 샘플 부재.

## 6. 채택안 (수행계획서 (A)/(B) 중 결정)

**page-level 가드와 동일 술어(`has_zone_specific_layout`)로 두 경로를 완전 상보화** ((B) 정련안).

`build_columns` 의 emit 조건(`layout.rs:2616-2622`)을 cc 단위 `zone_layout.is_some()`
대신 **페이지 단위 `has_zone_specific_layout`** 로 게이트:

```rust
let page_has_zone_specific = page_content
    .column_contents.iter().any(|cc| cc.zone_layout.is_some());
...
if page_has_zone_specific
    && zone_layout.column_areas.len() >= 2
    && zone_layout.separator_type > 0
{ prev_zone_layout_for_sep = Some(...); ... }
```

- 두 경로가 모두 page 술어 `has_zone_specific_layout` 로 키잉되어 **정확히 배타**:
  - false (대상 문서·연속 페이지): page-level 만 그림 (전체높이, 정답). 이중 제거.
  - true (shortcut.hwp): page-level skip, build_columns 만 그림. 혼재 None cc 도
    포함되어 누락 위험 없음.
- 단순 `is_some()` 게이트(A) 대비 **혼재 케이스 무손실** 이점.

## 7. 다음 단계

Stage 2 에서 위 채택안 적용 후, **shortcut.hwp 9개 구분선 전수 보존** + 대상 문서 23쪽
구분선 1개·전체높이를 즉시 재검증한다.
