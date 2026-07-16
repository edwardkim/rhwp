# PR #2297 검토 — cross-backend 시각 회귀 manifest + profile 전달 (seo-rii, Refs #536)

- 검토일: 2026-07-16 / head 기준 21파일 +3,853/−272 / CI 12 green / BEHIND
- 요지: 회귀 fixture 를 versioned manifest(120 corpus/대표 21)로 통합,
  document digest·backend/profile/surface provenance 검증을 pixel diff 와
  분리, profile(screen/print/high-quality/fast-preview)을 전 백엔드에 전달.
  missingPicture 를 profile 주도 정책으로 일반화.

## 발견 — 계약 회귀 1건 (수정 요청)

**export-png 기본 실행에서 missingPicture placeholder 가 그려진다** (#2225
한컴 계약 위반: 인쇄 등가 출력 = 억제).

- 실증: 결재문서 심볼 placeholder 영역 잉크 **5,776** (devel = 0 억제).
- 원인: `main.rs:971` `export_png` 기본 profile = `Screen`
  (`shows_editor_visuals()==true`) — skia 의 무조건 억제가 profile 의존으로
  바뀌면서 기본값이 편집 시각을 노출.
- 대조: export-svg(기본 None→legacy 억제 경로)·export-pdf(None→legacy)는
  안전 — **export-png 만 기본이 Screen**.
- 게이트 공백: 기존 #2225 표적 테스트가 SVG 만 검사해 전수 통과에도
  회귀가 새어나감 (PR 무수정 통과의 이유).

**요청**: ①export-png 기본 profile 을 인쇄 등가(high-quality 또는 print)
로 — `--profile screen` 명시 시에만 편집 시각 포함 (PR 본문 의도
"missingPicture 는 편집 profile 에서만 표시"와 정합) ②#2225 회귀 가드에
PNG(skia) 축 추가 권고.

## 그 외 구조 검토 (건전)

- profile 정책 자체는 올바른 일반화 — `shows_editor_visuals()` 의
  Screen/FastPreview=표시, Print/HighQuality=억제 분류와 단위 테스트 적절.
- provenance(digest/identity) 검증을 visual diff 와 분리 실패 처리 —
  P34~P36 증명 축 정합. 기본 CI 규모 불증(대표 21) + full sweep 옵션 분리.
- studio 283/0(본문), CI 12 green. 로컬 게이트는 수정 반영 후 재실행 예정.

## 판단

**수정 요청(CHANGES_REQUESTED) 1건 후 merge 수용** — 회귀가 실증된 계약
위반이므로 반영 전 approve 불가. 나머지 설계는 방향 정합.
