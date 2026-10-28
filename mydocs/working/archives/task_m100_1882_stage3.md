# task_m100_1882 Stage 3 완료 보고서 — 갭① 자동 제목

- 이슈: #1882 (C1c, #1431 Track C)
- 브랜치: `local/task1882`
- 단계: Stage 3 / 5 — 자동 제목 (`src/ooxml_chart/{mod,parser,renderer}.rs`)

## 변경 내용

1. **모델** (`mod.rs`): `OoxmlChart`에 `has_title_elem: bool` / `auto_title_deleted: bool` 추가.
   `chart.title`은 **명시 텍스트 전용 유지** → 파서의 빈 차트 조기 반환 가드
   (`series.is_empty() && title.is_none()`) **무변경** — 시리즈 없는 XML이 fallback으로
   새는 회귀를 원천 차단 (구현계획서 승인 사항).
2. **파서** (`parser.rs`): `b"title"` start에서 `has_title_elem=true` 병기,
   `b"autoTitleDeleted"` arm 신설 (`val ∈ {1, true}` → 억제).
3. **렌더러** (`renderer.rs`): `effective_title = chart.title.or("차트 제목" if
   has_title_elem && !auto_title_deleted)` — title_h 분기·텍스트 렌더 모두 이 기준.
   `font-weight` 600→**400** (한컴 제목은 regular weight, 정답지 PDF 실측).

근거: 코퍼스 27종 전부 `c:title` 요소 존재(텍스트 없음) + `autoTitleDeleted=0`인데
한컴은 자동 제목 placeholder **"차트 제목"**을 상단 중앙에 그림 (수행계획서 §2 실측).

## 테스트 (TDD — 구현 전 실패 확인)

- parser 신규 2 + 기존 1 보강: `test_parse_title_elem_without_text`(텍스트 없는 c:title →
  `title=None` 유지 + `has_title_elem=true`), `test_parse_auto_title_deleted`(val=1 → 억제),
  `test_parse_bar_chart`에 명시 제목도 요소 플래그 기록 단언 추가.
- renderer 신규 2: `test_render_auto_title_placeholder`("차트 제목" 출력 +
  `font-weight="600"` 미포함), `test_render_no_auto_title_when_deleted_or_absent`
  (autoTitleDeleted=1 / 요소 부재 → 자동 제목 없음).

## 검증

```
cargo test --lib ooxml_chart                              → 40 passed, 0 failed
cargo test --test issue_1431_scatter                      → 1 passed
cargo test --test issue_1453_chart_3d_ofpie_routing       → 2 passed
```

**시각 확인** (`output/poc/chart_c1c/stage3/` ↔ 정답지): 묶은세로막대형에 "차트 제목"이
상단 중앙 regular weight로 렌더 — 정답지와 일치. 잔여 차이는 범례 하단(Stage 4)뿐.

## 다음 단계

Stage 4 — 갭③ 범례 우측: `LegendPos` enum + `legendPos` 파싱 + 우측 세로 스택 배치.
