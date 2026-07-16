# PR #2248 검토 — proof-gated CanvasKit 리소스 replay + 실패 격리 (seo-rii, P34 계열)

- 검토일: 2026-07-15 / head `a1b3b2ce` / 72파일 +7,405/−165 / CI 13항목 전부 green
- 경위: 1차 검토(2026-07-15)에서 Render Diff 실패 원인 진단 3건 전달 →
  컨트리뷰터 전량 반영 + 근인 정련.

## 1차 진단 대비 반영 확인

| 진단 | 반영 |
|------|------|
| ①table-core `renderCountDelta: 2` (이중 렌더) | warm replay 를 page별 stale snapshot → **실시간 global counter 전후값**으로 — 타 페이지 prefetch 이력 오염 제거, delta=1 |
| ②font-native-bitmap 캐시 완전 비활동 | **근인 정련**: surface 편차가 아니라 `payloadResourceKey` 정밀도 불일치 (Rust 6자리 vs wire JSON 3자리 → strict 검증 상시 실패). 양측 3자리 통일 — imageCacheHitDelta=3, pixels=256 |
| ③base DIRTY (#2254/#2265) | rebase 완료 — lazy BinData 적응(사용 폰트만 ID별 1회 load), HML capability 보존 |

이전 실패 게이트(**Canvas visual diff**)가 원격 CI 에서 **pass** (4m26s),
readiness 5/5 · blocker 0.

## 구조 검토

- **#2225 계약 준수 확인**: MissingPicture 를 프로파일 분기 —
  Print/HighQuality = `missingPictureSuppressedPrintEquivalent`, screen =
  `missingPictureEditorVisual` (테스트 동반). 편집 표시/인쇄 억제 정책 정합.
- **#2265 적응**: `.load()` 소비 + 사용 폰트만 로딩 (미사용 lazy 유지) —
  지연 로딩 목적 훼손 없음.
- proof-gated 설계: strict 검증(BLAKE3 digest, bounded sidecar), fail-closed
  (미지원 스타일은 direct 완료 보고 금지) — P34 방향 정합.

## 게이트 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| default 전수 `--tests --no-fail-fast` | **3,217/0** |
| studio npm ci + tsc + test + build | 클린 / **283/0** / 성공 |
| @rhwp/editor | 15/0 |
| fmt / clippy(all-targets) | 통과 / 0 |
| 원격 CI | 13항목 green (readiness gate 포함) |

## 판단

**approve → merge 수용 권고.** BEHIND(경미 — rebase 후 devel 이 문서
커밋들로 전진) — merged tree 선검증 + admin merge.
