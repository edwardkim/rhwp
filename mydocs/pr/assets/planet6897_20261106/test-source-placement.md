# 회귀 테스트 원본의 외부 이동

## 원인과 변경 범위

`src/renderer/layout/paragraph_layout/issue_2439_lineseg_indent_tests.rs`는
`8e80671b63c9d1997a7c12633f08462d68e1feec`에서 내부 테스트로 추가되었다.
이동 직전 파일 변경은 `6844f6ec0854b1c8150244d65cc1a099031287ac`
(2026-11-06T20:11:29+09:00)이다.

사용자의 전체 이동 요청에 따라 기존 파일과 부모의 `mod` 선언을 제거하고 검사 5건을
`tests/cases/issue_2439_lineseg_indent.rs`로 옮긴다. 비공개 helper 2개를 테스트 전용
공개 API로 바꾸지 않는다. 기존의 직접 함수 검사 4건은 공개 `Document::layout_profile`과
`LayoutEngine::build_render_tree`가 만드는 최종 `TextLine` 원점으로 검증한다.
기존 렌더 트리 검사 1건의 글줄·글자 run 연결과 오른쪽 경계 검사도 유지한다.

| 기존 검사 | 이동 후 검증 |
| --- | --- |
| 같은 저장 시작점과 문단 여백 | 최종 제목 글줄이 문단 여백에 놓임 |
| 더 큰 저장 시작점 | 최종 번호 글줄이 저장 원점에 놓임 |
| 합성 줄·본문 이외 프레임 | 합성 태그와 별도 높이의 프레임이 본문 원점을 차용하지 않음 |
| HWP5 출처의 HWPX | 실제 `Document` 출처·마커를 통한 HWP5/HWPX 경계와 최종 위치 |
| 글줄과 글자 run | 번호·본문 run의 연결과 단 오른쪽 경계 |

같은 정책 점검에서 직전 글꼴 보정에 추가했던 내부 검사 2건도 발견했다. 이 검사는
`tests/cases/font_program_identity_contracts.rs`로 옮기고 공개 `FontEnvironment`·
`DocumentCore`의 소비 결과로 검증한다. 실제 가족 이름의 프로그램 신원과 원본 슬롯의
TrueType 요청 자격을 유지하며, 기존 OFL fixture를 명시적으로 공급한다.

이 변경은 합성 계약 검사의 위치·호출 경로 정리다. 제품 동작과 실물 문서의 배치를 바꾸지
않으며 HWP/HWPX·Print PDF·시각 측정 수치를 새로 만들거나 추정하지 않는다. 기존 시각
검증 결과를 이 테스트 이동의 출력 개선으로 해석하지 않는다.

## 검증

비교 base는 `08963f6825956984d38b0d9392f44777db7107ab`로 고정했다.
Cargo 검증은 전용 worktree의 최소 OFL 글꼴 환경에서 공용 `target/pr-review`,
`release-test`, `native-skia`, 빌드 잡 8개·테스트 스레드 12개로 실행한다.
suite 준비·파생 manifest 검사는 검토 worktree에서 수행하며 원본과 이 분석만 커밋한다.
파생 harness·manifest·정책 기준선을 커밋하거나 테스트 수 증가를 승인하지 않는다.

- 최종 집중 검사: #2439의 5건과 글꼴 계약 2건, 총 7건 모두 통과. 종료 코드 0.
- source-side unit-tier: 4,094개 검사·295개 모듈·지원 항목 28개로 통과. 기준선 변경 없음.
- 검토 worktree manifest: 2,125개 원본, 정적 검사 8,912개, 28 suites + 20 exceptions 통과.
- 이동한 원본의 `rustfmt --check`: 통과.

최종 집중 검사 뒤에는 #2439 첫머리의 주석 갱신 시각만 수정했다. 검사 본문과 제품 함수는
바꾸지 않았으며 이 주석 수정 때문에 전체 회귀나 문서 시각 출력을 다시 실행하지 않는다.

로그는 `output/pr-review/planet6897-integration-20261106/`의
`test-extraction-complete-focused.log`, `test-extraction-complete-unit-tier.log`,
`test-extraction-complete-manifest-check.log`, `test-extraction-complete-fmt.log`에 남긴다.
최초 외부 글꼴 검사의 대표 가족 기대값은 fixture의 typographic 별칭과 대표 이름을 혼동해
실패했다. `tests/fixtures/native_font_family_names/provenance.json`의
`primary_family`를 근거로 바로잡았으며 제품 구현을 바꾸거나 프로그램 신원 검사를 완화하지 않았다.
