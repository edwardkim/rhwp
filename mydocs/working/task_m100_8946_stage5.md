# #8946 수용 조건 대조와 PR 제출 전 검증

2026-11-12 KST. 이슈 본문(2026-11-11T10:17:17Z 수정)과 댓글 없음 상태를 다시 확인했다.
명시적 Canvas2D 조판 참여·프로그램 선택·환경 갱신·수명 처리라는 범위를 유지한다.
전체 공급 경로 통합, 모든 출력의 환경 일치, HFT 정책 전면 교체, 기본값 활성화로 확장하지 않는다.

## 기준과 변경

- 최신 devel `4c876654d8`로 리베이스했다. 직전 기준 `99b9d7ec5c` 이후 추가 기능은
  #8945의 RowBreak 바닥 여백 예산 보정이다. 실제 기능 보존 검증은 아래에 별도로 기록한다.
- 직전 최종 표지 근거는 [단계 4](task_m100_8946_stage4.md)다. 같은 세 그림을 포함한 독립
  한컴 Print로 원본 외부 공급 출력과의 동등성을 확인했다. 그림 제외 비교를 최종 판정으로 쓰지 않는다.
- 사용자 계정 `postmelee`의 열린 PR은 없었다. 원격 push·PR 생성·댓글은 실행하지 않았다.

## 수용 조건과 보호 경계

| 수용 항목 | 구현·정식 보호·실제 검증 |
| --- | --- |
| 명시적 opt-in과 기본값 보존 | HOST_FONTS.md의 연결 단위 `{layout:true}`. provider 단위 검사, #7403 두 backend E2E, 최신 base/후보의 none·omitted·false 전쪽 실제 출력 대조. |
| 첫 화면의 실제 프로그램 | 선언 프로그램 준비 → indexed SHA-256과 faceIndex → Rust 선택 → 공통 측정·페이지 paint. HWPX/HWP3 실제 파일 trace와 초기 화면·전쪽 출력으로 확인한다. |
| 같은 ID의 revision 교체·TTC | #7403 E2E의 A→B, CFF 교체, 같은 TTC의 face 교체, 실제 독립 face와 pixels/width 비교. |
| 제거·재공급·해제·실패 복구 | 각 상태의 열린 문서 tree/paint를 같은 상태의 새 문서와 대조한다. 이번에 read-failure/recovery·detach/reattach를 추가했다. |
| 지연 읽기의 세대 격리 | 옵션 변경·같은 ID/revision의 공급자 교체·문서 교체 중 이전 응답의 무효화 E2E와 provider 단위 검사. |
| 사용자 환경·내장 우선순위 | 자동 환경 소유권 E2E, 실제 선택 프로토콜의 embedded source/substitute 제외, host 선언 프로그램의 embeddedFamilies 제외 단위 검사. |
| 원본 메트릭 보존과 명시적 교체 | 자동 preserveSourceMetrics 유지, 명시 원본만 replaceSourceMetricsFor, 나머지 원본 정책 유지. 기존 선택 프로토콜과 E2E를 연결한다. |
| 저장·편집 이력·portable | 전환 상태별 HWP/HWPX 저장→재열기→재저장→재열기, 원래 family·내용 보존, 내부 별칭 비유출을 추가했다. dirty/undo/redo와 portable SVG 계약은 기존 실제 UI E2E로 보호한다. |
| 저장 줄과 편집 재조판 | 원본 HWPX/HWP3의 실제 편집/복원과 새 HWPX 합성 경계의 저장 줄·재조판 양쪽 소유·내용 순서·겹침 검사. |
| 최종 제출 검증 | 최소 글꼴 전체 회귀·Native Skia 3종·lint 전체 묶음·TypeScript·Studio tests·fresh WASM·전쪽 TSV·확대 판독·편람 gate. 실행 완료 결과만 아래에 기록한다. |

## 추가 회귀와 독립 기대값

`tests/cases/hft_outline_font_contract.rs`의 기존 HWP3 계약을 유지하고 다음을 추가했다.
합성 입력은 원본 HWPX에서 파생했으며 한컴 저장본으로 표시하지 않는다. 사용한 글꼴은 저장소의
MIT 합성 fixture뿐이며 사설 OTF를 테스트·배포물에 넣지 않았다. 신규 렌더링 회귀의 선행 근거는
단계 4의 전 763쪽 Native/fresh WASM 최저 96.34886%/99.56218%와 단계 3의 문장부호 확대 판독이다.

