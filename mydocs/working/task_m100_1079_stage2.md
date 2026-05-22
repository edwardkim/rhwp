# Stage 2 보고서 — Task #1079: 설계 + 페이퍼 검증

- 브랜치: `local/task1079` (소스 무변경)

## 핵심 통찰 (파일 vpos 자체가 pushdown 포함)
pr-149 그림 para(pi=2) line vpos=251.9px. 직전 원본:(pi=1) 끝 ≈34.6px. 그림 높이 209px.
→ 파일이 그림을 **34.6~243px(원본 아래)** 에 두고 그림 para 의 텍스트 줄을 **그림 아래(251.9)**
에 인코딩. 즉 **파일 vpos 가 이미 그림 pushdown 을 반영**. VPOS_CORR sync 가 이를 정확히 따름.
typeset `pushdown_h`(1350) + 렌더 `base_y+total_height`(picture_footnote.rs:473) 가 그림 높이를
**한 번 더** 더해 이중.

## 판정 신호 — gap_before
```
prev_end = V[pi-1] + line_height[pi-1]      (직전 문단 끝, px)
gap_before = V[pi] - prev_end               (그림 para 줄 앞 빈 공간)
already_accounted = gap_before >= picture_height - TOL
```
- pr-149 pi=2: gap≈217 ≥ 209 → already_accounted=true → **pushdown 생략**.
- #409 계열(그림이 para 줄 아래, 파일 vpos 미반영): gap 작음 → false → **pushdown 유지**(회귀 0).
- TOL: 라인/여백 미세차 흡수(예 8px). col_count==1 한정(다단 Stage E 미적용).

## 적용 지점 (양 레이어 동일 조건)
1. **typeset** (`typeset.rs:1311~1352`): `pushdown_h` 가산을 `!already_accounted` 로 게이트.
   메인 루프 `for (para_idx, para) in paragraphs` 안이라 `paragraphs[para_idx-1]` vpos 접근 가능.
2. **렌더** (`picture_footnote.rs:471~473`): `(VertRelTo::Para, _) => base_y + total_height` 를
   already_accounted 시 후속 콘텐츠가 파일 vpos 를 쓰도록 변경(그림 높이 미가산). 조건은
   호출부(문단 컨텍스트 보유)에서 산출해 인자로 전달 또는 동일 데이터로 재계산.

## 페이퍼 검증 (모순 점검)

| 케이스 | gap_before | pushdown | 동작 |
|------|-----------|----------|------|
| pr-149 (파일 vpos 그림 반영) | ≥ 그림높이 | **생략** | 1페이지(타깃) |
| #409/AI 184p (파일 vpos 미반영) | < 그림높이 | 유지 | 불변(회귀 0) |
| TAC 그림(treat_as_char) | pushdown_h 대상 아님 | — | 불변 |
| Square wrap / 비-TAC 블록표 | pushdown_h 분기 밖 | — | 불변 |
| Page/Paper 기준 그림 | vert_rel_to≠Para | — | 불변(플로팅) |
| 다단(col_count>1) | 조건 비적용 | 유지 | 불변 |

## 리스크 / Stage 3 선결
- **typeset used == 렌더 max_y 정합**: pr-149 에서 두 레이어가 동일 조건으로 그림 높이를 빼야
  page-break(typeset)와 그리기(렌더)가 어긋나지 않음. Stage 3 에서 used·max_y 동시 확인.
- 렌더 base_y/y_offset 의미: already_accounted 시 반환값을 y_offset(앵커) 으로 둘지 base_y 로
  둘지 Stage 3 에서 pr-149 시각 정합(그림이 원본:↔회색조: 사이)으로 확정.
- 전수 sweep 으로 그림 보유 문서 회귀 측정(특히 #409 계열 보존).

## 다음 (Stage 3)
typeset pushdown_h + 렌더 picture pushdown 양 레이어 게이트 구현 → pr-149 1페이지·overflow 0
(used·max_y 정합) + 전수 sweep 1차.
