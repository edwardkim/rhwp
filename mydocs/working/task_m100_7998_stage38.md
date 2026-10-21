# #7998 PR 제출 통합 검증 — 2026-10-21

작업지시자의 “원격 devel의 head와 머지한 후 PR 등록 절차” 지시에 따라 진행한다.
원격 push·Open PR 등록은 이 지시의 범위이며, CI 후 self-review·merge·issue close는 별도다.
기존에 수용된 출력 예외·후속 이슈와 새 글상자 편집 통과 판정은 유지한다.

## 통합 결과

- 구현 보존 commit: `e58323511` (stage37까지의 소스·회귀·작업 기록).
- 최신 원격 devel: `2e3b4f880bd11d35398e6ea0100846611f13a72d` (#7930).
- merge commit: `49cd265223b818faa667de84322001f5e318563a`, 충돌0개.
- 실제 유입 diff: Qt 참조 창·회귀·설명 및 native clipboard HTML 왕복, 총7파일.
- 원 PR #7930의 본문·기여/메인터너 보정과 실제 clipboard diff를 확인했다. 대기 geometry
  revision 검증과 빈 clipboard 거절, 공백·emoji·다중 문단·undo/redo 보존이 유지 대상이다.
- 본인 PR 준비 경로는 collaborator_self_merge이며 intake_and_review·local_validation·
  visual_fixture_evidence를 함께 적용한다. PR 번호·review·오늘할일을 미리 만들지 않는다.

## 실행 범위와 증거

필수 전체 Rust 회귀·Native Skia·lint 묶음·doc test, frontend와 실제 WASM/Worker/package,
최신 기준과 후보의 시각 증거·입력 대응을 확인한다. 이미 같은 제품 바이트에서 확인한
시각/편집 판정은 근거를 대조해 재사용하며 새 코드·기능의 미실행을 성공으로 대신하지 않는다.
실패한 검사는 원 해결 PR·통합·독립 기준을 확인하고 원인을 보정한다. 현재 수정의 회귀를
baseline 완화나 자동 이관으로 숨기지 않는다. 로그·중간 JSON은 ignored
`output/pr-review/issue7998/pr-registration-20261021/`에 보존한다.

현재는 병합과 준비만 완료했으며 전체 검증·PR 등록은 진행 중이다.

## 162쪽 실제 저장본의 추가 Print와 전각 공백 보정

실제 WASM 편집·복원 HWP/HWPX를 한컴2020 `11.0.0.9136`에서 정상 열고,
전처리 없이 원본 용지·100%·한 쪽씩 `pdf_print_method=0`으로162쪽을 인쇄했다.
초기 전쪽 Native/fresh WASM 대조는 양 형식 모두43쪽만 미달(약59%)이었다.
`행정종합관찰관` 앞 U+3000의 장평197%가 한컴에서 마지막 글자를 다음 줄로 넘겨
뒤 행을 밀었다. 같은 원본43쪽의 devel 저장·새 한컴 Print에는 이 줄 넘침이 없어
이번 변환 경로의 결함으로 판정했으며 기존 출력 예외에 포함하지 않았다.

값의 경로는 `structured_lowering::GlyphWidths::measure → observed_line_text의 ratio/spacing
fit → 공통 char_width_decision → 저장 CHARSHAPE → 한컴 실제 글줄`이다.
HCRBatang의 생성 표에서 U+3000은0(미수록)인데 일반 반각 폴백을 선택했다.
실제 HANBatang.ttf의 U+3000·한글 hmtx는 둘 다970/1000이며, Unicode 전각 공백을
반각으로 간주할 근거가 없다. 메트릭 hit는 유지하고 미수록 U+3000만 전각 폴백을
선택한다. ASCII·NBSP·HWP 고정폭 빈칸 계약은 그대로 유지한다. 특정 문서/수치 조건을
추가하거나 공백을 삭제하지 않았다.

보정 후 같은 셀의 literal U+3000과 문구를 유지하며 장평100%가 저장된다.
새 한컴43쪽 Print에서 한 줄 보존을 확인했고 Native는 원본PDF99.99374%, 새 Print
99.55895%다. 전체162쪽 양 저장 형식의 새 Print·Native/fresh WASM, 영향 대조군과
전체 Rust 검증은 이 변경 제품으로 다시 확인한다. 이전 source의 release build/lib는
통과했으나 새 코드의 최종 게이트로 재사용하지 않는다. 진행 중 integration은 중단했다.
개별 명령·실패/성공 원본은 ignored `pr-registration-20261021/space43/`에 보존한다.

### 새 제품의 집중 검증과 CI 글꼴 경계

수정된43쪽의 새2020 Print 대비 Native99.55895%·fresh WASM99.57078%를 확인한 뒤
공통 공백 관계 검사와 기존 실제 Worker 검사 안의 저장 회귀를 추가했다. 같은 Worker
검사를 수정 전 동결 WASM으로 실행하면 정확히 `197%` 저장 단언에서 실패하고,
새 Docker WASM에서는138검사·실제7 Worker가 통과한다. OS 글꼴 설치 여부가 아니라
원본의 literal 공백·문단 단일 소유와 저장 CHARSHAPE를 직접 확인한다. CI의 기존 실행
경로를 재사용하며 별도 workflow 호출이나90% 허용치 완화를 추가하지 않는다.

기존 Native158입력169쪽은 모두 이전 PNG와 같고, Helvetica6건의 정식 글꼴 예외도
원래 raw89.54211/89.67777%로 남는다. 기존 front-end font asset/Canvas font binding
CI 경로와 공통 메트릭의 독립성을 확인했다. 로컬 상용 글꼴이나 개인 PC 절대 경로를
CI 성공 조건으로 새로 넣지 않는다. 새 manifest1634 source/48 target, unit-tier4101
검사를 확인했다.

162쪽 전체 인쇄는 새 HWPX의2020 worker가85쪽, HWP의2020 worker가49쪽에서
동일한 worker 종료 코드로 실패했다. 이를 정상 열림/Print 성공으로 처리하지 않았다.
한컴2024 `13.0.0.3901`, 전처리none·Print method0·원본 한 쪽 인쇄에서 HWPX162쪽
완료를 확인했으며 새 Native/fresh WASM 전쪽 최저97.08677%/97.12104%, 누락·쪽수
차이0이었다. HWP의 같은2024 인쇄와 나머지 최종 검증은 진행 중이다. 기존2020
출력과 새2024 출력의 provenance·쪽수·TSV를 서로 바꾸어 보고하지 않는다.

### 최종 source 고정과 실제 편집 복원본

공백 보정과 정식 회귀를 `1f24b1526`에 커밋했다. 이후 제품 source 차이는 없다.
기존169쪽 전체를 새 Native/fresh WASM에서 다시 산출했고 PNG·공개 TSV가 모두
이전 수용본과 같다. 추가 실사용 원본138쪽/backend도 새 실행에서 모두 이전 PNG와 같아
기존 출력 예외·백로그를 그대로 유지했다. 이 수치를 합산해 일반90% 통과라고 하지 않는다.

새 실제 Studio의162쪽 열기·셀 삽입/직접 삭제·양 저장/재열기를 확인했다. 동시 빌드와
캡처 중 첫 화면25.33초는 통제된 성능 재측정이 아니므로 기존17.82초와 속도 비교하지 않는다.
셀 삽입/삭제63/51ms, 스크롤 최대 프레임 간격47.7ms는 이번 기능 실행의 관측값이다.
무편집 Native 저장과 실제 편집 복원본의 section0.xml이 달라 독립 Print를 새로 요청했다.
실제 복원본 양 형식의 한컴2024 Print 각각162쪽, Native/fresh WASM 각324쪽은 전쪽
최저97.08677/97.12104%, 누락·쪽수 차이0이다.43쪽은99.56189/99.57354%다.

같은 배율의43쪽·최저162쪽 직접 대조에서 남은 머리행 선/일부 제목 줄 표현은 B26에
구체적으로 기록했고 기존 범위 종료 지시를 유지한다. 독립 원본 PDF의 외형 보존과
한컴 재인쇄의 표현 차이를 구분한다. 공개 자료는
[submission-20261021](../pr/assets/issue_7998/submission-20261021/README.md)에 있다.

CI 경로의 Studio/Chrome/Firefox/VS Code 배포 빌드와 글꼴/라이선스8검사·실제 Canvas
font binding8검사·확장 배포3검사를 통과했다. 첫 배포 글꼴 검사 실패는 미생성 dist 경로,
확장 첫 빌드 실패는 이 worktree의 미설치 Node 의존성으로 분리했다. 해당 lockfile 그대로
npm ci 후 실제 배포를 만들고 검사를 다시 성공시켰다. 글꼴 공급이나 허용치를 바꾸지 않았다.
최종 source의 release 빌드는21분7초에 성공했고 나머지 필수 Rust 검증은 진행 중이다.

### 전체 회귀 컴파일 호환과 글꼴 예외 source 고정

최종 release lib는3,984 PASS/실패0/기존 ignored12건이었다. 이후 전체 nextest 컴파일에서
기존 anomaly 보조 생성자 두 곳의 `GroupNode.paint_transform` 누락과 최신 devel의
bitmap 보조 순회 두 곳의 새 `LayerNodeKind::ClipPath` 누락을 검출했다. 전자는 회전 없는
기존 입력의 `None`, 후자는 기존 ClipRect와 같은 child 순회로 보완했다. 기대값·assertion·
제품 코드·baseline을 바꾸지 않았다. 테스트 호환 commit은 `2681da929`·`da77cfc4d`이며
제품 source는 여전히 `1f24b1526`과 같다. 실패 컴파일 로그를 보존하고 전체 nextest를 재개했다.

정식 글꼴 예외의 도구 검사는 과거 source SHA를 현재 코드의 성공으로 받아들이지 않아
기존 증거의 `bd707a...`를 거절했다. 새 producer의6건 PNG/TSV와 비어 있지 않은26개
node 기하가 기존 증거와 모두 같은 것을 확인했다. 글꼴 선택 코드·공급 프로그램이 같은
기존 실제 Native 관측 trace의 재사용을 명시하고, 최종 테스트 호환 head에서 예외 JSON과
도구 판정을 다시 고정한다. 허용치나 source SHA 검사 자체를 완화하지 않는다.

### 추가 승인된 그림 기준선 원본의 세 쪽 보정

작업지시자는 `tac-img-02.hwp`26·45·53쪽의 기존 미달도 이번에 보정하도록 지시했다.
기존 base/후보는 같은66쪽 중 세 쪽이0/65.28/49.51%였다. 이 범위를 백로그나 기존
글꼴 예외로 이관하지 않는다. 원 해결#634·#6708·#7902 통합과 독립2020 Print를 대조했다.

- 26쪽: 뒤쪽61쪽의 `NewNumber(Page,1)` 존재만으로 앞쪽 전체의 명시적 `PageNumberPos`를
  숨겼다. 소스 제어의 실제 소유 쪽에서 위치 설정을 활성화하며, 번호 재시작과 감추기를
  별도 상태로 유지한다. Typesetter와 Paginator가 같은 `PageControlEvents`를 소비한다.
- 45·53쪽: 저장 글줄의 선행 텍스트/공백 행을 이미 배치했으나 `tac_reconcile::measure`
  → `commit_tac_capped_bottom`에서 표 줄만 센 상한으로 되감았다. 다음 표가31.05px 위로
  이동했다. formatter의 글줄/간격과 원본·측정 표의 높이 및 뒤 문단의 닫힌 사다리가
  일치하는 경우 선행 행부터 마지막 표 줄까지의 공통 점유를 보존한다. 변경/분할/무효
  저장 줄에는 이 계약을 적용하지 않는다. 첫 진단은45쪽99.70%·53쪽99.94%다.
- 26쪽의 번호는 원본 `쪽 번호` 스타일의 굴림10pt다. SFNT hhea의 하강값은145/1024em이며
  독립 PDF의 Gulim 기준선798pt와 대응한다. 기존 모든 TTF에 적용하던1/3em 여유는 이
  원본 번호를 약2.6px 위로 배치했다. 메트릭 별칭을 통해 원본 프로그램의 하강 메타데이터를
  공급하고 HFT·미수록 프로그램은 기존 계약을 유지한다. 숫자/문서ID 기준 이동은 사용하지 않는다.

진단 글꼴 경로에 여러 큰 TTC를 함께 넣으면64MiB catalog 예산으로 뒤 파일이 공급되지
않았다. 한도를 바꾸지 않고 이 문서의 실제 굴림·맑은고딕과 HCRBatang/Arial을 예산 내에서
명시 공급한다. Gulim.ttc face0를 fontTools로 개별 TTF에 추출했으며 글리프·전진폭·hhea는
그대로다. 글꼴 파일과 개인 절대 경로는 커밋하지 않는다. 바이트/hash/명령은
`pr-registration-20261021/fix17/picture-font-provenance.json`에 기록한다.

현재 새 Native 빌드·세 쪽 재출력과 전쪽 검증을 진행 중이다. 진단 probe의 높이 보정
성공을 최종 Native/fresh WASM66쪽 완료로 보고하지 않는다. 진행 중이던 전체 Rust
검증은 새 제품 변경 전에 중단했으며 최종 제품이 고정된 후 다시 실행한다.

### 세 쪽 보정의 전쪽 자격과 관계 회귀

제품 commit은 `d1d3091c5`다. 같은 원본/독립 Print66쪽을 새 Native와 fresh Docker WASM으로
각각 다시 출력했다. 최저는 Native96.39090%·WASM95.51053%, 목표26/45/53쪽은
각각 Native100/99.76853/99.97208%, WASM100/99.81509/99.97212%다.
같은 실제 글꼴 공급의 base Native66쪽과 비교해0.1%p 이상 하락은0이다.
새 표지 번호 노출은 최종 감추기 제어를 적용해 보정했으며61쪽은1,62쪽은2로 재시작한다.

`issue_7998_tac_prefix_numbering`은 원본 전체66쪽의 실제 CLI render tree에서 빈26쪽의
번호·뒤 재시작과 표지 감추기,45쪽 표→빈 줄→다음 표의 비겹침,53쪽 표→제목→표 순서를
검사한다. 같은 검사 두 건이 보정 전 동결 producer에서 의도한 원인으로 FAIL, 새 producer에서
PASS였다.26쪽은 양쪽 raw100%·직접 확대 판독을 완료한 쪽으로 독립 PDF798pt 기준선만
96dpi±2px로 보호한다. 설치 글꼴 파일을 CI의 기대값이나 성공 조건으로 사용하지 않는다.
원본·PDF hash와 전체 TSV·PNG는 `three-page-final/`에 보존한다.

기존#6708의 표지1쪽은 양쪽98%에 미달하므로 고정 page-origin pin을 유지하지 않는다.
실제 포함 글줄의 원점과 원본 baseline/그림 ascent의 정렬, 폰트 크기별 독립 Print의 상대
이동,66쪽 보존과 같은 줄 그림의 공통 원점으로 검사한다. 잘못된 baseline은 실제 행의
상단 정렬을 확인한다. 구현 helper의 반환값을 기대값으로 쓰지 않는다. 집중7건 PASS다.

새 Native의 기존 P5 각169쪽 및 추가 실사용138쪽 PNG는 각각 이전 수용본과 모두 같다.
실제 Studio 편집 복원 HWP/HWPX의 새2024 Print 각162쪽 대비 Native 최저는 둘 다
97.08677%이며 누락·쪽수 차이는 없다. 새 실제 Worker7개/138검사도 PASS다.
WASM 영향 범위 재검증과 최종 전체 Rust/lint는 별도로 계속 진행한다.

자동 쪽번호 원 계약의 `aift`는 추가적인 읽기 전용 비교에서 원본74쪽의 기존 미달이 남는다.
동일 공급의 base/후보 Native는 모두 같은10쪽 미달이며0.1%p 이상 하락이 없다. 이는
이번 tac 원본66쪽의 시각 자격을 대신하는 신규 fixture나 완료 근거가 아니다. 이 문서의
기존 본문 결함으로 검증 범위를 확대하거나 기존 회귀를 임의 이관하지 않았다.

### 최종 재검증 진행과 메인테이너 판정

메인테이너가 Studio7793의26·45·53쪽 새 탭과
`E:\tmp\issue7998-three-page-review\index.html`에 **시각 판정 통과**를 주었다.
같은 제품의 실제 Docker WASM 로드·66쪽·console오류0을 확인했다. Native/fresh WASM
각169쪽 고정 집합과 추가 실사용138쪽은 RGB 세 채널 비교에서 이전 수용본과 모두 같다.
RGBA alpha 차이가0인 것을 RGB 동일성의 대용으로 쓰지 않는다.

최초 공급에 실제 맑은고딕 bold를 보완한66쪽 별도 대조도 양쪽90% 이상이다.
기존64MiB catalog 예산을 유지하며 HCRBatang의 실제 문서 문자열/한글/Latin/기호 부분 집합과
굴림TTCface0 추출본, 실제 맑은고딕 regular/bold를 공급했다. 선택한15,679자의 advance를
원본과 대조했으며 font metadata와 원본 glyph를 보존했다. 최초 수용 PNG와 추가 공급의
RGB 변화는 별도로 기록한다. 글꼴 파일은 커밋하지 않는다. 공개 출처는
`submission-20261021/tac-baseline/font-programs.json`, 전체 원문 TSV는 같은 폴더다.

정식 회귀/기존 계약 정리 commit은 `73de01f03`, 새 테스트의 rustfmt만 반영한 최종 테스트
head는 `168e96b86`다. 제품 코드는 `d1d3091c5`와 같다. 뒤의 문서/증거 commit이 이 실행의
코드를 바꾼 것으로 표시하지 않는다. 최종 release 빌드는1,194.46초에 성공했으며 release
lib부터 전체 nextest·Native Skia·lint·workspace·doc·Qt FFI를 순차 실행 중이다.
새 테스트가 generated suite에 연결된 뒤 fmt check가 서식 차이를 검출해 해당 파일만
rustfmt했다. 의미/제품 변경이 없어 이미 성공한 빌드와 진행 중 lib 검사를 반복하지 않는다.

기존 Native Helvetica6건은 최종 테스트 head의 JSON에 고정하고 실제 도구가6건 모두
`font_mismatch_exception`을 반환했다. 같은26개 non-empty geometry node와 실제 fallback
trace 재사용 이유를 공개 증거에 연결했다. 새 CI 조건/허용치 완화는 없다.

읽기 전용으로 대조한 `aift`74쪽의 기존 출력 미달은 동일 원본의 열린
[#7791](https://github.com/edwardkim/rhwp/issues/7791)에 이미 등록돼 있다. 해당 이슈의 지시대로
범위 밖의 정상 roundtrip/bookmark/page-number 검사를 유지하며, 이번 tac66쪽 보정의 완료
근거나 새 정상 fixture로 사용하지 않는다.2022 PDF 진단과 기존2020 Print 증거도 구분한다.

### 앞선 전체 회귀17실패의 처리 근거

이전 `1021f7ab4`의 전체 실행은11,206건 중11,189통과/17실패/기존45skip였다.
각 검사의 원 해결 commit·PR과 현재 독립 출력을 대조한 뒤 다음과 같이 처리했다.
이 실패 실행을 최종 성공으로 덮어쓰지 않으며 최종 전체 실행 결과는 별도로 기록한다.

| 범위 | 보정과 유지한 독립 계약 |
|---|---|
| #7902·#6655·#4068 그림 원점4건 | 빈 inline 그림에도 표와 같은 source-aware 문단 offset을 적용. paint/hit가 같은 원점을 소비하며 독립 Print의107.14267px±2 기대는 유지 |
| #7958 공백1건 | 가용 너비가 된 TextLine frame 대신 실제 자식 TextRun 점유폭 비교. positive66.08 > no-gap62.08 > negative61.28 관계를 독립 PDF와 대조 |
| #4438 passthrough3건 | 3×2라 선언하고 Cell이0개인 수동 입력을6개 실제 빈 셀로 바로잡음. 원본104byte CTRL_DATA/비변경 보존은 유지 |
| #2724·#6520·#4969 각1건 | SessionState journal-only 분류, 실제 minor29, 명시적 알려진 `paint.textClipPath` wire 이름 반영. 실제 mutator 무효화·control7종·원래 글꼴 payload 예산은 유지 |
| #7998 포함 그림·빈 지역·숨은 경계 각1건 | 원본 그림/문단의 단일 셀 소유와 실제 행열 편집을 검사. 포함 그림은 실제1표, 교차 그림은0표, 병합 빈 행5셀과 분리 빈 셀4셀. 열린 외곽/clip 반례는 여전히 거절 |
| #7911 font pool1건 | 실제 run별 선언 face와 글꼴 프로그램 정체성을 확인. family 이름만으로 다른 프로그램을 제품에서 합치지 않음 |
| #7911 선택 페이지1건 | 연산 상한을 줄였을 때 참조 탐색 상한까지 함께 줄던 계상을 분리. 선택5연산 통과·실제참조>8, 명시 참조상한8 거절·미선택65연산 거절을 각각 확인. 기본 자원 상한 유지 |
| #6708 기준선1건 | 위66쪽 보정과 원본 저장 줄/독립 Print 관계를 먼저 검증한 뒤 잘못된 수동 TAG 가정과98% 미달 페이지 pin을 수정 |

실제 제품 보정은 `f8bf2a853`, 테스트/입력 계약 보정은 `8f9badd87`·`73de01f03`이다.
23입력37쪽/backend의 새 [전체 TSV·입력 SHA](../pr/assets/issue_7998/submission-20261021/qualification/README.md)는
Native 최저97.90355%·fresh WASM97.75192%다. 추가 tac66쪽은 별도 폴더에 보존했다.
집중11모듈과#4438 직접4건이 통과했으며, old failure·명령·새 결과는
`pr-registration-20261021/fix17/focused-results.json`과 해당 로그에 연결된다.

최종 release lib는892.96초에 실패0으로 완료됐다. 다음 integration 실행은 최종 테스트
head `168e96b86`에서 시작했으며 뒤의 필수 검증은 같은 공유 target에서 순차 실행한다.

최종 전체 integration은 **11,209 PASS/실패0/기존45skip**로 완료됐다. 전체959.58초 중
실제 nextest 실행512.503초이며 최종 테스트 head `168e96b86`다. 앞선17실패를 모두
해소했고 새 쪽번호/저장 선행 글줄 회귀2건과 기존#6708 다섯 관계 검사도 포함한다.
Native Skia3종·lint·workspace·doc·FFI/Qt는 이어서 순차 검증한다.

Native Skia lib는4,041 PASS/실패0/기존12 ignored, 공식 그림 placeholder·직접 PDF 검사도
각각 통과했다. 첫 Native Clippy는 두 lint로 실패했다. `compound.rs`의 attribute 뒤 빈 줄을
제거하고 `table_paint.rs`의 `!image && orientation.is_some()` + `unwrap`을
`if let (false, Some(h)) = (image, orientation)`으로 정리한 commit은 `c448e70a4`다.
`Option<bool>`은 Copy이며 image=true의 세 방향값/false+None에서는 양쪽 모두 비실행,
false+Some(false/true)에서는 같은 h로 같은 본문을 실행한다. 여섯 조합의 조건·body·순서·
좌표·자원 tick·소유 계상은 그대로다. 나머지 변경은 attribute 뒤 빈 줄 제거뿐이다.

이는 PDF table paint의 동작 불변 lint 정리이며 전체 회귀·Native 실행·Docker 출력의 기존
통과를 재사용한다. HWP/HWPX로 저장된 글꼴 예외 입력은 이 PDF import 분기를 소비하지
않는다. 새 lint head와 앞선 실제 전체 검사 head를 구분하고, Clippy부터 남은 게이트만
재개했다. 실패 로그/첫 gate 결과는 `clippy-native-before.log`·`gate-results-before-lint.json`에
보존한다. JSON의 source SHA를 완화하지 않고 정식 글꼴 예외 도구도 이 lint head에 고정했다.

Workspace 빌드 통과 후 all-target Clippy는 추가 lint2건으로 실패했다. `979188617`에서
`BinData.content_index` 함수 본문을 바꾸지 않고 cfg(test) module 앞으로 옮겼다. 이동 전후
본문 hash는 `three-page-final/bin-data-lint-body.json`에 보존한다. 고정110,000개의 collection
검사는 `used >= 110001`과 같은 `used > 110000`으로 표현했다. 기대값·강도·자원 정책·
제품 로직은 변하지 않는다. 새 formatter와 workspace all-target Clippy가 통과했으며 앞선
전체 검사/불변 동작 증거를 재사용했다. 실패 로그는 `clippy-workspace-before.log`에 남겼다.
정식 글꼴 예외 도구는 이 최종 lint head의 JSON/해시에서 다시6건 모두 통과했다.

### 필수 로컬 게이트 최종 결과

동작 불변 lint 정리 이후 새 동작 검사를 확장하지 않았다. 아래 실행 SHA를 구분하며
전체 실패 로그·원본 명령은 ignored `three-page-final/gates/`에 보존한다.

| 게이트 | 실제 실행 head | exit | 초 |
|---|---|---:|---:|
| release-build | `73de01f03` | 0 | 1194.46 |
| release-lib | `73de01f03` | 0 | 892.96 |
| integration | `168e96b86` | 0 | 959.58 |
| native-lib | `168e96b86` | 0 | 238.58 |
| native-placeholder | `168e96b86` | 0 | 197.32 |
| native-pdf | `168e96b86` | 0 | 10.93 |
| clippy-native | `c448e70a4` | 0 | 77.11 |
| clippy-wasm | `c448e70a4` | 0 | 87.09 |
| workspace-build | `c448e70a4` | 0 | 131.24 |
| clippy-workspace | `979188617` | 0 | 95.90 |
| clippy-root-targets | `979188617` | 0 | 117.93 |
| doc-tests | `979188617` | 0 | 119.72 |
| native-ffi-build | `979188617` | 0 | 177.15 |
| native-qt-smoke | `979188617` | 0 | 2.23 |

formatter도 최종979188617에서 통과했다. 실제 doc/Qt 검사 로그는 같은 폴더에 있으며,
정상 종료와 Qt screenshot 생성까지 확인했다. 전체 결과를 PR 준비 완료 근거로 사용한다.

Qt smoke는 offscreen 실제 Native render·synthetic Korean IME·clipboard·multi-page·caret·
save/reopen을 확인했다. 실제 OS IME는 `osIMEVerified=false`이며 이 검사로 검증했다고
표시하지 않는다. 문서 검사는8 PASS/실패0/기존3 ignored였다.

### PR #8120 Render Diff 선행 계약 검사 보정

등록 head `81de976032`의 [Render Diff 실행](https://github.com/edwardkim/rhwp/actions/runs/39547000125)은
`Check render diff script syntax`의 `npm run e2e:renderer-contract`에서 실패했다.
실제 그림/PDF diff와 90% 판정은 시작되지 않았으며 시각 회귀나 글꼴 오류로 분류하지 않는다.
일반 `npm test`와 별도로 실행되는 이 E2E 진입점을 로컬 제출 검사에서 누락했다.

기존 guard `ee67eef3635`는 모든 LayerNode 종류를 명시적으로 나열하도록 했다.
이번 #7998의 `90b4ffbe93`에서 편집 가능한 비사각 텍스트 가림용 `clipPath`를 추가했으나
이 목록에는 기존 `clipRect/group/leaf`만 남았다. CI와 같은 assertion을 수정 전 로컬에서도
재현했다. 새 유형을 명시적으로 등록하고 두 clip 종류 모두 실제 dispatcher를 거치는지
확인한다. 실제 CanvasKit renderer를 Vite SSR로 불러와 경로 교집합·nonzero/evenodd 규칙,
진단용 clip 비활성화 상태에서도 원본 텍스트 clip 유지, 자식 실패 시 canvas 복원을 검사했다.
제품 코드·그림 기대값·임계값·workflow는 변경하지 않았다.

수정 후 같은 계약 검사가 통과했다. `.github/workflows/render-diff.yml`의 해당 run 블록을
그대로 추출해 Node 구문·Native diff self-test·Python 구문·renderer contract·font coverage
12개 명령을 실행했고 모두 성공했다(2.248초). 수정 전/후 로그와 실제 명령은 ignored
`pr-registration-20261021/render-diff-ci/`에 보존한다. 실행 환경은 로컬 Node24.15.0,
실패 CI는 Node22.23.3이며 새 head CI에서 별도로 확인한다.

기존 제품/WASM·전쪽 시각 증거는 제품 무변경으로 재사용한다. 이 선행 검사 통과를 실제
Render Diff 통과로 표시하지 않으며, 후속 push의 정확한 head 결과를 확인한다.