- CFF 상속 슬롯에 글리프가 없고 원문 영문 CFF가 공급할 때 실제 run의 소유를 영문으로 유지한다.
  저장 줄·재조판 양쪽에서 내용 순서·누락·중복·run 겹침·뒤 한글의 소유를 검사한다.
  영문 미공급, 상속 슬롯에 글리프가 있음, TTF 대체, 비 HFT 선언의 비적용 대조군도 포함한다.
- 세나루·강낭콩·딸기의 굵기 전진은 독립 Print의 별도 TJ 50/1000 em을 근거로 한다.
  원본 HFT 선언과 실제 CFF 프로그램의 이름을 함께 확인하고 최종 run의 측정에 도달하는지 검사한다.
  비 HFT·내장 선언·미확인 실제 프로그램에는 자격을 빌려주지 않는다. 실제 좌표를 고정하지 않는다.
- 측정용 SFNT의 원본 cmap·hmtx 보존 검사를 공개 NativePuaFonts 등록/측정 API를 사용하는 기존
  integration suite로 이관했다. source-side cfg(test) 증가 금지 정책을 준수하며 보호를 없애지 않는다.

최신 devel에 같은 회귀 파일만 주입한 수정 전 실행은 14 PASS / 2 FAIL이었다. 실패는 HWPX CFF
문장부호 소유와 검증된 굵기 전진의 두 신규 검사에 한정된다. 후보는 동일한 최소 글꼴 조건에서
16 PASS였다. 기존 HWP3와 프로그램 선택·collection·cmap 대조군은 양쪽 모두 통과했다.

## 로컬 검증 진행 기록

회귀 격리는 macOS sandbox로 개인·상용 글꼴 파일 읽기와 font service를 차단한다. OS 기본 글꼴,
저장소 공개 기본 글꼴, 명시적으로 공급하는 라이선스 확인 fixture만 허용한다. 기존 suite가 사용하는
재배포 승인 HappinessSansVF.ttf는 정확한 파일 하나만 추가 허용했다. 개인 공급 폴더 71개 파일의
명시적 읽기가 모두 거부되는 것을 확인했다. 시각 검증의 71개 실제 프로그램 공급과 구분한다.

초기 Studio 테스트는 2,789 PASS / 2 SKIP, TypeScript와 기본 Rust lint 묶음이 통과했다.
회귀 보강 뒤에는 최종 실행을 별도로 기록한다. 파생 suite 변경에 따른 drift는 --prepare로 다시
생성했으며 suite-check·unit-tier 정책을 통과했다. 파생 파일은 커밋하지 않는다.

보강된 #7403 E2E는 기존 제품 런타임에서 Canvas2D/CanvasKit 모두 통과했다. 이 선행 결과를
리베이스 후 fresh WASM 최종 검증으로 대신하지 않는다.

공유 target에서 base를 빌드한 직후 후보를 빌드했을 때 Cargo의 fresh 판정으로 top-level 산출물이
base와 같은 해시로 남았다. 후보 WASM에 새 `clearTextFontOutline` API가 없는 것도 확인했다.
이 최초 후보 산출물은 검증에 사용하지 않고 소스의 mtime을 갱신해 실제 재컴파일했다.
코드 내용이나 Cargo 설정·한도는 바꾸지 않았다. 최종 producer 신원은 재컴파일 결과로만 기록한다.

최신 devel의 #8945 대조군 5개, 총 8쪽은 Native/fresh WASM 모두 raw 100.00%다.
후보에서도 같은 입력과 독립 Print로 검증하고 실제 픽셀·트리를 대조한다.

## 최종 시각 검증과 리베이스 보존

최종 코드 head는 `6b7ceb9cd5f11e8c4c073aeadc032d84c0767b2e`다. 재조회한 upstream/devel은
`4c876654d86d071d342fc78706c51cfff6ddbde4`로 동일했다. 이슈 본문의 수정 시각도 그대로다.
최종 원격 조회에서 본인 열린 PR은 0개였다. 새로 확인한 #8957은 같은 원본의 Microsoft Print
프레임·글리프 윤곽 조사이며 #8946의 Studio 호스트 선택·수명 범위와 구분한다. 아래 한컴 2024
독립 출력의 일치를 #8957 해결로 주장하지 않는다.

