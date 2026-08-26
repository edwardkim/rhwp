# Task M100 #4741 Stage 5 — 부분 열거·Canvas2D·절차 검증

## 범위

Local Font Access API가 설치 face 일부를 누락하는 상태를 강제하고, 문서 후보 raw probe와 snapshot
provenance, exact Canvas2D face, cache, CanvasKit 조달 경계와 편람 페이지 경계를 검증했다. 변경은
`rhwp-studio` TypeScript와 E2E 및 문서에 한정되며 Rust/renderer/WASM 소스는 바꾸지 않았다.

## 자동 검증

- focused TypeScript 37/37 통과
  - `local-fonts.test.ts` 15/15
  - `document-font-status.test.ts` 5/5
  - `font-substitution.test.ts` 9/9
  - `local-fonts-modal-copy.test.ts` 2/2
  - `document-initialization-order.test.ts` 6/6
- `npx tsc --noEmit`: 통과
- 호스트 `npm test`: 931건 중 930 통과, 1 skip, 실패 0
- `npm run build`: 통과
- `python3 scripts/check_e2e_manifest.py`: tracked 102개와 manifest 102행 일치
- `git diff --check`: 통과
- 변경 Markdown 상대 링크 검사: 이상 없음
- 문서 메타데이터 검사: 560개 이상 없음

Rust/WASM 소스 변경이 없으므로 release-test 전체, Native Skia, wasm-pack 재빌드는 이번 변경 범위의
필수 게이트로 확대하지 않았다. production Studio build는 기존 `pkg/` WASM을 포함해 성공했다.

## 호스트 Chrome CDP

- Windows 11 Chrome `151.0.7922.138`, CDP 1.3
- Vite `http://localhost:7700`
- 실행:

```bash
cd rhwp-studio
CHROME_CDP=http://localhost:19222 npm run e2e:issue-4741
```

테스트 탭의 `queryLocalFonts()`만 다음 부분 결과로 강제했다.

- 열거됨: `Issue4741 Enumerated Control Regular`
- 열거 누락·실제 설치: `KoPub바탕체 Light`
- 열거 누락·미설치 대조군: `Issue4741 Missing Control`

결과:

| 항목 | 결과 |
| --- | --- |
| Local Font Access 호출 | 최초 1회, 같은 감지 세대 재호출 0회 |
| KoPub record provenance | `font-presence-probe` |
| snapshot | v3, checked 2, probed 1, unresolved 1 |
| raw effective font | `72px "KoPub바탕체 Light"` |
| patched effective font | exact KoPub face → proportional serif fallback |
| 대표 문자열 raw 폭 | `2003.3277587890625px` |
| 대표 문자열 patched 폭 | `2003.3277587890625px` |
| 폭 delta | `0px` |
| probe-only CanvasKit bytes | `null`, 추가 Local Font Access 호출 없음 |

전역 Canvas font substitution patch가 설치된 상태에서도 probe는 보존한 raw descriptor를 사용했다.
단위 테스트 mock에서 patched setter 호출은 0이고 raw setter 호출은 후보 수와 고정 vector의 상한 안에
머물렀다.

## 편람 HWP 실제 경계

정확한 샘플 경로는 저장소의
`samples/2025 행정업무운영 편람(최종).hwp`이며, Vite의 percent-encoded `/samples/...` URL로
10,687,488 bytes를 가져왔다. Windows Chrome에 WSL 절대경로를 넘기지 않았다.

- backend: Canvas2D
- 페이지 수: 383
- 물리 11쪽으로 이동한 뒤 `KoPub바탕체 Light` 지정 2,104회를 수집
- 대표 요청은 style별 exact face를 포함한 Rust chain
- 제품 patch 뒤 effective font는
  `16px "KoPub바탕체 Light", Batang, AppleMyungjo, "Noto Serif KR", serif`
- `GulimChe`와 `monospace` 없음

Rust 페이지네이션은 local snapshot과 독립이므로 문서를 다시 파싱하거나 재페이지네이션하지 않았다.
편람 383쪽 경계가 유지됐고 Canvas2D paint face만 exact local face로 해소됐다.

## E2E 격리와 복구

초기 하니스는 Vite 동적 import가 app과 다른 module instance를 만들 수 있어 제품 singleton의 font
chain을 관찰하지 못했다. 개발 모드 `window.__localFonts` 진단 표면을 app singleton에 연결해 실제
전역 patch 경로를 검증하도록 정정했다.

하니스 작성 중 7700 origin의 저장 snapshot을 초기화한 사실을 발견한 즉시 실제 Chrome Local Font
Access와 문서 후보로 snapshot을 재생성했다. 최종 하니스는 원래 `rhwp-local-fonts` 값을 백업하고
종료 전에 복원한다. 최종 실행 뒤 읽기 전용 확인 결과는 다음과 같다.

- snapshot v3
- `detectedAt`: `2026-08-15T07:51:13.300Z`
- 375 face family record
- `KoPub바탕체 Light` exact record 존재

테스트 탭은 종료됐으며 메인테이너의 다른 Chrome 탭을 닫거나 변경하지 않았다.

## 문서화

- [폰트 감지·대체 사고 대응 절차](../manual/font_incident_response.md)
- [폰트 fallback 전략](../tech/font_fallback_strategy.md) 6.2 hybrid 감지 계약
- [#4741 원인 조사](../tech/investigations/issue-4741/README.md)

다른 폰트 이슈에서는 환경·원문·열거·probe·선택·backend·oracle 7축 매트릭스와
`포함 / 제외-후속 담당 / 차단 / 대체됨` disposition을 사용한다. PR 생성 전과 merge 후 관련 이슈의
담당자·잔여 수용 기준·다음 행동을 다시 조회한다.

## 판정

#4741의 구현·자동 검증·Chrome 기능 검증과 페이지 경계 확인은 통과했다. remote push, PR 생성,
GitHub 코멘트와 이슈 종료는 아직 수행하지 않았다.
