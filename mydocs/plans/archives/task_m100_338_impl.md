---
타스크: #338 Firefox AMO 워닝 해결
브랜치: local/task338
작성일: 2026-04-26
선행: mydocs/plans/task_m100_338.md (수행계획서, 승인됨)
---

# 구현계획서: Firefox AMO 워닝 해결

## 0. 작업지시자 결정 사항

| 질문 | 결정 |
|---|---|
| Q1. Firefox 최소 지원 버전 | **142.0**. Firefox Desktop 140+ 필드 지원뿐 아니라 Android 142 워닝까지 함께 해소 |
| Q2. reviewer note 위치 | **별도 보고 문서**: `mydocs/report/amo_reviewer_note_task338.md` |
| Q3. AMO 워닝 원문 | 현재 저장소에 `mydocs/feedback/amo-warning-01.md` 없음. 이슈 본문 워닝을 기준으로 1차 진행 |

## 1. 핵심 설계

### 1.1 manifest 버전 정합성

`rhwp-firefox/manifest.json`의 Gecko 설정을 다음처럼 조정한다.

```json
"browser_specific_settings": {
  "gecko": {
    "id": "rhwp@edwardkim.github.io",
    "strict_min_version": "142.0",
    "data_collection_permissions": {
      "required": ["none"]
    }
  }
}
```

근거:
- `data_collection_permissions`는 AMO 제출 필수 항목으로 유지한다.
- 이슈 본문상 Firefox Desktop은 140 이상, Firefox for Android는 142 이상이 필요하다.
- 하나의 `strict_min_version`으로 양쪽 워닝을 닫으려면 `142.0`이 가장 보수적이다.

### 1.2 워닝 분류 기준

AMO 정적 분석 워닝은 다음 기준으로 분류한다.

| 분류 | 처리 |
|---|---|
| 우리 소스 코드의 사용자 입력/외부 데이터 경로 | DOM API, `textContent`, 안전한 URL 처리로 수정 |
| 우리 소스 코드의 내부 초기화/컨테이너 비우기 | 위험도 기록 후 가능하면 안전 API로 치환 |
| Vite 번들 런타임 또는 의존성 코드 | reviewer note 대상 |
| wasm-bindgen 생성 코드 | reviewer note 대상, 생성물 수동 패치 금지 |
| 테스트 HTML 또는 dist 제외 파일 | 배포 포함 여부 확인 후 제외 근거 기록 |

### 1.3 reviewer note 문서 구조

`mydocs/report/amo_reviewer_note_task338.md`에 다음 항목을 남긴다.

- 제출 대상 버전과 manifest 변경 요약
- AMO 워닝별 위치, 원인, 사용자 데이터 영향 여부
- 제거 완료된 워닝 목록
- 남는 워닝에 대한 설명 문구
- 재현/검증 명령

## 2. 단계 분할 (4 Stage)

### Stage 1 — 워닝 후보 재현 및 원본 매핑

**조사**:
- `rhwp-firefox/manifest.json`과 `rhwp-firefox/dist/manifest.json`의 Gecko 설정 확인
- `rhwp-firefox` 소스와 `dist`에서 다음 패턴 검색:
  - `strict_min_version`
  - `data_collection_permissions`
  - `new Function`
  - `Function(`
  - `innerHTML`
  - `document.write`
- `rhwp-firefox/build.mjs`의 dist 복사 범위를 확인하여 test HTML 포함 여부 검증
- `assets/viewer-*.js` 워닝이 `rhwp-studio` 원본인지 Vite/의존성 코드인지 sourcemap 또는 원본 검색으로 분류

**완료 보고서**:
- `mydocs/working/task_m100_338_stage1.md`

**완료 기준**:
- 수정 대상과 reviewer note 대상이 표로 분리됨
- `strict_min_version` 상향 대상이 명확히 확인됨

### Stage 2 — manifest 및 소스 수정

**수정**:
- `rhwp-firefox/manifest.json`: `strict_min_version`을 `142.0`으로 상향
- 우리 코드 기원 `innerHTML`/`document.write`/동적 `Function`이 확인되면 안전 API로 수정
- 테스트 전용 코드가 dist에 포함될 가능성이 있으면 build 제외 규칙 또는 복사 범위 확인 후 조정

**예상 파일**:
- 필수: `rhwp-firefox/manifest.json`
- 조건부: `rhwp-firefox/*.js`, `rhwp-firefox/sw/*.js`, `rhwp-studio/src/**`

