# #198 최종 보고서 — rhwp-chrome 다운로드 인터셉터 부작용 수정

- **타스크**: [#198](https://github.com/edwardkim/issues/198) (`chrome-fd-001`)
- **마일스톤**: M100 (v1.0.0) — 이번 배포 포함
- **브랜치**: `local/task198`
- **기간**: 2026-04-19 (단일일)
- **상태**: **완료** ✅

## 1. 요약

사용자 보고 (`mydocs/feedback/chrome-fd-001.md`): 확장 활성 시 일반 파일 (이미지/웹페이지) 다운로드의 마지막 위치 기억 동작이 깨짐.

원인: [`download-interceptor.js`](rhwp-chrome/sw/download-interceptor.js) 의 `onDeterminingFilename` 리스너가 **모든 다운로드** 에 `suggest()` 를 호출 → Chrome 의 마지막 위치 기억 동작 무력화.

수정: `shouldInterceptDownload(item)` 순수 함수 추출 + HWP 인 경우에만 `suggest()` 호출. 추가로 작업지시자 검증 중 발견한 DEXT5 (POST 다운로드) 빈 뷰어 탭 부작용도 블랙리스트로 차단.

## 2. 진척 측정

### 2.1 chrome-fd-001 (1차 목표)

| 검증 | 결과 |
|---|---|
| 일반 이미지 첫 → 변경 위치 → 두번째 = 변경 위치 | ✅ |
| Chrome 완전 재시작 후 마지막 위치 유지 | ✅ |

→ 100% 회복.

### 2.2 DEXT5 빈 뷰어 탭 (2차 발견)

| 검증 | 결과 |
|---|---|
| biz.hira.or.kr (DEXT5) .hwpx 다운로드 → 빈 뷰어 탭 안 열림 | ✅ |
| Chrome 기본 다운로드 정상 진행 | ✅ |

작업지시자 보고: "기대한 동작이 확인되었습니다. 더이상 우리 탭이 열리지 않습니다."

## 3. 수행한 작업

### 3.1 단계별 산출물

| Stage | 산출물 | 보고서 |
|---|---|---|
| 1 | `shouldInterceptDownload` 추출 + 단위 테스트 20개 | [stage1](mydocs/working/task_m100_198_stage1.md) |
| 2 | dist 빌드 + `*.test.*` 자동 제외 필터 추가 | [stage2](mydocs/working/task_m100_198_stage2.md) |
| 3 | DEXT5 블랙리스트 추가 + 단위 테스트 +3 (총 23) + 작업지시자 검증 통과 | [stage3](mydocs/working/task_m100_198_stage3.md) |

### 3.2 코드 변경 요약

| 파일 | 변경 |
|---|---|
| [`rhwp-chrome/sw/download-interceptor.js`](rhwp-chrome/sw/download-interceptor.js) | `shouldInterceptDownload` 추출 + 분기 명확화 + DEXT5 블랙리스트 |
| [`rhwp-chrome/sw/download-interceptor.test.js`](rhwp-chrome/sw/download-interceptor.test.js) | 신규 — Node `--test` 23개 케이스 |
| [`rhwp-chrome/build.mjs`](rhwp-chrome/build.mjs) | sw 복사 시 `*.test.*` / `*.spec.*` 자동 제외 필터 |
| `rhwp-chrome/dist/` | 빌드 산출물 갱신 |

`viewer-launcher.js`, `manifest.json`, 다른 sw 모듈 — 변경 없음.

### 3.3 검증

| 항목 | 결과 |
|---|---|
| 단위 테스트 (`node --test rhwp-chrome/sw/download-interceptor.test.js`) | 23개 ✅ |
| dist 빌드 | 에러 0 ✅ |
| **작업지시자 수동 검증 (1차 + 2차)** | ✅ |

## 4. 핵심 설계 결정

### 4.1 분기 명확화

```javascript
// before: 모든 다운로드에 suggest 호출 → 일반 파일 마지막 위치 기억 깨짐
suggest({ filename: item.filename });

// after: HWP 인 경우에만 suggest → 일반 파일은 Chrome 기본 동작 유지
if (shouldInterceptDownload(item)) {
  handleHwpDownload(item);
  suggest({ filename: item.filename });
}
```

### 4.2 HWP 감지 다중화 + 재요청 불가 패턴 차단

```javascript
const HWP_EXTENSION_RE = /\.(hwp|hwpx)(\?|$)/i;
const HWP_MIME_HINTS = ['haansoft', 'x-hwp', 'hwp+zip'];
const NON_REFETCHABLE_PATTERNS = [
  /\/dext5handler\.[a-z0-9]+/i,  // DEXT5
];
```

- 화이트 신호: filename / url / finalUrl / mime
- 블랙 신호: dext5handler 류 핸들러 url

### 4.3 블랙리스트 운영 정책 (작업지시자 결정)

> "아직은 초기단계라 어떤 패턴이 들어올지 모릅니다. 사용자쪽에도 원인을 리포팅해서 다운로드 해서 열어 사용하라고 하면 이해하는 수준입니다."

새 다운로드 핸들러 발견 시 사용자 보고 → `NON_REFETCHABLE_PATTERNS` 에 추가.

### 4.4 build.mjs 추가 개선

테스트 파일이 dist 에 포함되는 부작용 발견 → `cpSync` 의 `filter` 옵션으로 자동 제외:

```javascript
const EXCLUDE_FROM_DIST = /\.(test|spec)\.[mc]?[jt]sx?$/i;
copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
});
```

## 5. 한계 및 후속

- 블랙리스트는 알려진 패턴만 차단. 새 다운로드 핸들러 (NamoUpload, GeoUpload 등) 발견 시 사용자 보고 기반 추가
- DEXT5 류 사이트의 hwp 자체 뷰어 처리는 본 타스크 범위 외 (별도 이슈 가능)
- 사용자 안내: 뷰어가 열리지 않으면 다운로드된 파일을 직접 한컴/rhwp 로 열기

## 6. 정체성 셀프 체크

- [x] 최소 침습 — 단일 sw 모듈 + 신규 테스트 + build.mjs 1줄
- [x] 분기 명확화 — `if HWP { suggest } else { skip }` + DEXT5 가드
- [x] 회귀 우선 — `handleHwpDownload` / `viewer-launcher` / manifest 변경 0
- [x] 트러블슈팅 사전 검색 (메모리 적용) — 0건 확인
- [x] 단위 테스트 + 작업지시자 수동 검증 두 게이트 모두 통과

## 7. 관련 자료

- 사용자 보고: `mydocs/feedback/chrome-fd-001.md`
- 단계별 보고서: `mydocs/working/task_m100_198_stage[1..3].md`
- 수행계획서: `mydocs/plans/task_m100_198.md`
- 구현계획서: `mydocs/plans/task_m100_198_impl.md`

## 8. 다음 작업

- PR #189 (rhwp-studio Ctrl+S 파일 핸들 유지) 머지
- #196 (rhwp-studio HWPX 저장 사용자 고지) 착수
