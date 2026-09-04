# 구현계획서 — #2015 HWPX saved-bounds RowBreak 표 잔존 드리프트

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow` (base `origin/devel`)
- 수행계획서: `mydocs/plans/task_m100_2015.md`
- 원칙: 소스 무수정 분석 완료. 각 단계는 **자체 진단(`LAYOUT_OVERFLOW`)+회귀 테스트**로 검증. 하드코딩 분기 금지.

## 대상 코드 (규모 주의)

| 파일 | 줄수 | 역할 |
|---|---:|---|
| `src/renderer/typeset.rs` | 15910 | fit·`used`(current_height) 회계, saved-bounds fit 판정 |
| `src/renderer/layout/table_layout.rs` | 7558 | PartialTable 프래그먼트 실측 높이/`end_cut` |
| `src/renderer/composer.rs` | 1883 | HWPX 합성 lineSeg 생성 |

## 단계 (5단계)

### Stage 1 — 기준선 하네스 + 코드 국소화 (소스 무수정)
- 목표: 재현을 고정하고 오차 발생 함수를 **정확히** 특정.
- 산출: `tests/issue_2015_saved_bounds_rowbreak.rs` — p3(4쪽) 렌더 시 `LAYOUT_OVERFLOW(para=52)` 부재를 기대하는 실패 테스트(현재 red) + `used` vs 실측 바닥 계측.
- typeset의 pi=52 부동 표 높이 회계 지점과 layout의 프래그먼트 실측 높이 지점을 코드 인용으로 문서화(`working/task_m100_2015_stage1.md`).
- 커밋: 테스트 + stage1 문서.

### Stage 2 — 발원지 ① 부동 RowBreak 표 오버플로우 소거 (포맷 공통)
- typeset `used` 회계가 부동(tac=false) RowBreak PartialTable 프래그먼트의 **실측 row-cut 높이**를 반영하도록 정합.
- body 바닥(1026.5px) 초과 금지. `end_cut` 이 페이지 잔여공간에 맞게 산출되는지 확인.
- 검증: `para=52` OVERFLOW 소거, `issue_1749_*`/`issue_1035`/`issue_1139` 회귀 유지, clippy.
- 커밋: 소스 + stage2 문서.

### Stage 3 — 발원지 ② HWPX 인라인 표 합성 lineSeg 줄 피치 (HWPX 전용)
- pi=50 합성 lineSeg 내부 별표 줄 피치(문단 내 1818HU / 문단 사이 +500HU 교대)를 한컴 균일 렌더에 정합.
- HWP 경로(B=1 저장 lineSeg)는 불변, HWPX 합성 경로만 보정.
- 검증: `export-render-tree -p 3` y좌표가 PDF 균일 피치에 근접, ir-diff 회귀 없음.
- 커밋: 소스 + stage3 문서.

### Stage 4 — 회귀·시각 검증
- `cargo test --profile release-test --tests` 전량, `cargo clippy --all-targets -- -D warnings`.
- p2 부수 `pi=26 overflow=1.6px` 동반 해소 여부 확인(같은 계열이면 포함, 아니면 별도 기록).
- 시각: 래스터 도구 가용 환경에서 p4/p5 `ink_match` 기준선 대비 개선 계측(불가 시 render-tree y좌표 정합으로 대체 판정).
- 커밋: stage4 문서.

### Stage 5 — 최종 보고 + 오늘할일
- `mydocs/report/task_m100_2015_report.md` 작성(before/after 수치, 회귀 결과, 잔여 리스크).
- 오늘할일(`orders/`)은 불가침 메모리 룰에 따라 수정하지 않음(참조만).
- merge 전 `git status` 클린 확인.

## 완료 기준

1. `para=52 PartialTable` `LAYOUT_OVERFLOW` 소거(body 바닥 미초과).
2. HWPX 인라인 표 줄 피치 한컴 정합.
3. 기존 회귀 전량 유지, clippy 무경고.
4. p4/p5 잉크 정합(또는 render-tree y 정합) 개선.

## 리스크

- 16k줄 엔진의 saved-bounds fit 로직은 다수 샘플이 얽힘 → 각 단계마다 광범위 회귀 필수.
- ①/② 상호작용: ① 높이 정합이 ②의 후속 문단 vpos 기준점을 바꿀 수 있음 → Stage 3에서 재계측.
- 완전 정합이 어려우면 오버플로우 소거(①)까지 확정하고 ②는 후속 이슈로 분리 가능(보고서에 명시).