**완료 보고서**:
- `mydocs/working/task_m100_338_stage2.md`

**완료 기준**:
- manifest 충돌 해소
- 우리 코드에서 제거 가능한 보안 워닝 제거 또는 안전성 근거 확보

### Stage 3 — Firefox 확장 빌드 및 dist 갱신

**작업**:
- `cd rhwp-firefox && npm run build`
- `rhwp-firefox/dist/manifest.json`에 `strict_min_version: "142.0"` 반영 확인
- dist 산출물에서 워닝 후보 재검색
- 빌드 결과의 `wasm/rhwp.js`와 `assets/viewer-*.js` 잔여 워닝 위치 기록

**완료 보고서**:
- `mydocs/working/task_m100_338_stage3.md`

**완료 기준**:
- 빌드 성공
- dist 산출물 갱신
- 잔여 워닝이 reviewer note 대상인지 확인 완료

### Stage 4 — reviewer note 및 최종 보고

**문서**:
- `mydocs/report/amo_reviewer_note_task338.md`
- `mydocs/working/task_m100_338_report.md`
- `mydocs/orders/20260426.md` 상태 갱신

**검증 요약**:
- manifest 버전 확인 결과
- 빌드 성공 여부
- 잔여 워닝과 reviewer note 매핑
- 소스 수정 범위와 회귀 위험

**완료 기준**:
- AMO 재제출용 reviewer note 초안 확보
- 최종 보고서와 오늘할일 갱신 완료

## 3. 파일 변경 요약

| Stage | 신규 파일 | 수정 파일 |
|---|---|---|
| 1 | `mydocs/working/task_m100_338_stage1.md` | 없음 |
| 2 | `mydocs/working/task_m100_338_stage2.md` | `rhwp-firefox/manifest.json`, 조건부 소스 파일 |
| 3 | `mydocs/working/task_m100_338_stage3.md` | `rhwp-firefox/dist/*` |
| 4 | `mydocs/report/amo_reviewer_note_task338.md`, `mydocs/working/task_m100_338_report.md` | `mydocs/orders/20260426.md` |

## 4. 검증 명령

```bash
rg -n '"strict_min_version"|"data_collection_permissions"' rhwp-firefox/manifest.json rhwp-firefox/dist/manifest.json
rg -n "new Function|Function\\(|innerHTML|document\\.write" rhwp-firefox rhwp-studio/src --glob '!node_modules'
cd rhwp-firefox && npm run build
rg -n '"strict_min_version"|"data_collection_permissions"' rhwp-firefox/dist/manifest.json
rg -n "new Function|Function\\(|innerHTML|document\\.write" rhwp-firefox/dist
```

## 5. 위험 요소

| 위험 | 단계 | 완화 |
|---|---|---|
| `142.0` 상향으로 Firefox 140~141 Desktop 사용자 제외 | Stage 2 | Android 워닝까지 닫기 위한 결정으로 reviewer note와 보고서에 명시 |
| `Function(` 검색이 일반 함수명/문서 텍스트까지 과검출 | Stage 1/3 | 위치별 수동 분류, 실제 AMO 워닝 라인 중심으로 기록 |
| Vite 번들 내부 `document.write`를 무리하게 제거하려다 빌드 복잡도 증가 | Stage 2 | 우리 코드 기원이 아니면 reviewer note 처리 |
| dist 빌드가 기존 WASM 산출물(`pkg/`) 부재로 실패 | Stage 3 | 실패 시 원인 기록 후 필요한 선행 WASM 빌드 여부를 작업지시자에게 보고 |
| `rhwp-studio` 수정이 Firefox 외 산출물에 영향 | Stage 2 | Firefox 워닝 제거에 필요한 경우만 최소 수정 |

## 6. 일정

- Stage 1: 0.2일
- Stage 2: 0.4일
- Stage 3: 0.2일
- Stage 4: 0.2일
- **총: 1일 내외**

## 7. 정체성 셀프 체크

- [x] 이슈 → 브랜치 → 오늘할일 → 수행계획서 → 구현계획서 순서 준수
- [x] 최소 3단계 이상, 최대 6단계 이하 구현 단계 구성
- [x] Firefox 확장 범위에 집중
- [x] wasm-bindgen 생성물 직접 패치 금지
- [x] reviewer note 대상과 실제 수정 대상을 분리

## 8. 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
