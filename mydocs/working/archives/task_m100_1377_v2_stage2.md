# Task #1377 v2 Stage 2/3 — 전체 회귀 가드 + PDF 검증 + 판정(보류)

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 2(가드) + Stage 3(판정). narrowing 3회로 19→8 회귀 축소했으나 0 미달 → **보류·revert**.
- **결과**: 코드 Stage5 로 revert(green 123/0). 단 **PDF 측정으로 fix 방향이 정답임을 입증**하고
  잔여 8건의 정체(섹션 between-notes 마진 의미)를 규명 — v3 의 명확한 출발점 확보.

## 1. PDF 검증 — sep2020 은 compact 가 정답 (중요)

`pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`(한글 2022) p22 좌단, 공통 앵커 `[출제의도]`로 정렬:

| 항목 | rhwp(현 render) | PDF | compact(목표) |
|------|------|------|------|
| 문29→출제의도 갭 | 18.7px | 18.56px(13.92pt) | (문단 내) ✅ 정합 |
| 문29 top y | 166.7px(=gap유지) | **148.1px** | **146.3px** |

→ render 는 문29 를 PDF 보다 **~18px 낮게**(gap 유지) 배치. compact(51.7) top=146.3 ≈ PDF 148.1.
**compact 가 PDF 정답** 확정(typeset 51.7 이 옳고 render gap 이 #1246 과대). #1377 의 render-버그 전제 재확인.

## 2. narrowing 여정 (3회 시도, 19→8)

| 시도 | 게이트/치환 | 회귀 | sep2020 |
|------|------|------|---------|
| Stage5 universal clamp | 전 미주 para → typeset acc | 19 | fix |
| v2 Stage1 (paragraph_layout) | eqonly+ls≈marker+Percent → base-Percent 치환 | 14 | fix |
| v2 adj1 (build_single_column) | eqonly+ls≈marker → typeset 채널값 clamp | 14 | fix |
| v2 adj2 (+vpos-delta 게이트) | +파일 raw saved-vpos delta<marker/2 | **8** | fix |

- **vpos-delta 게이트가 6건 제거**: 2024 between20/미주사이20 p13/18/19/21/22-23 등은 파일 saved-vpos
  에 실제 갭(큰 delta)이 있어 compact 제외 → 무회귀. sep2020(delta=452HU<992)만 compact.

## 3. 잔여 8건의 정체 (#1246 비대칭 + 마진 의미)

남은 8건 = **2022/2023 문서**(issue_1189/1209/1256/1274/1284 계열). 공통점:
- 파일 saved-vpos delta 가 **작은데도**(게이트 통과) PDF 는 between-notes 갭을 **보존**.
- 이는 `tech_trailing_model_no_ssot` 의 **#1246 비대칭**: between-notes 마진이 vpos 에 인코딩되지
  않고 **render 시점**에 가산됨 → saved-vpos 는 작지만 PDF 엔 갭.
- **sep2020 과의 차이(가설)**: sep2020 샘플 = `구분선아래20구분선위20`(구분**선** 마진, inter-note
  갭≈0 → compact 정답). 8건 = `미주사이20`/2022·2023 표준(inter-note 마진 20mm → 갭 정답).
  즉 **섹션의 between-notes 마진 의미(구분선 마진 vs 미주사이 마진)**가 진짜 구분자일 수 있음.
  현 게이트는 `endnote_between_notes_hu` 만 보고 둘을 구분 못함.

## 4. 판정 (Stage 3) — 보류·revert

- 회귀 0 절대조건 미달(8건). 플랜의 "미세조정 1회"를 3회 초과 → 규율상 중단.
- 코드 Stage5 로 revert. 전체 테스트 **123/0 green** 유지.

## 5. v3 출발점 / 권고

- **fix 방향은 PDF 로 입증됨**(sep2020 compact=정답). 잔여는 sep2020(갭0)과 8건(갭20mm)을
  가르는 **섹션 between-notes 마진 의미 판별**이 관건.
- v3 후보: `endnote_between_notes_hu` 가 **구분선 마진 기원인지 미주사이 마진 기원인지** 구분
  (섹션 EndnoteShape/footnote 설정 파싱) → 구분선-기원(inter-note 갭 없음)일 때만 compact.
  실패 시 #1246 의 documented 한계로 종결.
