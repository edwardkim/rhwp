# PR #9322 병합 이후의 리베이스와 기능 보존 검증

새 `upstream/devel`은 `93e861523623bcd84d57b808edb62af5af2aa82a`다. 이전 기준
`622af208ee12c4cb16357d465e4802e7fd4726bc` 위의 103개 커밋을 순서대로 리베이스했다.
리베이스 직후 후보는 `d322327c0f6d71c4343a5a2835dfaa6e4459a24c`이며, 새 기준과 후보의
병합 모의 트리는 후보와 같은 `47b9e2b83cc0c35ffe04d2df8d461e264905adda`다. 기존 렌더·테스트
프로세스가 사용하는 작업 디렉터리는 유지하고 별도 sparse worktree에서 충돌을 해결했다.

#9322는 #9280의 HWP 버전별 본문 기호, 단독 실로티 나그리 결합 부호, 누락 글리프의
표시와 전진폭을 저장·측정·Native·WASM·Studio 경로에 반영했다. `native_pua_fonts.rs`에서
같은 등록 지점에 추가된 두 분석을 합쳤다. 원래 face에서 가시 glyph 0의 전진폭과 가시
옛한글 호환 glyph의 전진폭을 각각 수집하고, compact face로 바뀐 뒤에는 각 소비자가
자신의 측정값을 쓴다. 버전별 기호의 표시·저장 처리와 기존 87개 기호의 공급 순서도 보존했다.

[원 PR 검증 자료](../pr/assets/pr9280/README.md)의 Native·WASM 37쪽씩 원본 TSV 74개와
37종의 입력·독립 한컴 Print PDF 해시를 새 base의 Git 객체에 대조했다. 원 PR 생산 소스
`9595602fe1024fc314bb0c9918a8f7343a91bb75`의 Rust·Studio·스크립트 입력은 새 base와
같았다. 이 기록은 #9322 검증 입력의 출처다. 새 후보의 실제 출력, HWP 버전 경계에서의
HWPX 저장·재열기, 결합 부호의 내용·편집 위치와 실제 화면·인쇄 backend는 새 base와 비교한다.

37종의 입력·PDF를 쪽수와 해시로 고정했다. 검증에 사용하는 글꼴 다섯 파일은 원 PR의
해시와 일치한다. 굴림은 현재 TTC의 첫 face를 독립 한컴 PDF 16개가 실제 사용한 GID 0과
대조해 4,539개 표시에 쓰인 윤곽과 전진폭을 확인했다.
[검증 입력 목록](../verification/issue7754-nine-fidelity/incoming-pr9322/inputs.tsv)과
[글꼴 프로그램 대조](../verification/issue7754-nine-fidelity/incoming-pr9322/font-supply.json)를 사용한다.

현대 HWPX 재저장 대조군에서 리베이스 전 두 생산본은 같은 여섯 문단의 BMP 문자를
Plane 15 기호로 바꿨고 굵은 구간의 UTF-16 시작 위치도 1만큼 밀렸다. 한컴 HWPX와 같은
원문과 굵은 구간을 새 생산본에서 확인하는 데 사용한다.
[재저장 대조군의 실제 차이](../verification/issue7754-nine-fidelity/incoming-pr9322/modern-hwpx-save-before-pr9322.json)

새 `devel` 생산본에서 원본·편집본 11종을 Native와 fresh WASM으로 각각 44개 저장 산출물로
재현했다. 각 경로의 HWPX 33개는 독립 한컴 저장본과 문단 텍스트·굵은 구간이 일치했다.
WASM에서 원본과 재열기의 실제 표시 60개와 결합 부호 편집·재열기 13개 상태를 확인했다.
후보 Native도 11종의 저장·재열기에서 같은 한컴 문단 텍스트와 굵은 구간을 보존했다.
[기준과 후보의 산출물 해시 및 비교 결과](../verification/issue7754-nine-fidelity/incoming-pr9322/roundtrip-verification.json)

기준 생산본의 Studio에서는 원본·저장본 10종을 기본 글꼴과 Windows 검증 글꼴로 열고,
Canvas2D·CanvasKit의 화면·인쇄 80조건을 실행했다. CanvasKit 40조건은 해당 렌더러에서
그리기를 마쳤다. 두 렌더러의 대표 인쇄 캡처에서 단독 결합 부호의 점선 원과 정상 결합을
확인했다. [실제 렌더러와 PNG 해시](../verification/issue7754-nine-fidelity/incoming-pr9322/studio/studio-verification.json)

리베이스 직전 기준의 보호 정본 46조건·4,046쪽은 `devel`과 후보의 PNG 해시가 일치했고,
하락과 누락은 0쪽이었다. [리베이스 전 전쪽 보호 결과](../verification/issue7754-nine-fidelity/protected/validation-before-pr9322.json)

리베이스 전 생산본 `b75ce19774a4b059b26c2d134d49d5f7ce4a93b2`의 최소 글꼴 전체 Rust 검사는
13,984개 통과, 실패 0개, 설정에 따라 제외한 45개로 끝났다.
[검사 명령과 종료 기록](../verification/issue7754-nine-fidelity/local-checks.json)

같은 생산본의 fresh WASM은 원래 아홉 이슈 583쪽에서 최저 98.11%, 보조 검증을 포함한
633쪽에서 최저 98.07%였다. [원본별 전쪽 TSV와 측정 기록](../verification/issue7754-nine-fidelity/wasm/candidate-before-pr9322.json)

Stage 20의 보호 검증과 원래 아홉 이슈의 증빙은 이전 기준과 생산 SHA에 연결한다. 새 base와
후보에서 동일한 Harness와 글꼴·연결 그림 공급으로 원래 대상, 보호 정본, #9318·#9322 기능을
다시 대조한다. 시각·최소 글꼴 Rust·Studio 필수 검증을 완료한 뒤 PR을 제출한다.
