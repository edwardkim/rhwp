# 선언된 글꼴 후보의 실제 CanvasKit 소비

Issue: #7445

## 원인과 적용 범위

`samples/issue6034/2912735_court_report_form.hwp`의 U+2024 `․`는 준비된 굴림 face에
없고, 공통 스타일에 명시된 다음 후보 `Haansoft Dotum`에 있다. CanvasKit은 첫 family만
찾고 기본·기호 글꼴로 건너뛰어 `textRun:glyphMapping`을 보고했다.
실제 glyph ID 호출126개에서 누락 문자가 U+2024인 것을 확인했다.

설치 파일의 독립 cmap 대조는 다음과 같다. 글꼴 바이너리를 추가하지 않았다.

| 요청 family | 실제 파일 | SHA-256 | U+2024 |
| --- | --- | --- | --- |
| 굴림 | `~/Library/Fonts/gulim.TTC`, face0 | `ed7502755424c560f4538647eff0833b738fe7f3f4ba4de8fca9ff8c4d6a776c` | 없음 |
| Haansoft Dotum | `~/Library/Fonts/HDOTUM.TTF` | `4e7f536a6715270e06c6cc3b5474456007c193a7754131335195286d889f6757` | `onedotenleader` |

저장된 글리프 위치가 있는 TextRun과 CharOverlap에서 준비된 명시적 family 후보를
선언 순서대로 먼저 확인한다. 선택한 face의 style을 적용하고 공통 원점·전진을 소비한다.
Host 준비도 같은 family 목록과 weight/slant를 사용하고 중복 face는 한 번 요청한다.
준비되지 않은 primary의 기존 차단과 끝까지 glyph가 없는 경우의 진단은 유지한다.
새 진단 source는 `missingGlyphFamily`다.

## 독립 입력과 실제 출력

- HWP SHA-256: `9e68695b504656383f7686b75b3e92d28056201ad967570c28144e4bfce880ac`.
- 독립 기준: `pdf/2912735_court_report_form-2020.pdf`, 원본과 같은2쪽.
- PDF SHA-256: `039f83dda89a7ac1fe3a162b39fdee0afbc43f4cc98af0c48a1223eed38d27a8`.
- 정본192DPI 전2쪽은 Native 최저90.14068%, fresh WASM 최저90.15480%다.
  이 문서에는98% 페이지 조건을 충족하지 않은 실제 배치 px 핀을 추가하지 않았다.
- 실제 Studio file-open의 software CanvasKit은 screen/print 각2쪽, 총4개 PNG를 생성했다.
  준비 검사 통과, unsupported op0, 숨은 Canvas2D overlay 사용 없음이다.
  1쪽의 실제 선택은 `굴림 → Haansoft Dotum`, `glyphCoverageFallback`이며 2쪽은 빈 쪽을 유지했다.
- 초기 JS/WASM과 선언 글꼴 환경·CanvasKit renderer·Host 준비 모듈의 실제 응답을 검사했다.
  두 profile의10개 필수 응답이 frozen package/현재 TS 원문과 일치하고 응답 읽기 오류는0이다.

아래 실제 화면은 UI 장식도 포함한다. 같은 정본 Poppler 방법으로96DPI PDF를 만들어
직접 비교한1쪽은96.91413%, 빈2쪽은100%다. 이 값은 실제 소비 확인이며 위192DPI
Native/fresh WASM 전쪽 판정과 구분한다. 원형 점·제목·표 경계·첨부서면·각주와 빈 쪽을 확인했다.

[실제 CanvasKit review](court-canvaskit-print-p001-review.png) ·
[실제 CanvasKit overlay](court-canvaskit-print-p001-overlay.png)

## 수정 전후 검사

| 검사 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 실제 CanvasKit 공개 SFNT 대조 | exit1, U+2024 누락·선언 face 그림 불일치 | exit0, screen/print 모두 통과 |
| Host family 준비 | exit1, bold face만 요청 | 전체13PASS/0FAIL, 메타데이터만 조회 |
| 실제 법원 서식 | `textRun:glyphMapping`으로 준비 검사 실패 | 4개 출력 모두 준비 검사 통과 |

공개 테스트 글꼴은 기존 NotoSansKR/Pretendard/NanumGothic 파일이다.
primary가 있는 A의 보존, 선언 후보 순서, U+2024를 가진 독립 face와 실제 그림 일치,
미준비 후보의 결측 진단, CharOverlap의 같은 face 소비를 확인한다.
실제 문서 좌표·고정 PNG golden 대신 현재 입력을 독립 face로 그린 대조군과 비교한다.

재현: `npm --prefix rhwp-studio run e2e:issue-7445-font-family`.
이 E2E와 기존 검사3개의 원장 누락을 등록하고 이미 제거한 ruler 실행 파일의 유령 행을
정리했다. 원장은156개 파일/156행, 오류0이다. 시험지 검사를 복원하지 않았다.

로그·cmap·초기 응답·실제 backend·준비 상태와 source hash는 로컬 진단용 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-fe5725dc-20261017/canvaskit-family-fallback-c92/`에 보존한다.

## 현재 전체 판정과 최종 이미지 사용 계획

Rust code head `59b1f5b204`의 전체10514PASS/0FAIL/45skip, 필수18게이트를 완료했다.
이번 TS 수정본의 TypeScript와 Studio2032PASS/2skip도 같은 순차 검증에서 완료했다.
frozen Native/WASM 생산자는 같다. 전쪽 각48조건1745쪽에는 두 경로 각각3입력의
18·35쪽 WMF 미달과 devel 대비0.01%p 하락 Native11쪽/WASM91쪽 검토가 남아 있다.
따라서 전체 PR 준비 완료나 이슈 전체 해결을 주장하지 않는다.

최종 PR·이슈 코멘트에는 위 review/overlay 두 파일을 사용해 실제 점의 표시와 소속을
보여준다. 본문에는 실제 source/head·검증 범위·잔여와 함께 merge SHA로 고정한 두 이미지 URL을
기록한다. 실행 JSON·TSV·로그와 그 밖의 탐색 이미지는 커밋하지 않는다.