| 입력·763쪽 | Native raw 최저 | fresh WASM raw 최저 | 실제 Host Canvas2D raw 최저 |
| --- | ---: | ---: | ---: |
| HWPX | 96.34886% | 99.56218% | 99.35730% |
| HWP3 | 98.77175% | 99.54840% | 99.35495% |

[최종 증적](../pr/assets/issue8946/stage5-20261112/)에 각 Native/fresh WASM 전쪽 TSV 원본,
대표 PNG, 실제 호스트 화면 TSV·첫 화면 결과·초기 JS/WASM 응답 해시를 보존했다.
모두 독립 PDF와 763쪽이 일치하고 그림 영역을 제외하지 않았다. HWPX 표지는 Native/fresh WASM
각 100.00%다. HWPX의 입력·동일 세 그림·독립 Print 출처는 단계 4의 계약을 그대로 사용한다.

HWP3 프로덕션 번들의 기본 사이드카 자동 읽기는 비활성이라 처음에는 표지 그림이 없었다.
그 첫 결과(표지 32.50%)는 통과 근거에서 제외하고, 원본의 같은 세 그림을 기존
`injectExternalImage` API로 문서 로드 직후, 첫 환경 준비 전에 명시 공급했다. 각 그림의 SHA-256과
주입 수 1을 기록했다. 제품의 사이드카 UI나 저장·출력 계약을 변경하지 않았다.

리베이스 전 단계 3과 최종 후보의 HWPX/HWP3 Native/fresh WASM은 각각 763쪽 모두 PNG와
render tree가 바이트 단위로 동일하다. 실제 HWPX 호스트 화면도 단계 4의 전 763쪽 tree/PNG와
같다. 따라서 단계 3·4에서 완료한 표지의 세 그림·문구, 본문 제목 23곳, 문장부호 350·608·686쪽,
729쪽 및 HWP3 완료 좌표의 직접 확대 판독 근거를 그대로 연결한다. 새 300 DPI 렌더를 실행한
것으로 표시하지 않는다. 이번 실제 편집 전후 표지와 #8945의 b 사례 2쪽도 직접 판독했다.

공급자 없음·옵션 생략·명시적 false 각각을 최신 base/후보에서 새 프로필로 실행했다.
원본 HWPX, 그림을 포함한 HWPX 운반본, 같은 그림을 공급한 HWP3의 3 × 3 조건에서 각 763쪽의
tree/PNG 해시가 모두 같다. 기본 모드의 쪽수·글자 폭·줄바꿈·표 높이·화면을 보존했다.
#8945의 5개 입력·8쪽 역시 양 backend raw 100.00%, base/후보 tree/PNG 모두 동일하다.

## 실제 편집 후 줄과 빈 문단 점유

HWPX/HWP3에서 표지 문단 23의 첫머리에 `검증`을 넣고 삭제했다. 각 상태 모두 763쪽이며
전 문서 내용 순서, 삽입 문단 소유, 기존 내용의 누락·중복 없음, 나머지 가시 run의 geometry와
다음 쪽 제목차례를 보존했다. 변경 run은 본문 안에 있고 앞뒤 줄은 겹치지 않는다.

처음 작성한 진단의 ‘편집·삭제 후 모든 빈 run의 좌표도 원본과 같아야 한다’는 조건은 실패했다.
이 실패를 숨기지 않고 원본 XML, 빈 문단 처리, 실제 저장 후 재열기를 대조했다. HWPX는
`hideFirstEmptyLine=1`, 대상 저장 줄 높이 1800 HU·간격 1080 HU·가용 폭 31436 HU이며
뒤 빈 문단 24·25는 높이 800 HU·간격 480 HU다. 추가 글자로 가용 폭을 넘으면 두 줄이 되고,
뒤의 빈 두 줄은 넘침 숨김 경로에서 같은 끝점에 흡수된다. 삭제는 undo나 저장 LineSeg 복원이
아니므로 재조판된 빈 줄 상자가 다시 활성화될 수 있다.

진단을 내용·소유·포함·겹침과 빈 줄 점유 관계로 분리했다. 빈 줄도 검사하며, 가시 글자가 없다는
이유로 제외하지 않았다. 활성 빈 줄은 본문 안에서 앞 줄 뒤를 점유하고, 숨김 줄은 최대 두 개이며
본문 끝점에서 소유를 유지한다. 저장 후 새 bridge에서 재열어도 편집된 제목과 빈 줄의 세로
흐름·763쪽·다음 제목차례는 유지됐다. HWPX의 재열기 후 빈 줄 가로 가용 폭은 live와 다를 수
있으므로 이를 가시 글리프 좌표나 저장 원점의 일치로 승격하지 않았다.
이 경로의 제품 코드나 기대값을 임의 좌표로 바꾸지 않았다. 상세 관측은
[HWPX 편집](../pr/assets/issue8946/stage5-20261112/edit-hwpx.json),
[HWP3 편집](../pr/assets/issue8946/stage5-20261112/edit-hwp3.json)에 연결한다.

