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
