# Task #1377 v2 Stage 3 (판정) + v3 마진-의미 조사 — 깨끗한 구분 신호 부재

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 3 판정 = 보류·revert. v3(마진 의미 판별) 조사 = **깨끗한 구분 신호 없음**.
- **코드 상태**: Stage5 (green 123/0). v3 조사는 코드 무수정.

## 1. v3 동기 (작업지시자 선택)

v2 잔여 8건(2022/2023)은 sep2020 과 구조 동일하나 PDF 정답이 반대(sep2020=compact, 8건=gap).
가설: **섹션 between-notes 마진 의미**(구분선 마진 vs 미주사이 마진)가 진짜 구분자.

## 2. PDF 픽셀 재확정 — sep2020 은 compact 정답

p22 PNG(96dpi) 픽셀 측정: PDF 문29 행 y=149(=yMin 111pt). rhwp SVG 문29 y=176 → render 가
**~27px(≈7mm) 낮음**. 27px ≈ between-notes 1984HU(7mm). → render 가 sep2020 에 1984 갭을
적용하나 PDF 는 미적용. **compact 가 PDF 정답** 확정(수동 좌표 측정의 모순은 픽셀로 해소).

## 3. FootnoteShape 필드 비교 — 구분 불가 (핵심 부정 결과)

`endnote_between_notes_margin(shape) = shape.raw_unknown` (typeset.rs:7273), typeset.rs:2381 이
미주 경계에서 직전 para 마지막 줄 line_spacing 에 stamp.

| 샘플 | raw_unknown(=적용값) | note_spacing | sep_bot | PDF 정답 |
|------|------|------|------|------|
| sep2020 (구분선아래20위20) | **1984(7mm)** | 5669(20mm) | 5669(20mm) | **compact(미적용)** |
| 미주사이20 | **5669(20mm)** | 576(2mm) | 0 | **gap(적용)** |

- 미주사이20: raw_unknown=20mm = PDF 갭 → 정합. 적용이 맞음.
- sep2020: raw_unknown=1984(7mm) 이나 PDF 는 미적용(compact).
- **어느 필드도 compact-vs-gap 를 신뢰성 있게 구분 못함**: raw_unknown 은 1984 vs 5669(둘 다 양수,
  적용/미적용 가르지 못함), note_spacing 은 sep2020=20mm·미주사이20=2mm 로 **역전**(PDF 와 반대).
- 즉 "구분선 마진 기원 vs 미주사이 기원"을 **파싱된 필드 값만으로 분리할 단서가 없다**. sep2020 의
  raw_unknown=1984 가 default 인지 misparse 인지, pi=1128→1129 가 진짜 note 경계인지(같은 note 면
  stamp 자체가 오적용)도 현 정보로 불확정.

## 4. narrowing 누적 결과

| 단계 | 게이트 | 회귀 | sep2020 |
|------|------|------|---------|
| Stage5 universal | 전 미주 para | 19 | fix |
| v2 eqonly+marker | 빈수식+ls≈marker | 14 | fix |
| v2 +vpos-delta | +파일 raw saved-vpos<marker/2 | 8 | fix |
| v3 마진 의미 | (필드 구분 신호 부재) | — | — |

## 5. 판정 / 권고

- **fix 방향(sep2020 compact)은 PDF 로 3중 입증**(픽셀·SVG·14테스트 정합). 그러나 sep2020 을
  gap-정답 문서(미주사이20·2022/2023 8건)와 **가를 신뢰 신호가 parsed FootnoteShape·line_seg·
  saved-vpos·구조 어디에도 없음**. 이는 `tech_trailing_model_no_ssot` 의 #1246 비대칭이 입력
  데이터(한컴 인코딩) 차원의 모호성임을 재확인.
- **남은 유일 경로(고위험·별도 타스크)**: 파서 차원에서 (a) `raw_unknown` 의 바이트 의미를 변형별로
  검증(separator 오귀속 여부), (b) 미주 note-boundary 그룹핑 정확화(pi=1128→1129 동일 note 여부).
  payoff 불확실·회귀 위험 큼.
- 현 권고: #1377 은 Stage5 상태(기록 인프라 유지·clamp 보류)로 종결하거나, 위 파서 조사를 신규
  타스크로 분리. v2/v3 는 fix 방향 입증 + 구분 신호 부재를 documented 한계로 확정.
