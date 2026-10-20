# #7998 stage33 — 들여쓰기·목록 본문의 공통 줄 원점

2026-10-20 다음 작업 진행 승인을 받아 stage32의 미검증 원점 경계를 처리한다.
기준은 `bf6ccedf11d1235c74cf3470ff319f46a63ac9af`다. 기존 본문 글줄의 수용·전쪽
증거는 보존하며 제품이 달라지는 경계만 새로 검증한다.

## 독립 근거와 수정 전 추적

공개 `pdf/issue7998/body-indent.pdf`는 첫 줄이 이어지는 줄보다 12pt 오른쪽이며,
`body-list.pdf`는 첫 줄 뒤의 이어지는 줄이 15pt 오른쪽이다. PDF text-show의 좌표와
stage32 실제 glyph·문단 속성·LineSeg를 보존했다. 각각 ParaShape indent=2400/-3000
(문단 속성의 1/200pt 단위), source 줄의 물리 시작=3200/2000 및 2000/3500HU다.
이 두 줄은 강제 개행으로 확정된 기록이 아니며 한 문단과 정상 편집 흐름을 유지한다.

값 추적: `text_analysis::paragraph_layout`의 첫 줄/이어지는 줄 차이 →
`structured_lowering::paragraph`의 표준 여백·indent 및 source 물리 `column_start` →
`wrap_width::Trial`의 `ParagraphBox`·`reflow_line_segs` → 새 줄의 공통 왼쪽 여백과
`TAG_INDENTATION` → paint의 `paragraph_line_indent_for_source` → 최종 glyph 원점이다.
현재 검증은 source의 물리 시작과 fresh 줄의 기본 상자 시작을 같은 값으로 비교한다.
또 source 줄의 비트 부재를 실제 한컴 저장 줄의 들여쓰기 미적용 증언으로 해석한다.
정상 한컴 저장 기록의 수용 규칙을 완화하지 않고 PDF producer의 증거와 구분한다.

수정은 PDF proposal의 관측 원점으로 검증하는 작은 왕복 범위다. 실제 한컴 저장 줄은
기존대로 존중한다. 임의 좌표 clamp·indent 삭제·하드 개행·그림 대체는 하지 않는다.
같은 공통 들여쓰기 규칙을 소비한 실제 원점과 source를 대조하고, source 컷·기준선·
최종 점유/뒤 문단이 일치한 경우에만 fresh 결과를 발행한다. 실패한 입력은 원문을 유지한다.

수정 전 CLI·rlib를 `/mnt/e/tmp/issue7998-stage33-evidence/before/`에 동결했다.
진단은 ignored `output/pr-review/issue7998/p4-paragraph-indentation/`에 보존한다.
시각 선행 조건 전에 새 정식 렌더링 회귀를 추가하지 않는다. 긴 전체 검증·원격 작업은
이번 절편 범위가 아니며 R1/B24·나머지 R2·P5와 `parser_allocation_bounds_proven=false`를 유지한다.

## 구현·집중 검증

`wrap_width.rs`만 변경했다. PDF proposal의 trial clone에 기존
`composer::restamp_indentation`을 적용하고, fresh 줄의 `column_start`와 paint가 쓰는
`paragraph_line_indent_for_source`의 합을 source 물리 시작과 대조한다. 원래 문단은
trial 성공 전까지 변경하지 않는다. baseline·source UTF-16 cut·모든 row 검증을 통과한
경우에만 그 공통 reflow 결과를 발행하며 stage32의 `paragraph_flow_end` 소비를 유지한다.
정상 한컴 파서·공통 재조판·paint·serializer의 규칙은 바꾸지 않았다.

들여쓰기의 source 시작3200/2000HU는 fresh 기본 시작2000/2000+첫 줄 indent1200HU,
목록의 source2000/3500HU는 fresh2000/2000+이어지는 줄 indent1500HU다. source 양 줄의
일치 폭 구간은 각각7372–11427HU/10112–12663HU이며 작성 당시 폭은 unknown이다.
source Paragraph/TextBox 소유와 원문을 유지했고, 실제16문서의 표·셀·문단 그룹과 전체
쪽수도 기준과 같다(`semantic-owner-comparison.json`).

정식 등록 전 임시 `indent_contract.rs`의 **같은 함수 전체**를 frozen 기준 rlib와 현재
rlib에 각각 링크했다. 기준은 직접 삽입·paint·삭제 뒤 두 줄이 한 줄로 바뀌는 의도한
원인으로 FAIL, 현재는 PASS다(`indent-before.log`, `indent-after.log`,
`indent-contract-before-after.json`). PDF/HWP/HWPX 6경로에서 literal source row/문단
기대값·12pt/-15pt 상대 원점·첫 줄/뒤 문단 기준선·최종 glyph 순서/단일 보존과 양 저장
재열기를 대조한다. snapshot 복원으로 직접 삭제를 대신하지 않는다.

기존 #7998 집중158건·문단 여백12건 PASS다. 정상 한컴 저장 플래그와 편집 들여쓰기
대조는21건 PASS 뒤 한 건이 상대 경로 오류로 중단됐다. 샘플은 실제로 존재했으며
임시 실행의 cwd가 Studio여서 발생한 **환경 실패**다. 해당 한 건만 worktree root에서
재실행하여 PASS를 확인했다. 22건 전체를 다시 실행하지 않았으며, 이 환경 실패를
수정 전 결함 검출 증거로 계산하지 않는다. 따라서 현재 제품의 기존 검사 합계는
158+12+22=192 PASS이며 신규 후보1건 PASS는 정식 등록 대기다.

