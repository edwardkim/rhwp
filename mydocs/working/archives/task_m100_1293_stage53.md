# Task 1293 Stage 53: full sweep visual drift 후보 직접 검토

## 목적

Stage52 최신 full sweep all에서 핵심 게이트는 모두 0이었다.

- SVG/PDF/render tree page count mismatch: 0
- renderer `LAYOUT_OVERFLOW`: 0
- frame overflow 후보: 0
- question title/text overlap 후보: 0
- equation/text overlap 후보: 0
- line order overlap 후보: 0
- endnote separator gap drift 후보: 0

다만 `visual_metrics`의 coarse drift 후보는 남아 있다. 이 단계에서는 contact sheet와
compare/annotated PNG를 직접 확인해, 남은 후보가 실제 미주 기능 실패인지 또는 전역 텍스트
렌더링/마커 위치 차이에 따른 비차단 후보인지 분류한다.

## 검토 대상

자동 후보가 많은 target을 우선 확인한다.

| target | contact sheet | flagged | red | line | large | bottom |
|---|---|---:|---:|---:|---:|---:|
| `2024-11-practice-no-separator-above20-between20-below20` | `output/task1293_stage51_full_sweep_final/2024-11-practice-no-separator-above20-between20-below20/contact_sheet.png` | 22 | 11 | 7 | 14 | 9 |
| `2024-11-practice-above0-between0-below0` | `output/task1293_stage51_full_sweep_final/2024-11-practice-above0-between0-below0/contact_sheet.png` | 20 | 12 | 10 | 13 | 10 |
| `2024-11-practice-shape987` | `output/task1293_stage51_full_sweep_final/2024-11-practice-shape987/contact_sheet.png` | 20 | 10 | 10 | 13 | 8 |
| `2024-09-between20` | `output/task1293_stage51_full_sweep_final/2024-09-between20/contact_sheet.png` | 18 | 9 | 9 | 14 | 5 |
| `2022-10` | `output/task1293_stage51_full_sweep_final/2022-10/contact_sheet.png` | 14 | 6 | 7 | 11 | 5 |

## 판정 기준

- 미주 구분선 위/아래/미주 사이 간격이 PDF와 구조적으로 다르면 미완료로 본다.
- 미주 제목과 본문, 수식과 텍스트, 하단 frame overflow가 보이면 미완료로 본다.
- 페이지 수가 맞고 overflow/겹침/구분선 gap 후보가 없으며, contact sheet에서 보이는 차이가
  문항 빨간 marker, 일반 텍스트 폭, 그림/본문 ink 영역의 coarse metric 차이면 별도 후속 후보로
  남기되 미주 기능 완료 차단 사유로 보지 않는다.

## 직접 확인 기록

### `2024-11-practice-no-separator-above20-between20-below20`

- `compare_009.png`: 9쪽은 대체로 PDF와 rhwp의 문항 흐름이 같다. 큰 drift 후보는 보기 박스/그림
  영역의 민감한 픽셀 매칭에서 비롯된 후보가 섞여 있다.
- `compare_021.png`: 21쪽은 오탐이 아니다. rhwp는 왼쪽 단에 문27 도형 풀이와 문28을 배치하지만,
  PDF는 같은 위치에 문28 continuation과 문29 이후 흐름을 배치한다. page count만 23쪽으로 맞고
  내부 pagination이 뒤로 밀린 구조적 차이다.
- `compare_022.png`, `compare_023.png`도 같은 tail drift의 결과로 보인다. `flagged_pages.json`에서
  23쪽은 rhwp red marker가 7개, PDF red marker가 0개로 잡힌다.

## 중간 판단

Stage52의 핵심 게이트 0은 충분하지 않다. `visual_metrics`의 `red_marker_drift`,
`line_band_drift`, `large_ink_region_drift`, `content_bottom_drift` 중 일부는 실제 pagination
차이를 드러낸다.

공식 도움말 기준:

- `구분선 넣기`: 본문과 미주 내용 사이에 구분선을 긋는 옵션이다.
- `구분선 위`: 본문과 미주 구분선 사이의 간격이다.
- `구분선 아래`: 미주 구분선과 미주 내용 사이의 간격이다.
- `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격이다.

구분선이 꺼진 샘플에서 rhwp는 후반부로 갈수록 PDF보다 미주 흐름이 뒤로 밀린다.

## 실험 기록

아래 실험은 코드에 임시 적용 후 focused 테스트와 targeted sweep으로 확인했고, 효과가 없거나 회귀를
만들어 원복했다.

1. `endnote_separator_height_px`에서 보이는 구분선이 없으면 `0.0`을 반환하게 했다.
   - `issue_1139_inline_picture_duplicate`는 통과했다.
   - `2024-11-practice-no-separator-above20-between20-below20` page count는 23/23으로 유지됐지만,
     `compare_021.png` 구조 차이는 그대로였다.
   - 최초 separator block만 원인은 아니다.
2. no-separator 경계에서 `미주 사이 20mm` 전체가 아니라 기본 7mm 초과분만 pagination에 더하게 했다.
   - SVG page count가 22쪽으로 줄어 PDF 23쪽과 불일치했다.
   - no-separator에서도 전체 between-notes 예약은 page count 유지에 필요하다.
3. `large_separator_block`의 첫 단 tail 허용을 보이는 구분선 조건 없이 허용했다.
   - focused 테스트는 통과했지만 targeted sweep의 21쪽 구조 차이는 그대로였다.
   - 문제 지점은 문27/28 자체가 아니라 그 이전 문22~23 근처의 누적 column flow drift다.

## 다음 분석 후보

- `compare_020.png` 기준 PDF는 문23/문24를 왼쪽 단 하단에 남기지만, rhwp는 문23부터 오른쪽 단에
  배치한다. 즉 최초 큰 차이는 21쪽이 아니라 20쪽 문23 직전 단 경계다.
- trace상 문23/28 일부 paragraph에서 빈 단(`items=0`)인데도 `adv_fit=true`가 잡히는 케이스가 있다.
  큰 vpos span을 자체 높이로 과대평가하는지 확인해야 한다.
- 다음 단계에서는 문22 tail → 문23 title 경계의 `en_fit`, `total_advance_fit`, render-tree `pi`
  흐름을 page index와 함께 추적하고, 자동 sweep에 문항 시작 page/column drift를 core 후보로
  승격하는 방법도 함께 검토한다.
