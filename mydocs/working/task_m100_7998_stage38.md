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