## 독립 Print·빌드·전쪽 시각 검증

Native `CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test --features
native-skia --lib --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`를
2분58초에 완료했다. fresh Docker WASM은 같은 고정 target의 wasm 경로를 재사용한
`docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=14
-e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`으로7분56초·exit0이다.
소스·frozen Native·WASM·glue SHA와 public 일치는 `source-runtime.json`에 고정했다.

수정 전 진단 HWPX 인쇄의 첫 연결은 job ID를 받기 전에 `fetch failed`로 종료됐다.
attempt를 보존하고 중복 제출하지 않았다. 이후 최종 새 저장본12개는 모두 정상 처리돼
별도 수동 Print를 요청하지 않았다. 독립 한컴2020 Print8개 작업+입력 바이트가 같은
4개 alias로12개를 검증한다. 모든 작업의 engine2020·11.0.0.9136·method0·one-up을
status.result에서 확인했다(`final-print/remote-print`, `print-aliases.json`).

Native 전쪽은70입력81쪽, 최저 **91.61577%**, 미달/누락/쪽수 차이0이다. 원본37쪽+
기존 수용 저장본32쪽+새 동일 편집 저장본12쪽이다. 원본 들여쓰기·목록은 각각100%이며
직전 수용 범위69쪽은 모두 PNG가 같다(`native-previous-comparison.json`). 이전에
잘못 생성된 직접 역편집의 실제 Native PNG는 `before-inverse/`에 별도로 보존한다.

새 실제 Studio WASM은 PDF/HWP/HWPX6경로의 삽입 표시·직접 삭제·Snapshot undo/redo·
양 저장을 통과했다. PDF 경로의 편집 저장 bytes는 Native와 같다(`studio-result.json`).
fresh WASM 전쪽은 같은70입력81쪽, 최저 **90.85213%**, 미달/누락/쪽수 차이0이다.
원본 들여쓰기·목록은 각각100%다. 이전69쪽 중68쪽의 PNG가 같으며 보고서10쪽의 글자
하단 한 픽셀 행(x140–240, y924)에 차이가 있다. raw는98.35918%→98.35573%로
**0.00345%p 하락**이다. 독립 PDF·이전 WASM·현재 WASM·현재 Native의 같은 영역을
확대했고 큰 원점 이동·줄바꿈·누락은 관측되지 않았다. 작은 RGB/경계 변화도 비교 화면에
남겼다(`wasm-previous-comparison.json`, `wasm-source-edge-comparison.png`).

`E:\tmp\issue7998-paragraph-indent-review\index.html`에 전체140개 TSV 원문·대표
compare/overlay·같은 영역 확대·실제 PDF/HWP/HWPX·독립 Print·수정 전 역편집을
준비했다. 한컴의 Haansoft Batang 대체와 PDF Helvetica의 모양 차이도 확대에 명시했다.
Studio7793의 `?review=issue7998-paragraph-indent-source`, `...-saved`, `...-list-source`,
`...-list-restored` 새 네 탭에서 실제 fresh WASM SHA의 로드와 파일 열림을 확인했다
(`human-review-tabs.json`). 기존 탭과7700 서버는 유지했다. 작업지시자는 위 미세 변화와
대체 글꼴 차이를 포함한 이번 범위를 **“시각·편집 구조 통과”**로 판정했다.

## 정식 회귀 등록·절편 마감

사용자 판정 뒤 `tests/cases/issue_7998_pdf_body_indentation.rs`를 정식 등록했다.
임시 후보와 달라진 것은 결과보고를 가리키는 두 문서 주석뿐이며 helper·cfg·검사 함수는
수정 전 FAIL/수정 후 PASS로 실행한 소스와 같다. 실행 코드 SHA와 등록 파일 SHA를
구분해 `indent-contract-before-after.json`/`source-runtime-registered.json`에 고정했다.
제품 source도 빌드·전쪽 캡처 시점과 같아 동일 빌드·Print·검사를 반복하지 않았다.
기존192건+새1건=**193 PASS**이며193건을 묶어 다시 실행한 것으로 바꾸지 않는다.

기존 입력을 참조하는 정식 함수도 확인했다. `issue_7998_pdf_body_paragraphs.rs`는 이미
문단 소유·원문·최종 paint의 순서/포함과 원래 indent 부호를 검사하며 절대 px 핀이 없다.
교체할 과거 좌표 핀은 없었고 새 함수가 실제 역편집·저장 원점과 내용 보존을 보완한다.
fixture·baseline·기존 기대값을 변경하지 않았다. 원본 속성에 근거한12pt/-15pt 상대 관계를
검사하고 문서 전체 SVG 해시나 임의 영역을 고정하지 않는다.

자동 수집 `node scripts/rust-test-suite-manifest.mjs --prepare`/`--check`를 통과했다.
새 모듈은 `regression_suite_009`, 1566sources·6973static attrs·28suites+20exceptions,
48integration targets다. CI 개별 호출을 추가하지 않았다. `rustfmt --check`와
`git diff --check`를 통과했고140개 TSV 복사본의 바이트·비교 화면 링크도 확인했다.

이번 들여쓰기·목록 본문 절편을 로컬에서 마감한다. 나머지 B10–B13 장평/자간 표현 경계와
R1/B24·P5·기존 클립 호환 범위는 유지한다. 전체 회귀·전체 lint·Worker/package·원격
push·PR·다음 절편은 실행하지 않았으며 `parser_allocation_bounds_proven=false`를 유지한다.
