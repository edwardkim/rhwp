# #7403 호스트 글꼴 공급 — 1차 구현·검증 기록

- Issue: [#7403](https://github.com/edwardkim/rhwp/issues/7403)
- 설계 공유: [이슈 코멘트](https://github.com/edwardkim/rhwp/issues/7403#issuecomment-5809158954)
- 담당자: `postmelee`
- 기준 base: `505661360e9a2d596f55300d0cb0c5222f0e14b4`
- 검증 source: `32a9211c271f811d0b03389542604a649e846c82`
- 브랜치: `codex/issue-7403-host-font-provider`
- 상태: 로컬 1차 구현과 아래 frontend 검증 완료. 정식 Visual Sweep 미실행,
  기존 E2E manifest 오류 잔존으로 제출 게이트 전체 충족을 주장하지 않는다. push·PR 미수행.

## 구현 범위

Studio와 같은 realm의 `rhwpStudio.fonts.setProvider()`로 선택적인 공급자를 연결한다.
메타데이터 목록, 필요한 face 바이트, revision 변경 구독을 분리한다. 공급자 연결 세대와 revision,
ID를 캐시 키로 사용하고 변경·해제 시 요청과 이름 색인을 무효화한다. 호스트 목록은 브라우저
감지 snapshot·CSS 사용 가능 판정·영속 저장과 섞지 않는다.

호스트 공급자 지정 시 CanvasKit의 textRun/charOverlap 경로가 실제 family·weight/slant로 face를
선택한다. 선택한 참조를 읽기까지 유지하고, TTC face index를 기존 SFNT 정규화 함수에 전달한다.
알려진 Bold/Italic face에는 합성 굵기/기울기를 중복 적용하지 않는다.

소비 경로는 `HostFontSource` → `local-fonts.resolveCanvasKitLocalFont` → 실제 PageLayerTree의
`collectHostFontRequests` → `prepareHostFonts` → `findPreparedTypeface` → textRun/charOverlap paint다.
변경 통지는 `main` → `CanvasView.refreshFontResources` → RendererSession 자원 reset 및 선택 세대
갱신 → 필요한 face 준비 → 페이지 갱신으로 연결한다. 문서 모델이나 dirty/undo를 수정하지 않는다.

브라우저 공급자 경로는 기존 매칭·저장을 유지한다. 사용 예와 제한은
[HOST_FONTS.md](../../rhwp-studio/HOST_FONTS.md)에 기록했다.

## 검증

모든 아래 실행은 `/private/tmp/rhwp-7403`에서 위 source SHA를 기준으로 수행했다.
최종 검증 뒤 코드 변경은 없으며 이 기록은 문서 추가다.

| 명령 | 결과 |
| --- | --- |
| `CARGO_TARGET_DIR=/private/tmp/rhwp-7403-target scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` | PASS, fresh dev WASM |
| `npx --prefix rhwp-studio tsc --noEmit -p rhwp-studio/tsconfig.json` | PASS |
| `npm --prefix rhwp-studio test` | 1,783개 중 PASS 1,781, SKIP 2, FAIL 0 |
| `npm --prefix rhwp-studio run build` | PASS, production Studio bundle |
| `CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' npm --prefix rhwp-studio run e2e:issue-7403` | PASS, 실제 Chrome·CanvasKit |
| `npm --prefix rhwp-studio run e2e:canvaskit-font-coverage` | PASS, 기존 번들·옛한글·exact TTC GlyphRun 회귀 |
| `git diff --check upstream/devel...HEAD` | PASS |
| `python3 scripts/check_e2e_manifest.py` | FAIL, base부터 존재한 미등재 3건. 이번 E2E의 등록·배선 오류 없음 |

dev WASM SHA-256: `fcb45acbe3491a981e02c5f752595fd039ca9a3e6826fe3bd730ae7b9801a35f`.
glue SHA-256: `8f01a5bcc227a41d72b4aca57792445e9f413c4400154b74bdf8992f35301df3`.
검증 당시 `pkg/`와 `rhwp-studio/public/` 두 파일의 hash가 각각 일치했다. 생성 glue는 source 변경에
포함하지 않는다. production bundle 검사는 dev WASM을 포함하며 release WASM 검증으로 보고하지 않는다.

manifest의 기존 누락은 `canvaskit-cropped-contain.test.mjs`, `issue-7333-line-selection.test.mjs`,
`probe-flow-input-latency-issue3794.mjs`다. 고정 base의 파일 존재와 manifest 미등재를 각각 확인했다.
이번 작업에서 관련 없는 테스트 분류를 추정해 수정하지 않았다.

## 실제 확인한 계약과 증거

- metadata 조회 후 read count 0. 실제 스타일 수요에서 선택된 face만 읽으며 동시 요청을 공유한다.
- 교체 중 enumeration/bytes의 늦은 응답, 연결 해제, 같은 ID/revision의 다른 공급자, 실패 후 새
  revision 복구를 단위 테스트로 확인했다. 잘못된 목록은 빈 결과와 오류 상태로 처리한다.
- 일반 face의 fullName이 family와 같은 경우에도 Bold run이 Regular로 가로채이지 않는 경계를 포함했다.
- 실제 CanvasKit에서 family+bold 결과와 명시적 Bold face 결과가 동일했다. 같은 이름·개수의
  Noto Serif → Noto Sans 바이트 교체는 다른 픽셀을 만들고, 실패 뒤 Serif 복구는 원래 픽셀을 복원했다.
- 실제 편집 문서에서도 Regular/Bold 두 face를 준비하고 변경 통지가 페이지를 갱신했다.
  HWP 저장 바이트, 문서 세대와 dirty=true를 보존했다. HWP/HWPX 재열기에서 원래 문서 글꼴명과 Bold를 확인했다.
- 최종 `host-font-7403-faces.png`와 `host-font-7403-document.png`를 직접 열어 한글·영문·숫자,
  Regular/Bold, Serif/Sans 교체와 문서 본문 표시를 확인했다.

로컬 증거는 `/private/tmp/rhwp-7403-validation/`에 모았다. 실행 로그, E2E HTML, 두 PNG가 있다.
재현 테스트는 `rhwp-studio/e2e/host-font-provider-issue7403.test.mjs`이며 매 실행 새 캡처를 만든다.

## 남은 범위와 판정

위 결과는 합성 문서·공개 번들 글꼴을 이용한 공급 계약 및 화면 연결 검증이다. 독립 한컴 기준 PDF와의
Native/fresh WASM compare·standalone overlay·review를 생성하는 정식 Visual Sweep은 수행하지 않았다.
따라서 한컴 출력 충실도나 전체 렌더링 제출 게이트를 충족했다고 판정하지 않는다.

Canvas2D FontFace 등록, SVG 내보내기, PDF·인쇄, Rust layout, 호스트 글꼴을 근거로 한 자동 renderer
적합성 확대, 기존 glyph resource 교체, 네이티브 앱 IPC 연결은 미구현/이번 1차 범위 밖이다.
호스트 TTC index 전달은 구현했지만 새 host 경로의 실제 TTC 화면 검증은 남았다.
기존 exact TTC GlyphRun 검사 통과를 그 증거로 대체하지 않는다.
toolbar에 호스트 보관함 목록을 별도로 표시하는 UI도 추가하지 않았다.

최초 E2E의 캐시 개수 실패는 변경 직후 초기화된 상태를 이전 프레임의 상태로 검사한 테스트 시점 문제였다.
24pt 테스트 입력은 HWPUNIT 2400으로 수정한 뒤 최종 E2E·화면 확인을 다시 수행했다.
새 API의 부재나 초기 빌드 환경 실패를 수정 전 결함 재현(RED)으로 보고하지 않는다.
