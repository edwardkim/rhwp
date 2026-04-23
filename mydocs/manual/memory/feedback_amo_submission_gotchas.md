---
name: AMO (Firefox Add-ons) 제출 시 마주치는 4대 함정
description: rhwp-firefox v0.2.1 제출 시 연속으로 마주친 AMO 에러와 해결책. 다음 확장 등록 또는 다음 버전 재제출 시 동일 실수 반복 방지를 위한 체크리스트.
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
# AMO 제출 4대 함정 (2026-04-23 rhwp-firefox v0.2.1 경험)

## 규칙

Firefox AMO (addons.mozilla.org) 에 확장을 신규 제출하거나 새 버전을 올릴 때, 아래 4가지 에러 중 하나 이상을 마주칠 가능성이 높다. 모두 **사전에 manifest 와 자료 준비로 회피 가능**.

**Why**: rhwp-firefox v0.2.1 제출 시 "data_collection_permissions 필수 → Android 버전 숫자 거부 → 중복 gecko id → 소스 코드 제출 요구" 4개 에러가 연속 발생했다. 각 에러 원인이 AMO 정책 문서에 분산 기록되어 있어 사전 파악이 어려웠다. 다음 제출에서 이 문서를 먼저 확인하면 같은 우회로를 반복하지 않아도 된다.

**How to apply**: Firefox 확장을 AMO 에 제출하기 **전** 이 체크리스트를 먼저 돌린다. manifest 수정 + 자료 준비 → 한 번에 통과.

## 체크리스트

### 1. `data_collection_permissions` — AMO 필수 키

```json
"browser_specific_settings": {
  "gecko": {
    "data_collection_permissions": {
      "required": ["none"]
    }
  }
}
```

- AMO 에러 메시지: *"The property is required for all new Firefox extensions"*
- web-ext lint 는 `KEY_FIREFOX_UNSUPPORTED_BY_MIN_VERSION` 경고를 내지만 **무시**. AMO 서버 요구 우선.
- 실제 데이터 수집 없으면 `"required": ["none"]` 이 정확한 선언.
- 참고: https://mzl.la/firefox-builtin-data-consent

### 2. `gecko_android` 옵트아웃 방식

**placeholder 버전 숫자 사용 금지**:

```json
// 거부됨
"gecko_android": {
  "strict_min_version": "999.0"
}
```

AMO 에러: *"Unknown strict_min_version 999.0 for Firefox Android"* — AMO 는 실존 Gecko 버전 (addons.mozilla.org/api/v5/applications/firefox/) 만 인정.

**공식 옵트아웃**: `gecko_android` 키 **완전 생략**.

> MDN: "To support Firefox for Android without specifying a version range, the `gecko_android` sub-key must be an empty object. Otherwise, the extension is only made available on desktop Firefox."

즉 `gecko_android` 키가 없으면 자동으로 desktop 전용.

Android 호환 여부 판단 기준 (현 상태):
- `browser.downloads.onCreated/onChanged` — **v79 에서 제거됨** (MDN BCD)
- `browser.contextMenus` — **미지원** (version_added: false)
- 이 둘을 쓰면 Android 옵트아웃 필수.

### 3. gecko id 충돌

**"중복된 부가 기능 ID가 발견됨"** 에러:
- 같은 id 를 타인/타계정이 이미 AMO 에 등록함
- 또는 본인 계정에 이전 draft 가 남아 있음

확인 방법:
```
curl https://addons.mozilla.org/api/v5/addons/addon/{id}/
# 404 → 공개 등록 없음 (하지만 draft/unlisted 가능성 있음)
# 200 → 타인 등록됨
```

**해결**: **id 에 플랫폼명 포함** 권장 — 충돌 회피 + 추후 rhwp-chrome / rhwp-safari 가 개별 id 를 가질 때 일관성.

예: `rhwp@...` → **`rhwp-firefox@...`**

**주의**: gecko id 는 AMO 첫 등록 후 **변경 불가**. 첫 제출 전이 유일한 변경 기회.

### 4. 소스 코드 제출 — GitHub URL 불가

**GitHub URL 만으로 대체 불가능**. AMO 정책:
- 소스 zip 파일 **직접 업로드** 필수 (200MB 한도)
- GitHub 은 삭제·변경 가능성 때문에 신뢰할 수 없다고 판단
- 심사자가 diff 도구로 로컬 파일끼리 비교

다음 조건 중 하나라도 해당하면 소스 제출 요구:
- Minified / 번들된 JS (Vite · webpack · rollup 등)
- TypeScript → JavaScript 트랜스파일
- WASM 바이너리 (Rust · Emscripten 등)
- Docker / wasm-pack / wasm-bindgen 같은 외부 빌드 도구

즉 **rhwp 류 확장은 거의 확실히 소스 제출 대상**.

**제출 zip 최적화**:
```bash
# 1. 클린: .gitignore 기반 (git archive 사용)
git archive --format=zip --prefix=rhwp-source/ HEAD \
  ':(exclude)samples' \
  -o output/amo/rhwp-source-{sha}-no-samples.zip

# 2. samples/ 제외: HWP 테스트 파일은 빌드에 불필요 → 60MB 절약
```

전체 저장소 91MB → samples 제외 37MB 로 감소.

## 제출 체크리스트 순서

AMO 에 zip 업로드 전 사전 점검:

1. [ ] `manifest.json` 의 `browser_specific_settings.gecko` 에 `data_collection_permissions` 키 포함
2. [ ] Android 미지원 확장은 `gecko_android` 키 생략 (추가하지 말 것)
3. [ ] gecko id 에 플랫폼 suffix 포함 (`@...` 앞부분 예: `rhwp-firefox`)
4. [ ] AMO 공개 API 로 id 중복 확인: `curl https://addons.mozilla.org/api/v5/addons/addon/{id}/`
5. [ ] 소스 zip 사전 준비 (`git archive` 로 samples 등 제외)
6. [ ] `npx web-ext lint --source-dir=dist` errors 0 확인 (warnings 는 AMO 통과 가능)
7. [ ] Reviewer Notes 영문으로 준비 — 권한 정당화 + WASM 안전성 + 테스트 방법

## 매뉴얼 참조

- `mydocs/manual/chrome_edge_extension_build_deploy.md` §4.4 — Firefox 제출 절차
- `mydocs/release/amo_submission_v0.2.1.md` — 실제 제출 메타 (유형 템플릿)

## 관련 이력

2026-04-23 rhwp-firefox v0.2.1 최초 AMO 제출 — 위 4개 에러 연속 대응 후 접수 완료. 본 메모리 자산.
