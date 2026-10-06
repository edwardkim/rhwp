# Task M100 #1685 최종 보고서 — HWPX 'HWP 변환 저장' 잔존 안내 제거

- 이슈: #1685
- 브랜치: `local/task1685` (base: `local/devel`)
- 작성일: 2026-06-30

## 1. 증상

rhwp-studio 에서 HWPX 파일을 열면 "HWPX 문서는 저장 시 HWP 형식으로 변환 저장됩니다. 원본
HWPX 를 덮어쓰지 않도록 .hwp 파일명으로 저장합니다." 토스트(+ 상태바 "HWPX 변환 저장 모드")가
표시됐다.

## 2. 근본 원인

`main.ts` 의 `notifyHwpxSaveModeIfNeeded()` 는 #888 시절(HWPX 직접 저장 불가) 안내다. 그러나
#1532/#1533(#196 베타 해제)으로 HWPX 직접 저장이 활성화되어, `saveCurrentDocument` 는 이미
HWPX 출처를 HWPX 로 저장한다(file.ts:125 "HWPX 출처는 HWPX 로 직접 저장"). #1533 이 저장
동작은 바꿨으나 이 안내 토스트/상태바를 정리하지 않은 누락이라, **안내가 실제 동작과 모순**됐다.

저장 로직 자체는 정상(HWPX→HWPX) — 잘못된 것은 안내뿐.

## 3. 수정 (안내 완전 제거)

| 위치 | 변경 |
|---|---|
| `main.ts:777` | `notifyHwpxSaveModeIfNeeded()` 호출 제거 |
| `main.ts:824-846` | 함수 전체 제거(토스트 + 상태바 "HWPX 변환 저장 모드" + #888 이슈 링크) |
| `types.ts:51` | 주석 정정 — "HWPX 출처는 HWP 변환 저장" → "저장 시 출처 포맷 유지(HWPX→HWPX, HWP→HWP), 다른 포맷 저장은 별도 메뉴(#1613)" |

HWPX 는 이제 HWPX 로 저장되므로 변환 안내 불필요. 사용자는 "HWP 형식으로 저장" 메뉴(#1613)로
명시적 변환을 여전히 선택할 수 있다.

## 4. 검증

| 항목 | 결과 |
|---|---|
| 잔존 참조 (`notifyHwpxSaveMode`/변환 안내 문구) | 0건 (완전 제거) |
| `showToast` import | 유지(다른 9곳 사용) |
| `tsc --noEmit` | 에러 0 |
| `npm run build` | ✓ built |
| studio 전체 `npm test` | **153/153 pass** (회귀 없음) |

## 5. 산출물

- 코드: `rhwp-studio/src/main.ts`(-27), `src/command/types.ts`(주석)
- 본 보고서: `mydocs/report/task_m100_1685_report.md`

> 주의: 사용자 환경 반영은 pkg 무관(순수 studio TS 변경)이나, studio dist/번들 재빌드 +
> 브라우저 하드 리프레시 후 토스트가 사라진다.
