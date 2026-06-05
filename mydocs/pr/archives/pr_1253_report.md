# PR 준비 보고서 — 미주 모양 UI와 HWPX 구분선 굵기 정합

- **작성일**: 2026-06-03
- **PR**: 준비 중
- **제목 후보**: `task 1253: 미주 모양 UI와 HWPX 구분선 굵기 정합`
- **브랜치**: `local/task_m100_1253`
- **원격 브랜치 후보**: `task_m100_1253`
- **연결 이슈**: #1253

## 1. 처리 요약

한컴오피스 2024의 `주석 모양 > 미주 모양` UI와 rhwp-studio UI가 다르게 보여 같은 설정인지 판단하기 어려운 문제를 보정했다.

이번 PR의 핵심 변경:

- 미주 구분선 종류/굵기/색 선택 UI를 한컴 대화상자처럼 시각 미리보기 중심으로 보강
- 구분선 길이 항목에 `사용자` 선택 콤보 추가
- HWPX `<hp:noteLine width>` 파서가 일반 선 굵기 코드표를 사용하도록 보정
- `0.12mm → 1`, `0.7mm → 9` 선 굵기 매핑 회귀 검증 추가

내부 미주 간격 저장 계약은 유지했다.

- Studio JSON `noteSpacing` → 내부 `raw_unknown` → 한컴 UI `미주 사이`
- Studio JSON `separatorMarginBottom` → 내부 `note_spacing` → 한컴 UI `구분선 아래`

## 2. 단계별 내용

Stage0:

- #1253 이슈를 등록했다.
- 한컴 UI와 rhwp-studio UI/렌더 차이를 분리했다.
- 기존 HWP5/HWPX/Studio JSON 매핑을 재확인했다.

Stage1:

- `rhwp-studio/src/ui/endnote-shape-dialog.ts`의 구분선 종류/굵기/색 UI를 한컴 기준에 맞게 보강했다.
- 구분선 굵기 옵션을 저장 코드표 기준으로 정리했다.

Stage2:

- 구분선 길이 `사용자` 콤보를 추가했다.
- HWPX `noteLine width`를 공통 `parse_hwpx_line_width()`로 파싱하도록 변경했다.
- Stage별 작업 기록과 최종 보고서를 작성했다.

Stage3:

- PR 준비 이후 작업지시자가 `3-09월_교육_통합_2022.hwp` 9쪽이 한컴오피스와 여전히 다르게 렌더링된다고 보고했다.
- rhwp 기준 9쪽 좌표를 덤프했다.
  - 정답표 하단: `y≈325.17px`
  - 녹색 미주 구분선: `y≈351.09px`
  - `문1)` 시작: `y≈370.97px`
- 현재 PR 범위는 UI/API/HWPX 파서 정합이며, 실제 미주 `LINE_SEG`/VPOS 소비 방식의 한컴 정합은 후속 Stage3 문제로 남긴다.

## 3. 자동 검증

| 항목 | 결과 | 비고 |
|---|---|---|
| `npm run build --prefix rhwp-studio` | 통과 | Vite build 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 43 passed |
| `wasm-pack build --target web --out-dir pkg` | 통과 | `pkg` 산출물 생성 |
| `cargo test --tests` | 통과 | 전체 integration 통과 |

`wasm-pack` 검증 중 prebuilt `wasm-bindgen`이 없는 플랫폼이라 cargo install fallback 경고가 있었지만 최종 산출물 생성은 성공했다.

## 4. PR 본문 이슈 표기

```text
관련 이슈: #1253
```

Stage3에서 아직 렌더 차이가 남아 있으므로 자동 종료 키워드는 쓰지 않는다.

## 5. 남은 사항

- 작업지시자가 요청한 대로 Chrome extension 세션 검증은 제외했다.
- rhwp-studio UI 최종 시각 정합은 작업지시자 판단 기준으로 확인한다.
- `3-09월_교육_통합_2022.hwp` 9쪽 미주 조판 차이는 후속 Stage3에서 계속 추적한다.
