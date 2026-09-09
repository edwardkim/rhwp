# Task #1557: HWPX 저장본 한글 페이지 붕괴 해소 — 구현 계획서

> 수행계획서 `task_m100_1557.md`(승인 완료). 본 문서는 단계별 구현 계획(3단계).
> 확정 사실: 원인은 저장 `header.xml`(DocInfo). 누락 태그 — `<hc:winBrush>` 22→21,
> `<hh:tabItem>` 2→1, `<hp:switch>/<hp:case>/<hp:default>` 56→55.

## 대상 코드 (조사 완료)
| 방출 | 위치 |
|------|------|
| winBrush (borderFill/fillBrush) | `src/serializer/hwpx/header.rs` (~1489–1517) |
| tabItem (TabDef) | `src/serializer/hwpx/header.rs:708` `write_tab_pr` / `:723` |
| switch/case/default (호환 변형 블록) | `src/serializer/hwpx/header.rs` |
| DocInfo 등록 컨텍스트 | `src/serializer/hwpx/context.rs:108` (CharShape/ParaShape/BorderFill/TabDef/…) |

대표 케이스: `36382669`(PASS, 8→1), `36384160`(d10, 29→1). 산출물 `output/poc/fidelity2/rt/`.

---

## Stage 1 — root-cause element 확정 (조사, 코드 수정 없음)
**목표**: header.xml 의 세 누락 후보 중 **붕괴 유발 element** 를 단정.

- header.xml 내부 bisection(격리 재압축 + 한글 PageCount):
  - rt header 에 원본의 (a)winBrush 블록 / (b)tabItem(TabDef) / (c)누락 switch 블록 을
    하나씩 복원 → 8쪽 복원되는 항목이 원인.
- 다중 케이스 교차 확인(36384160 등 다른 붕괴 파일에서 동일 element 누락·동일 복원인지).
- 원인 element 를 `header.rs` 방출 코드 경로에 매핑 + 누락 메커니즘(off-by-one/조건 누락/
  등록 누락) 1차 규명.
- **산출**: `task_m100_1557_stage1.md`(원인 element + 코드 지점 + 메커니즘). 커밋(보고서만).

## Stage 2 — serializer 수정
**목표**: 누락 element 방출 정합 → 저장 header.xml 이 원본과 동형(해당 항목 보존).

- `header.rs` 의 해당 방출(또는 `context.rs` 등록) 결함 수정. 최소 변경 — 누락 1건 복원에 한정.
- `cargo build --release` + `cargo test`(기존) 회귀 없음.
- **검증(필수)**: 대표 케이스 한글 PageCount 복원 — `rhwp hwpx-roundtrip` 재생성 후
  pyhwpx 로 36382669 **8→8**, 36384160 **29→29** 확인.
- IR diff 불변(수정이 IR 의미 변경 아님): 해당 파일 `hwpx-roundtrip` diff 증가 없음.
- **산출**: `task_m100_1557_stage2.md` + 소스 커밋.

## Stage 3 — 회귀 가드 + 광역 검증 + 최종 보고
**목표**: 재발 봉인 + 효과 정량화.

- 코드 레벨 회귀 가드: 누락되던 element 가 방출되는지 단위 테스트(header.rs 직렬화 단언).
- `cargo test --test hwpx_roundtrip_baseline` 회귀 없음(samples/hwpx 전건 PASS 유지).
- 광역 효과 측정: `hwpx-roundtrip --batch hwpdocs/samples` 재실행 후 T3 페이지 붕괴
  표본(40건) 재측정 — 붕괴율 감소 정량화(목표 0).
- **산출**: `task_m100_1557_stage3.md` + `mydocs/report/task_m100_1557_report.md` + 커밋.

---

## 공통 주의
- HWPX serializer 범위 내 최소 수정. HWP3/HWP5 전용 분기 추가 금지(CLAUDE.md).
- 한글 의존 검증의 한계를 코드 레벨 가드로 보완(한글 미접근 환경에서도 회귀 감지).
- pic 드롭(V2-B)·F3(#1556)은 본 타스크에서 건드리지 않음(혼합 금지).
- 단계마다 완료보고서 + 소스 동반 커밋, 승인 후 다음 단계(자동승인 시 연속 진행).
