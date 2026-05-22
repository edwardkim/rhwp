# 구현계획서 — Task #1079: 그림 pushdown ↔ 파일 vpos 이중 계상 정정

- 이슈: edwardkim/rhwp#1079
- 브랜치: `local/task1079` (stream/devel `be2a71c4` 기준)
- 수행계획서: `task_m100_1079.md` (승인) / Stage 1: `working/task_m100_1079_stage1.md` (승인)

## 목표
비-TAC TopAndBottom(vert=Para) 그림에서, 파일 vpos 가 이미 그림 공간을 반영한 경우 typeset/
렌더가 그림 높이를 **추가로** 누적(pushdown)하지 않도록 정정 → pr-149 1페이지 수용
(`pdf/pr-149-2022.pdf` 정합), #409 계열(파일 vpos 미반영) 회귀 0.

## 핵심 조건 (Stage 1 도출)
그림 para 의 first_vpos 앞 gap 이 그림 높이 이상이면 파일이 이미 그림을 반영:
```
gap_before = V[pi] - (V[pi-1] + line_height[pi-1])
if gap_before >= picture_height - tol  →  pushdown 생략(파일 vpos 사용)
else                                    →  현행 pushdown 유지(#409)
```
- pr-149 pi=2: gap≈217px ≥ 그림 209px → 생략. #409: gap 작음 → 유지.
- typeset(`typeset.rs:1311~1352 pushdown_h`) 와 렌더(`picture_footnote.rs` y_offset+total_height)
  **동일 조건** 적용(양 레이어 정합).

## 단계 (Stage 2~4)

### Stage 2 — 설계 + 페이퍼 검증 (소스 무변경)
- gap_before 조건 정밀화: 직전/현 그림 para vpos 접근 경로, tol, col_count==1 한정(다단 제외)
  확정. 그림이 para 첫 컨트롤이 아닌 경우/다중 그림 문단 경계 명시.
- typeset·렌더 적용 지점 + 데이터 흐름 확정.
- #409 비회귀 케이스(파일 vpos 미반영 그림) + Square wrap/비-TAC 블록표와 모순 점검 표.
- 산출물: `working/task_m100_1079_stage2.md`.

### Stage 3 — 구현 (typeset + 렌더 양 레이어)
- `typeset.rs` pushdown_h 를 gap_before 조건으로 게이트.
- 렌더 그림 pushdown(picture_footnote.rs) 동일 조건 게이트.
- 검증: pr-149 used ≤ body, 1페이지, overflow 0(typeset+렌더 정합). dump-pages + RHWP_TYPESET_DRIFT.
- 전수 sweep 1차(그림 보유 문서 회귀 측정).
- 산출물: 소스 + `working/task_m100_1079_stage3.md`.

### Stage 4 — 회귀 검증 + 가드
- pr-149 1페이지 + overflow 0. 전수 sweep LAYOUT_OVERFLOW 합 회귀 0. 골든 8,
  `cargo test --release`, clippy/fmt. 회귀 가드 `tests/issue_1079_*.rs`(pr-149 1페이지 + 그림
  하단 page 내 + #409 계열 보호 케이스).
- 산출물: `working/task_m100_1079_stage4.md` → `report/task_m100_1079_report.md`.

## 완료 기준
pr-149 1페이지 수용 + overflow 0(typeset·렌더 정합) + #409 비회귀 + 전수 sweep 회귀 0 + 가드.

## 리스크
- pushdown 은 그림/도형 보유 전 문서 공유 경로 → 광범위 회귀. gap_before 조건이 #409 케이스를
  정확히 보존하는지 Stage 3 전수 sweep + 그림 문서 표적 검증.
- 렌더와 typeset 의 vpos/pushdown 모델 미세 차이 → 동일 조건이라도 결과 불일치 가능. Stage 3
  에서 pr-149 typeset used == 렌더 max_y 정합 확인.