## 검증 환경 오류의 보정

- 전체 회귀의 유일한 실패는 기존 SVG full-font embed 검사가 직접 만든 `regular`/`bold` 임시
  파일을 sandbox가 차단한 것이었다. 해당 테스트의 PID별 생성 디렉터리 상위만 허용하고 같은
  테스트를 다시 실행해 PASS를 확인했다. 기대값·제품 코드·개인 글꼴 차단은 그대로다.
  최종 허용 목록은 OS·공개 fixture 430개이며 개인 공급 파일 71개 읽기는 모두 거부됐다.
- 편람 gate의 초기 실행은 368쪽 raw 99.97%로 기준 100.00%에 미달했다. 이전 해결 기록을
  대조한 결과 CLI의 `RHWP_WEBFONT_CACHE` 누락으로 Noto Serif 읽기가 차단돼
  KoPub바탕체 Light 슬롯 5/font 13의 fallback이 HCRBatang만 남았다. 테스트용 설정과 별도로
  기존 시각 격리 profile 및 T7 CLI 캐시를 함께 지정했다. 실제 환경 JSON은 이전 통과 환경
  `d68b9023…`와 같고, 최종 head의 383쪽 gate는 passed·기준 하락 0쪽·최저 99.96465%다.
  기준 TSV나 허용치를 바꾸지 않았다.

## 최종 로컬 검증 결과

| 검사 | 결과 |
| --- | --- |
| 최소 글꼴 전체 회귀 | 13,235개 선택, 최초 13,234 PASS / 임시 fixture 차단 1 FAIL. 환경 보정 뒤 해당 1개 PASS. 미해결 0, suite 지정 skip 45. |
| Native Skia library | 3,852 + 15 + 165 + 2 PASS, 기존 ignore 12. |
| Native Skia placeholder / direct PDF | 2 PASS / 35 PASS. |
| Rust fmt / Native·WASM·workspace all-targets Clippy | 모두 PASS, `-D warnings`. |
| suite manifest / unit-tier 정책 | PASS. |
| TypeScript / Studio | PASS / 2,789 PASS, 기존 skip 2. |
| frontend binding·editor contract / E2E manifest | PASS / 190개 일치. |
| fresh WASM #7403 E2E | Canvas2D·CanvasKit PASS, 조판 계약 97개 모두 true. |
| 최종 시각 검증 / 편람 | 위 6개 763쪽 묶음 모두 90% 이상 / 383쪽 gate passed. |

[검증 결과 원장](../pr/assets/issue8946/stage5-20261112/local-verification-results.json)에
최초 전체 실행의 실패와 해당 원인의 보정·재실행을 구분해 기록했다. 전체 실행 한 번이
처음부터 전부 녹색이었다고 표시하지 않는다. 원시 로그는 ignored output에만 보존하고
정식 원장에는 해시와 판정·수치만 기록한다. 자동·명시 메트릭, 내장 우선순위, 실패 복구·세대 경쟁,
저장·dirty/undo/redo·portable 계약은 기존 보호와 보강한 E2E를 함께 통과했다.

원본·운반본·PDF·프로그램의 정체성, 초기 런타임 응답, 전쪽 수치와 직접 판독 근거까지 연결했다.
#8946 범위에서 남은 미해결 필수 검증은 없다. PR 제출안과 로컬 커밋을 준비하며 원격
push·PR 생성·이슈 댓글은 이번 준비 작업에 포함하지 않는다.


## PR 제출 준비

제목: `feat(studio): 명시적 호스트 글꼴 조판 참여와 프로그램 수명 연결 (#8946)`.
본문은 ignored `output/issue8946/stage5/pr-body.md`에 준비했다. source·test commit과 이 검증
기록 commit을 같은 `task/8946-host-font-layout`에서 제출한다. 최신 base와의 merge-tree는
충돌 없이 성공했다. 검증 뒤에는 문서·증적만 추가하며 runtime source는 그대로다.
원격 push·PR 생성은 아직 수행하지 않았다.
