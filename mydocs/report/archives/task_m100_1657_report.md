# Task M100 #1657 최종 보고서 — 원격 로드 시그니처 게이트 HWP3 차단 수정

- 이슈: #1657
- 브랜치: `local/task1657` (base: `local/devel`)
- 작성일: 2026-06-29

## 1. 증상

studio/확장에서 URL 파라미터로 HWP3 문서 로드 시 코어가 파싱 가능한데도 게이트가 차단:

```
[main] 파일 로드 실패: Error: 실제 HWP/HWPX 파일이 아닙니다. 파일 시그니처를 확인할 수 없습니다.
```

재현(조달청 공공 실문서): `https://www.pps.go.kr/common/fileDown.do?key=200001100001&sn=1`
- 응답 본문 첫 23바이트 = `HWP Document File V3.00` (HWP3)
- 서버는 HTTP 404 + Content-Type: text/html 을 보내지만 본문은 정상 HWP3.

## 2. 근본 원인

`rhwp-studio/src/main.ts` 의 `detectDocumentByteKind()` 가 HWP5(CFB `D0CF11E0`) + HWPX(ZIP `PK`)
만 검사하고 **HWP3 매직을 누락** → `unknown` → `assertRemoteDocumentBytes()` 거부. 그러나 코어는
HWP3 정식 지원(`src/parser/hwp3/`). 실제 PPS 파일을 코어로 검증: `info` 버전 3.0.0.0/14페이지,
`export-svg` 14페이지 정상 렌더 → 게이트가 코어 지원 포맷을 막는 불일치.

## 3. 수정

- 시그니처 로직을 `src/core/document-signature.ts` 로 추출(studio 관례: 순수 로직 모듈 +
  `tests/*.test.ts`). main.ts 는 `assertRemoteDocumentBytes` 만 import.
- `HWP3_SIGNATURE`("HWP Document File V3.00", 23바이트) 추가 → `detectDocumentByteKind` 가 `'hwp'`.
- **매직 우선 판정**이라 서버의 404/text/html 헤더보다 본문 매직이 먼저 매칭 → PPS 케이스 통과.
  실제 HTML 오류/미리보기 페이지 거부, unknown 거부는 그대로 유지(보안 게이트 의도 보존).

## 4. 검증

| 항목 | 결과 |
|---|---|
| 실제 PPS HWP3 (ct=text/html) | `hwp` 판정 → 통과 |
| HTML 오류 페이지 / unknown 바이트 | `html`/`unknown` → 여전히 거부 |
| HWP5(CFB) / HWPX(ZIP) | 회귀 없음 |
| 신규 `tests/document-signature.test.ts` | 6/6 pass |
| studio 전체 `npm test` | **153/153 pass** (기존 147 + 신규 6) |
| `tsc --noEmit` / `npm run build` | 에러 0 / ✓ built |
| 코어 PPS 파싱 (info/export-svg) | 14페이지 정상(게이트만 문제였음 확증) |

## 5. 영향 범위

- 확장(chrome/firefox)은 별도 시그니처 검사 없이 studio main.ts 게이트를 공유 → 본 수정으로
  확장 경로도 함께 해소. 배포 시 dist 재빌드 시점에 반영.
- 서버의 404/text/html 오발신은 외부 요인이라 수정 불가하나, 매직 우선 판정으로 우회.

## 6. 산출물

- 코드: `rhwp-studio/src/core/document-signature.ts`(신규), `src/main.ts`(블록 추출+import)
- 테스트: `rhwp-studio/tests/document-signature.test.ts`(신규 6)
- 본 보고서: `mydocs/report/task_m100_1657_report.md`
