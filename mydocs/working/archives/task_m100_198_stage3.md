---
타스크: #198 rhwp-chrome 다운로드 인터셉터 부작용 수정
단계: Stage 3 — 작업지시자 수동 검증 + DEXT5 블랙리스트 추가
브랜치: local/task198
작성일: 2026-04-19
선행: Stage 2 완료
---

# Stage 3 단계별 완료 보고서

## 1. 진행 — 검증 + 발견 + 보강

### 1.1 1차 검증 결과 (Stage 2 dist 기준)

| 케이스 | 결과 |
|---|---|
| 일반 이미지 마지막 위치 기억 복원 | ✅ 통과 |
| 일반 이미지 첫 → 변경 위치 → 두번째 = 변경 위치 | ✅ 통과 |
| Chrome 완전 재시작 후 마지막 위치 유지 | ✅ 통과 |

→ **chrome-fd-001 핵심 (일반 파일 마지막 위치 기억) 100% 회복**.

### 1.2 추가 발견 — DEXT5 POST 다운로드 빈 뷰어 탭

작업지시자 추가 검증: biz.hira.or.kr (DEXT5 사용 사이트) 에서 .hwpx 다운로드 시:
- 다운로드된 파일은 정상 (3.5MB)
- 그러나 **rhwp 뷰어 탭이 열림** + `Invalid CFB file (76 bytes too small)` 에러

원인:
- DEXT5 는 POST `/com/dext5handler.ndo` 로 다운로드 응답
- 우리 인터셉터의 `handleHwpDownload` 가 `openViewer({url: item.url, ...})` 호출
- 뷰어 탭이 url 을 GET 으로 재요청 → POST 전용 핸들러는 빈 응답 → CFB 파싱 실패

작업지시자 통찰:
> "처리 못 할 다운로드면 탭도 안 열려야 하잖아요?"

본 fix 정신 (chrome-fd-001) 과 일관 — 부작용 주지 말 것.

### 1.3 블랙리스트 정책 결정 (작업지시자)

화이트리스트 vs 블랙리스트 검토 후 작업지시자 결정:
> "아직은 초기단계라 어떤 패턴이 들어올지 모릅니다. 사용자쪽에도 원인을 리포팅해서 다운로드 해서 열어 사용하라고 하면 이해하는 수준입니다."

→ **블랙리스트 운영**, 사용자 보고로 새 패턴 추가하는 방식.

## 2. 산출물 (Stage 3a 보강)

### 2.1 코드 변경

[`rhwp-chrome/sw/download-interceptor.js`](rhwp-chrome/sw/download-interceptor.js):
- `NON_REFETCHABLE_PATTERNS` 배열 추가 (현재 `dext5handler.{ext}` 1패턴)
- `shouldInterceptDownload` 진입 직후 url/referrer 가드 추가

```javascript
const NON_REFETCHABLE_PATTERNS = [
  /\/dext5handler\.[a-z0-9]+/i,  // DEXT5 (예: dext5handler.ndo, .jsp, .do)
];

export function shouldInterceptDownload(item) {
  if (!item) return false;
  const url = item.url || '';
  const referrer = item.referrer || '';
  if (NON_REFETCHABLE_PATTERNS.some(re => re.test(url) || re.test(referrer))) {
    return false;
  }
  // ... 기존 hwp 감지
}
```

### 2.2 단위 테스트 (+3, 총 23)

```
DEXT5 핸들러 url 차단 (filename 이 hwpx 여도) ... ok
DEXT5 핸들러 referrer 차단 ... ok
DEXT5 변종 확장자 (.jsp/.do) 도 차단 ... ok
```

### 2.3 dist 재빌드

```bash
rm -rf dist && npm run build
```

dist 의 `download-interceptor.js` 에 블랙리스트 패턴 반영 확인 (3건 매치).

## 3. 2차 검증 결과 (Stage 3a dist 기준)

| 케이스 | 결과 |
|---|---|
| **DEXT5 (biz.hira.or.kr) 빈 뷰어 탭 안 열림** | ✅ 통과 |
| Chrome 기본 다운로드 정상 진행 | ✅ |
| 일반 파일 마지막 위치 기억 (1차 케이스 회귀 없음) | ✅ |

작업지시자 보고:
> "기대한 동작이 확인되었습니다. 더이상 우리 탭이 열리지 않습니다."

## 4. 정체성 셀프 체크

- [x] 최소 침습 — `download-interceptor.js` + 테스트 파일 1건 + build.mjs 의 sw 복사 필터
- [x] 분기 명확화 — `if HWP { suggest } else { skip }` + DEXT5 가드
- [x] HWP 감지 다중화 + DEXT5 차단 — 화이트/블랙 두 신호 결합
- [x] 회귀 우선 — `handleHwpDownload` 변경 0
- [x] 블랙리스트 운영 정책 — 사용자 보고 기반 패턴 추가

## 5. 한계 및 후속 (작업지시자 공유)

- 블랙리스트는 알려진 패턴만 차단. 새 다운로드 핸들러 발견 시 사용자 보고 → 패턴 추가
- DEXT5 류 사이트의 hwp 자체 뷰어 처리는 **본 타스크 범위 외** (별도 이슈 가능)
- 사용자 안내: 뷰어가 열리지 않으면 다운로드된 파일을 직접 한컴/rhwp 로 열기

## 6. 승인 요청

본 단계 완료 보고서 승인 후:
- 최종 결과 보고서 작성 (`mydocs/report/task_m100_198_report.md`)
- 오늘할일 갱신
- 커밋 → merge → push → 이슈 #198 close
