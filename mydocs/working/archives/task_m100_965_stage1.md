# Task #965 Stage 1 — 현재 WMF baseline 측정

## 1. WMF 보유 sample 식별

`data:image/svg` (WMF → SVG converter 결과 embed) 보유 sample/page 스캔:

### hwp3-sample16 (사용자 보고 핵심 sample)
- page 18 (HWP3 native page 16): 1 WMF (주전산센터 목표시스템 구성안 다이어그램)
- SVG size: 4.2MB
- 결함: 박스 내부 한글 텍스트 ("PE6450", "기록서버", "Windows 서버군", "Unix 서버군" 등) 가 박스 하단 라인에 걸침 / 박스 외부 표시

### hwp3-sample14 (PR #918 검증 sample)
| Page | WMF count | SVG size |
|------|-----------|----------|
| 0 | 1 | 249 KB |
| 1 | 2 | 250 KB |
| 2 | 2 | 207 KB |
| 3 | 2 | 259 KB |
| 4 | 1 | 203 KB |
| 5 | 2 | 215 KB |
| 6 | 1 | 235 KB |
| 8 | 2 | 178 KB |

### hwp3-sample4
- page 1: 1 WMF (352 KB)
- page 10: 1 WMF (587 KB)

## 2. Baseline PNG 저장

`/tmp/task965/baseline/png/` 에 다음 PNG 저장 (Stage 4 비교 기준):
- `sample16_p18.png` — 핵심 결함 page
- `sample14_p0.png` ~ `sample14_p8.png` — 회귀 검증
- `sample4_p1.png` — 회귀 검증

## 3. 현재 SVG 구조

WMF 가 SVG 외부 (rhwp SVG) 의 `<image href="data:image/svg+xml;base64,...">` 로 embed:
- 인라인 nested `<svg>` 가 아닌 base64-encoded SVG document

Stage 33-A 의 nested `<svg>` inline embed 는 본 task 의 scope 외 (별도 issue).

## 4. 결함 패턴 (sample16 page 18)

WMF 다이어그램 내부 텍스트 상태:
- "Windows 서버군" - 박스 내부 정상 (TOP align?)
- "PE6450" 박스 우측의 라벨 - 박스 하단 라인에 걸침 → root cause: VTA_BASELINE 인 mode 가 VTA_TOP 으로 잘못 매핑 → text 가 +ascent 만큼 아래로 shift

## 5. 후속 (Stage 2)

Stage 2 — 구현 계획 V2 작성. PR #918 commit `f53235c6` 의 핵심 변경 cherry-pick:
1. `set_text_align` (~2191): vertical bits 분기 정정
2. `ext_text_out` (~813-826): baseline y shift 정합
3. 두번째 `ext_text_out` (~1541-1548): 동일 보정

renderer/svg.rs 변경 (nested SVG embed) 과 woff2 제거는 **제외**.
