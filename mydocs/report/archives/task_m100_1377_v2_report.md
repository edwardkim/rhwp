# Task #1377 v2/v3 최종 결과보고서 — 미주 빈 수식 spacer 발산: fix 방향 입증, 구분 신호 부재로 종결

- **이슈**: #1377 (M100) / 브랜치 `local/task1377` (base devel)
- **성격**: 미주 단 render fidelity — Stage5(universal clamp 불가) 이후 좁힌 재도전(v2) + 마진 의미 조사(v3)
- **결론**: sep2020 빈 수식 spacer 의 **compact 가 PDF 정답임을 3중 입증**(픽셀·SVG·기존 14테스트).
  그러나 sep2020(compact)과 gap-정답 문서(미주사이20·2022/2023 8건)를 **가를 신뢰 신호가
  parsed 데이터에 부재** → `tech_trailing_model_no_ssot` 의 #1246 입력 모호성으로 **종결**.
- **코드 상태**: Stage5 (기록 인프라 유지·clamp 보류). 전체 테스트 **123/0 green**.

## 1. 발산 정밀 진단 (v2)

sep2020 미주 단0: 발산은 **pi=1128(빈 TAC-수식 spacer) 한 곳**에서 발생, 이후 +20px offset 상속.
- pi=1128 line_seg `ls=1984HU`(=`endnote_between_notes_hu`, between-notes 마커). render 가
  `paragraph_layout` 에서 lh(27.6)+ls(26.45)=54.1px advance. typeset saved-vpos = lh+452 = 33.6px.
- 1984 는 typeset.rs:2381 이 미주 경계 직전 para 마지막 줄 line_spacing 에 **stamp** 한 값
  (`endnote_between_notes_margin = shape.raw_unknown`).

## 2. fix 방향 = PDF 정답 (3중 입증)

`pdf/...구분선아래20구분선위20.pdf`(한글 2022) p22:
- **픽셀**: PDF 문29 행 y=149(=yMin 111pt) vs rhwp 문29 y=176 → render **27px(≈7mm=1984HU) 낮음**.
- **SVG/PDF bbox**: 문29 top render 166.7 vs PDF 148.1 (±compact 146.3) → compact 일치.
- **문단 내 정합**: 문29→출제의도 갭 rhwp 18.7 = PDF 18.56px(앵커 검증).
→ render 가 1984 갭을 적용하나 PDF 미적용. **compact 가 정답**. (typeset 51.7 이 옳고 render 가 #1246 과대.)

## 3. narrowing 4단계 — 회귀 19→8, 그리고 신호 부재

| 단계 | 게이트/방법 | 회귀 | sep2020 |
|------|------|------|---------|
| Stage5 universal clamp | 전 미주 para → typeset acc | 19 | fix |
| v2 base-Percent 치환 | eqonly+ls≈marker+Percent | 14 | fix |
| v2 typeset-채널 clamp | +typeset advance 값 | 14 | fix |
| v2 +vpos-delta 게이트 | +파일 raw saved-vpos<marker/2 | **8** | fix |
| v3 마진 의미 | FootnoteShape 필드 구분 | (신호 부재) | — |

- vpos-delta 게이트가 2024 미주사이/between20 **6건**(파일에 실제 갭) 올바르게 제외.
- 잔여 8건(2022/2023): 파일 saved-vpos 작은데도 PDF 갭 보존 = #1246 비대칭(마진이 vpos 아닌
  render 시점 가산).

## 4. v3 핵심 부정 결과 — 마진 필드 구분 불가

| 샘플 | raw_unknown(적용값) | note_spacing | PDF |
|------|------|------|------|
| sep2020 | 1984(7mm) | 5669(20mm) | compact(미적용) |
| 미주사이20 | 5669(20mm) | 576(2mm) | gap(적용) |

- raw_unknown(둘 다 양수)·note_spacing(역전) 어느 것도 compact-vs-gap 를 가르지 못함. sep2020 의
  1984 가 default 인지 misparse 인지, pi=1128↔1129 가 동일 note 인지도 parsed 정보로 불확정.
- **구분 신호가 parsed FootnoteShape·line_seg·saved-vpos·구조 어디에도 없음** = #1246 입력 모호성.

## 5. 교훈 / 권고

- **교훈**: render-vs-typeset 발산에서 **어느 쪽이 PDF 정답인지는 문서마다 다름**(sep2020=render 과대,
  8건=render 정답). 단일 규칙·단일 신호로 통일 불가(Stage5·v2·v3·#1246·#1248·#1258 일관 확인).
- per-item 커서 보정은 반드시 `y_offset=new_y` advance **뒤에** add(대체 금지) — v2 구현 함정 교훈.
- **남은 경로(별도 타스크, 고위험)**: 파서에서 raw_unknown 바이트 의미 변형별 검증 + note-boundary
  그룹핑 정확화. payoff 불확실.
- **종결**: #1377 = Stage5 상태(기록 인프라 유지·clamp 보류, green) 유지. fix 방향 입증 + 구분 신호
  부재를 documented 한계로 확정.
