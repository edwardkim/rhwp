---
kind: canonical
status: active
canonical: mydocs/manual/font_incident_response.md
last_verified: 2026-08-15
---

# 폰트 감지·대체 사고 대응 절차

## 목적

문서에 기록된 글꼴과 실제 화면·SVG·PNG·PDF 글립 또는 조판이 다를 때, 이름 하나를 예외 처리하기
전에 원인을 같은 형식으로 분류하고 후속 이슈까지 인계하는 절차다. 대상은 다음 축을 포함한다.

- 브라우저 Local Font Access와 Canvas2D 설치 face 감지
- family/full name/PostScript/localized alias/style·weight 해소
- Canvas2D CSS face와 CanvasKit SFNT/Typeface 조달
- native/WASM 레이아웃 메트릭과 한컴/PDF missing-font 대체 결과

기술 계약은 [폰트 fallback 전략](../tech/font_fallback_strategy.md), 시각 판정은
[시각 검증 거버넌스](verification/visual_verification_governance.md)를 함께 따른다.

## 1. 작업 시작 게이트

1. 증상과 글꼴명으로 기존 이슈·PR·`mydocs/tech/investigations/`·`mydocs/troubleshootings/`를 찾는다.
2. 이슈가 있으면 댓글과 담당자를 확인하고 메인테이너를 담당자로 지정한다. 없으면 재현과 완료 기준을
   포함한 이슈를 먼저 만든다.
3. 연결된 open PR이 없는지 확인한다.
4. 깨끗한 기존 checkout에서 최신 `upstream/devel`을 fetch하고 이슈 전용 브랜치를 만든다.
5. 수행계획에 관련 이슈 disposition과 아래 7축 진단 매트릭스를 포함하고 메인테이너 승인을 받는다.

원격 push, PR 생성, GitHub 코멘트와 이슈 종료는 일반
[문서와 Git 워크플로우](codex/docs_and_git_workflow.md)의 승인 경계를 유지한다.

## 2. 필수 진단 매트릭스

| 축 | 필수 기록 |
| --- | --- |
| 환경 | OS, browser/version, Local Font Access 권한, renderer backend, snapshot version·generation |
| 원문 | document family, 언어 slot, style/weight, HWPX `substFont`, embedded font 여부 |
| 열거 | family/full/PostScript/style, localized alias, `FontData.blob()` 가능 여부 |
| probe | raw FontFace/local 또는 raw Canvas 결과, test vector, exact/fallback 폭, 모호성 |
| 선택 | 아래 상태 분류, 실제 CSS chain 또는 Typeface key, 선택 provenance |
| backend | Canvas2D effective font·폭, CanvasKit SFNT/typeface 여부, native/portable fallback |
| oracle | 한컴·PDF producer/version, 설치 글꼴과 hash, 쪽수·대표 glyph·첫 발산 지점 |

진단 결과는 이슈별 조사 문서에 보존한다. 반복 가능한 절차는 이 문서에, 장기 기술 계약은
`font_fallback_strategy.md`에 승격한다.

### 2.1 선택 상태 분류

| 상태 | 의미 |
| --- | --- |
| `exact-enumerated` | 열거된 face의 exact/localized alias와 style까지 일치 |
| `exact-probed` | 열거에는 없지만 문서의 정확한 이름을 raw Canvas/FontFace가 사용 가능 |
| `alias-only` | family/full/PostScript 중 일부 이름으로만 연결되며 동일 face 근거가 있음 |
| `style-collapsed` | family는 있으나 요청 style/weight가 다른 face로 뭉개짐 |
| `fallback-only` | exact face가 없고 문서 대체 또는 portable fallback만 사용 |
| `ambiguous` | probe 폭이 fallback과 구분되지 않거나 동일 face 근거가 부족 |

`exact-probed`는 Canvas2D 사용 가능 상태일 뿐 CanvasKit 사용 가능 상태가 아니다.

## 3. 감지 실패 유형별 확인 순서

### 3.1 Local Font Access

API 함수의 존재, 성공 응답, 큰 결과 count를 전체 설치 글꼴 목록의 완전성으로 간주하지 않는다.

1. 문서 후보를 열거 record의 family/full/PostScript/style/localized alias와 대조한다.
2. 해소되지 않은 후보만 raw Canvas presence probe로 확인한다.
3. raw probe는 제품이 패치한 `CanvasRenderingContext2D.font` setter를 통과하지 않아야 한다.
4. 양성·음성·모호 후보를 감지 세대에 저장하고 같은 세대에서 반복 측정하지 않는다.
5. 새 문서에 아직 확인하지 않은 후보가 있거나 사용자가 명시적으로 재감지할 때만 새 세대를 만든다.

### 3.2 이름과 style

- family가 있다는 이유로 Light/Medium/Bold 또는 variable font axis를 임의 선택하지 않는다.
- SFNT name table의 Unicode family, typographic family, subfamily, full name, PostScript name을 함께
  기록한다.
- 지역화 이름과 영문 이름은 같은 바이너리의 name table 또는 공인 배포 근거가 있을 때만 alias로
  연결한다.
- successor font는 일반 문자열 alias가 아니라 대상 legacy 이름에만 적용하는 curated mapping으로
  둔다. 메트릭이 다르면 하나의 layout profile로 합치지 않는다.

### 3.3 backend

- Canvas2D는 브라우저가 CSS 이름으로 face를 찾으면 원본 파일 bytes 없이 그릴 수 있다.
- CanvasKit은 실제 SFNT bytes와 style별 Typeface가 필요하다. `FontData.blob()` 실패 또는
  `exact-probed`를 local Typeface 성공으로 보고하지 않는다.
- native/WASM 레이아웃 메트릭, Canvas2D paint, CanvasKit glyph는 각각 따로 판정한다.
- 한 backend의 성공으로 다른 backend나 페이지네이션을 완료 처리하지 않는다.

## 4. RED fixture 최소 분할

폰트 감지 코드를 변경할 때 최소한 다음 상태를 자동 테스트한다.

1. Local Font Access 미지원
2. API 지원 + exact face 열거
3. API 지원 + family/alias만 열거하고 요청 style face 누락
4. API 지원 + 문서 후보 전체 누락, raw Canvas exact face 사용 가능
5. 메타데이터 열거 성공 + `FontData.blob()` 실패
6. raw candidate와 fallback 폭을 구분할 수 없는 모호 결과
7. 오래된 snapshot에 새 문서 후보가 없는 상태

브라우저 버전 변화로 자연 재현이 사라질 수 있으므로 3·4번은 mock 또는 CDP 하니스로 강제한다.
글자별 render/measure hot path에서 probe가 호출되지 않는지 호출 횟수도 고정한다.

## 5. 검증 자산 경계

- 공개 재배포가 허용된 폰트만 저장소 fixture·asset 후보로 검토한다.
- 상용·사내·재배포 불명 폰트는 저장소 밖 `ttfs/` 경로에서 사용하고 파일 hash, name table 요약,
  라이선스 출처만 기록한다.
- 폰트 바이너리, private HWP/HWPX, 비공개 corpus와 식별 가능한 파일 목록은 PR에 첨부하지 않는다.
- PDF oracle에는 입력 hash, 한컴 버전, PDF producer, 설치 글꼴 목록과 hash를 함께 기록한다.
- missing-font PDF와 exact-font PDF를 같은 정답지로 섞지 않는다.

## 6. 검증 게이트

### Studio 감지·Canvas2D만 변경

```bash
cd rhwp-studio
node tests/local-fonts.test.ts
node tests/document-font-status.test.ts
node tests/font-substitution.test.ts
npx tsc --noEmit
npm test
npm run build
```

실제 설치 face 검증은 [CDP 가이드](e2e-cdp.md)에 따라 호스트 Chrome에서 실행한다.

```bash
cd rhwp-studio
CHROME_CDP=http://localhost:19222 npm run e2e:issue-4741
```

보고에는 raw/patched effective font, 대표 문자열 폭과 delta, browser/version, snapshot provenance,
probe/cache 횟수를 남긴다. E2E가 localStorage를 임시 변경하면 기존 값을 백업하고 종료 전에 복원한다.

### CanvasKit·renderer·layout까지 변경

[로컬 사전 검증](pr_review/local_validation.md)의 변경 범위별 게이트를 추가한다. Rust/WASM 변경이
있으면 focused test, release-test, Native Skia, wasm-pack build와 필요한 시각 증적을 수행한다.
쪽수·geometry가 변하면 HWP/HWPX와 provenance가 고정된 PDF를 함께 대조하고 최종 시각 판정은
메인테이너가 한다.

## 7. 관련 이슈 인계 게이트

계획서의 모든 관련 이슈를 다음 중 하나로 분류한다.

| disposition | 조건 |
| --- | --- |
| `포함` | 이번 구현과 수용 기준에서 완료 |
| `제외-후속 담당` | 범위 밖이지만 이슈·담당자·다음 행동이 정해짐 |
| `차단` | 외부 조건 또는 선행 작업과 해제 조건이 명시됨 |
| `대체됨` | 다른 이슈/PR이 같은 수용 기준을 충족했다는 증거가 있음 |

다음 두 시점에 GitHub 상태를 다시 조회한다.

1. **PR 생성 전**: 관련 이슈의 담당자, 연결 open PR, 남은 수용 기준, PR 본문의 `Refs`/`Closes`
   대상을 확인한다.
2. **merge 후**: 자동 종료 결과와 실제 산출물을 대조한다. 잔여가 있으면 이슈를 open으로 유지하고
   담당자·다음 작업을 명시한다.

“후속에서 처리”라는 문장만 남기고 담당자 없는 open 이슈로 방치하지 않는다. 반대로 계획이나 일부
테스트만 존재하는 상태를 완료로 간주하지 않는다.

## 8. 이슈 조사 기록 템플릿

```markdown
## 환경
- OS / browser / backend / permission:
- snapshot version / generation:

## 원문 face
- family / full / PostScript / style:
- embedded / substFont:

## 감지
- enumeration:
- raw probe:
- state: exact-enumerated | exact-probed | alias-only | style-collapsed | fallback-only | ambiguous

## backend 결과
- Canvas2D effective font / width:
- CanvasKit SFNT / Typeface:
- native/WASM metric:

## oracle
- input/PDF/font hashes and producer:
- page/glyph/first divergence:

## 관련 이슈 disposition
- #NNNN: 포함 | 제외-후속 담당 | 차단 | 대체됨 — owner / next action
```

## 관련 문서

- [Issue #4741 조사](../tech/investigations/issue-4741/README.md)
- [폰트 fallback 전략](../tech/font_fallback_strategy.md)
- [시각 검증 거버넌스](verification/visual_verification_governance.md)
- [PDF/SVG visual sweep](verification/visual_sweep_guide.md)
- [CDP E2E](e2e-cdp.md)
- [로컬 사전 검증](pr_review/local_validation.md)
