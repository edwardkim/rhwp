---
kind: evidence
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-13
---

# PR #7825 체리픽·메인터너 보정 검토 계획

## 2026-10-15 최종 통합 검토 연결

최종 적용 범위와 판정은 [개별 review](pr_7825_review.md)와 [통합 review](planet6897_followup_20261015_review.md)를 따른다. 검증 code SHA는 `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 최초 전체 실행·사용자 승인한 집중 재검증 합산, 전21문서181쪽 TSV와 최종 로컬 게이트를 구분해 기록했다. 아래 단계별 “현재”·미완료·C20 기록은 각 작성 당시의 이력이며 최종 상태를 대신하지 않는다. 원격 통합 PR의 CI·merge는 실제 확인 전 완료로 표시하지 않는다.


## 변경 전 C20 통합 검증 — 2026-10-13

변경 전 C20 검증 source HEAD는 `e2a525d36f2923918185f62610869e7c3068b9d0`, 검증 base는
`552f5228d2dbfd6046c838f14241704f1bcb8a05`다. 오늘 `upstream/devel` fetch와 rebase가 모두 성공했으며
SHA 변경·충돌은 없었다. 아래 과거 단계의 SHA·관측값은 당시 기록으로 유지한다.

이 C20 source에서 Rust 전체 회귀 **10,241건 통과·45건 제외**, Studio **1,836건 통과·실패 0건**,
Native Skia library/workspace **4,046건 통과·12건 ignored**를 확인했다.
Studio typecheck/build, Native·WASM·workspace all-target Clippy와 두 정책의 base 비교,
최적화 fresh WASM 및 Native 빌드도 exit 0이다. 모든 Cargo 빌드는 공용
`/home/tsjang/rhwp/target/pr-review`를 사용했다. 실행 명령·시간·SHA·exit code는
ignored `output/pr-review/planet6897-followup-20261011/c20-validation-sequence-results.json`,
상세 lint 결과는 `c20-lint-results.json`에 보존한다.

9개 PR의 전체 Native/fresh WASM 시각 검증은 **진행 중**이다. 이 테스트 통과를
시각 수용·PR 승인으로 판정하지 않는다. 문서별 실제 Print 세대, 전체 TSV, 대표 review와
직접 확대 판독을 대조한 뒤 이 PR의 적용 범위를 최종 판정한다. 현재 head의 결과는
`c20-native-whole-corpus-results.json`과 `c20-wasm-whole-corpus-results.json`에 기록한다.
완료되지 않은 결과 파일의 부재·일부 레코드는 통과가 아니다. 원격 통합 PR·CI·merge와
후속 처리는 아직 진행하지 않았다.


### C21 Canvas 점 페인트 보정 후보

현재 소스에는 C20 이후의 Canvas 가운뎃점 보정이 있다. 일반/효과 페인트가
SVG·Skia와 같은 전진 상자 중앙 및 점 기하 상수를 소비한다. 근거·실제 호출
차이는 [#7815 구현 검토](pr_7815_review_impl.md)의 최신 문장부호 항목을 따른다.
C20 Native 주요24문서·325쪽은 완료했지만 미달 문서가 남았다. C20 전체 WASM과
뒤 추가 corpus는 새 소스와 섞이지 않도록 중단됐다. C20 완료 증거는 그대로 보존한다.

보정 후보의 prepare·fmt·Native/WASM/all-target Clippy·workspace build와
고정 base 대비 manifest/unit-tier **8단계 preflight는 모두exit0**다.
새 code commit의 정식 회귀·fresh WASM 실제 재생·관련 전쪽 시각 검증은 아직
필요하며, 이전 C20 결과를 보정 후보의 최종 통과로 옮겨 적지 않는다.
원격 PR·CI·merge와 후속 처리는 미실행이다. 상세 후보와 명령은 ignored
`c21-canvas-dot-source-candidate.json`, `c21-canvas-lint-preflight-results.json`,
`c21-c20-source-supersession.json`에 보존한다.


## 상태와 검토 범위

검토 진행 중이며 수용·승인 판정은 아직 하지 않았다. 범위는 **문단의 글자 단위 여백**다.

- branch: `review/planet6897-followup-20261011`
- 초기 검토 base(과거 단계): `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`
- 원 PR head: `b368487aa2bac4234c1dd1755b1a768485d65212`
- 기본 경로: `collaborator_external_pr` 9.1.1, 주 작업공간 체리픽 통합
- 보조 경로: intake, local validation, visual fixture, multi PR, rework

| 원 고유 commit | 적용 commit |
| --- | --- |
| `d3cc21e2f1927fdc32cf3e8d4ccea5541c10ff5c` | `542b7622f5f5dc51d6dd2f9c5585d1e513566f6a` |
| `b368487aa2bac4234c1dd1755b1a768485d65212` | `2d6bb006abd7f35f3f3350fca5049840a879c9cb` |

원 fork history를 보존하고 중복 선행 commit·merge commit은 다시 적용하지 않았다.

## 구현 주장과 확인할 경로

stored unit decode·기준 스타일 크기 → composer/layout의 유효 여백·indent → 실제 줄 구성과 문단 원점을 확인한다. 측정 뒤 별도 원점 덮어쓰기도 대조한다.

## 독립 기대값과 검증 계획

합성 9개와 같은 입력 Print 전체를 비교한다. 들여쓰기 경계·정렬·base char style·위아래 간격을 확인하고 절대 원점 pins는 시각 통과 뒤 독립 관계형 계약으로 보정한다.

최종 통합 source의 해당 lint·회귀·Native/fresh WASM 전쪽 TSV·대표 review 확대 판독이
완료되기 전에는 통과로 판정하지 않는다. 기여자 PNG·기여자 CI는 최종 통합 head의 증적을
대신하지 않는다. raw TSV·중간 JSON·로그는 ignored
`output/pr-review/planet6897-followup-20261011/`에 보존한다.

입력/Print SHA, 실제 source SHA·명령·관측값과 `충족 / 미충족 / 미검증 / 비해당`
판정은 실행 결과로 이 기록을 보완한다. 필요한 오늘할일은 최종 검토 묶음과 함께 통합 PR에
포함하며 별도 기록 PR을 만들지 않는다.

## 합성 줄 말미 공백 정렬 보정 근거

통합 source `8e56e22c1b82a420f57958566738d4b6cc37dfcf`의 9개 합성 입력 전쪽 대조에서
여러 입력이 90% 미만이었다. 실제 바탕 글꼴 공급과 Full 임베딩을 확인한 뒤에도 미달이 남았다.
`base_charpr0_20pt`의 PDF subset과 실제 Batang.ttf의 한글 11개 글리프는 윤곽·폭이 동일하다.
첫 긴 줄의 줄 나눔은 같지만 마지막 단어의 시작은 Native 652.688px, 독립 Print 658.843px였다.
이는 배치 고정용 좌표가 아니라 같은 영역의 수정 전 관측값이다.

`compute_line_extra_spacing`은 저장 줄에서 말미 구분 공백을 제외하면서 합성 줄에서는
포함했다. 합성 여부는 정렬 규칙을 달리할 근거가 아니다. 내부 공백 수와 가시 내용의 폭을
같은 결과로 계산하고 `emit_paragraph_line_runs`도 말미 공백의 배분 여분을 회수하게 보정한다.
구분 공백의 원문 소유와 자연 advance는 보존하며 내용을 삭제하거나 bbox를 clamp하지 않는다.
생산(가시 줄 폭·내부 공백 수) → 배분 extra_word_spacing → char_width_decision →
run bbox/후속 x 소비를 함께 확인한다. 실제 배치 고정 px 검사는 추가하지 않았다.

기존 `synthetic_wrap` 산술 테스트는 이 가정을 고정하므로 시각 선행 조건을 충족한 뒤
독립 정렬 관계로 재검토한다. Native/fresh WASM 전쪽과 #4956 쪽 여백 편집 대조군을 검증하며,
진단값만으로 보정 완료 또는 원 PR 수용으로 판정하지 않는다.

## 말미 구분 공백 보정의 시각 증거와 관계형 검사

source `ab5112e29a2a23535525e4fe1dc96936defbe4e1`의 #7816 합성 9개 및 #7824 대조군은
Native/fresh WASM 전쪽 최저 99.83377% 이상이고 쪽수가 Print와 일치했다.
`base_charpr0_20pt`는 두 경로 모두 100%였다. 새 fresh WASM package의 wasm SHA-256은
`8e2fa96106bb6dc660fc8092766f3f2140d92472ba227334779f2ded88a8bc13`이다.
fresh WASM 대표 review와 같은 영역의 확대 PNG에서 마지막 단어·본문 오른쪽 끝·줄 나눔을
직접 대조했다. 가시 글자 위치의 기존 말미 공백 배분 차이는 사라졌고 글리프 획 raster 차이는
남는다. 전체 문서 시각 일치의 수치를 글리프 획의 완전 동일성으로 해석하지 않는다.

정식 원본 `tests/cases/issue_synthetic_wrap_trailing_separator_alignment.rs`는 첫 긴 문단의
소유 줄 3개와 말미 공백 소유를 확인하고, 앞 두 줄의 마지막 가시 한글 glyph advance 끝이
소유 줄 오른쪽 끝과 맞는 관계를 검사한다. 절대 원점 대신 독립 Batang 1em advance와 줄 상자를
사용한다. #7701까지의 source `f22cdb0359cd0c0c400e1b0348c7f8c63f53be0c`에서 정식 원본을
실행해 해당 정렬 관계로 FAIL을 확인했다. 주 작업공간의 production source를 강제 재컴파일한
라이브러리에서 같은 정식 원본이 PASS했다(`trailing-separator-test-after-verified.log`).
이는 focused 검사이며 최종 통합 head의 전체 회귀·lint를 대신하지 않는다.

ignored 검토 output의 `integration-small-native-results-trailing-space.json`,
`integration-small-native-results-fresh-wasm.json`,
`integration-trailing-space-fresh-wasm-review/issue7816-base_charpr0_20pt/review/review_001.png`,
`integration-trailing-space-fresh-wasm-review/last-words-enlarged.png`와
`trailing-separator-test-before.log`에 연결한다. 전체 통합 검증의 남은 실패를 이 증거로
면제하지 않으며, #7816 기존 원점 pins의 관계형 전환도 별도로 완료해야 한다.

공유 target에서 중간 worktree를 빌드한 뒤 주 작업공간의 Cargo가 중간 라이브러리를
fresh로 오판했다. 최초 `trailing-separator-test-after.log`는 해당 중간 라이브러리에서
실행된 결과라 보정 후 판정에서 제외한다. `src/lib.rs`의 내용은 바꾸지 않고 mtime만
갱신해 재컴파일했다. 검증 라이브러리 SHA-256은
`cb86bf181478945ad6e50ab0fcdc2ec76a33ce8a4f36e544a1c350155091b0a8`이며,
`shared-cargo-worktree-stale-artifact.json`에 발견·교정 근거를 보존했다.

## 글자 단위 간격 보정 뒤 저장·재열기 회귀

전체 회귀의 #7114 실패 4건은 삭제·이관하지 않았다. 이전 통합과 #7701까지의 source에서
통과한 저장 보존 계약이다. #7815까지의 source도 56자 편집의 페이지별 내용이 같았지만,
통합 source에서는 로터리 조종기 행이 재열기 후 103→104쪽으로 이동했다.

원인은 기존 저장 프레임 재작성의 출처 계약이다. 빈 문단 2279의 `spacing_after=161`은
글자 단위이며 원본 저장 사다리에도 뒤 여백 400HU가 있다
(`41206−38886−1200−720=400`). 새 해석의 30.9px가 이 간격과 맞고 종전 26.7px는 짧다.
이 올바른 간격이, 재열기에서 중첩 호스트 마지막 유닛 3곳에 9.6px를 더하는 기존 차이를
페이지 경계에서 드러냈다. 간격 해석이나 검사의 기대값을 되돌리지 않았다.

`writeback_reflowed_table_frames`가 실제 `cell_units`의 `CellUnitLineAnchor`를 소비한다.
같은 빈 비인라인 표 호스트 줄에 여러 물리 유닛이 속하면 부모 LINE_SEG 하나는 그 내부 컷을
증언할 수 없다. 재작성 스냅숏에서 그 줄만 기존 `TAG_IMPLEMENTATION_PROPERTY`로 구분해,
reader가 이를 한컴의 독립 저장 프레임으로 다시 해석하지 않게 한다. 원문·줄 메트릭·다른
tag와 live IR은 보존한다. 무편집 저장은 진입하지 않고 단일 유닛·인라인 표·글자 줄은
기존 저장 조성 정보를 유지한다. 문서 ID·고정 좌표·페이지 수 분기는 없다.

생산은 `cell_units`의 줄 소유와 실제 pagination 컷 → 저장 스냅숏의 줄 tag이며, 소비는
HWP serializer/parser → 기존 합성 줄 판별 → 셀 유닛·측정·배치이다. 빈 호스트의 내부 컷은
뒤 본문 줄에 거짓 reset으로 쓰지 않는다. 실제 호출 경로의 같은 소유 결과를 사용한다.

후보 라이브러리의 기존 정식 #7114 원본 5건이 모두 PASS했다
(`giant-frame-provenance-formal-tests.log`, 41.89s). 1·55·56·61·200자, 무편집 저장,
독립 Print의 첫 쪽 끝/둘째 쪽 시작을 포함한다. fmt·Native Clippy·base 고정 manifest/tier도
통과했다. source 파일 hash와 라이브러리는 `giant-frame-provenance-candidate/provenance.json`에
고정했다. 이 판정은 저장 보존 회귀의 focused 결과이며 giant 문서의 기존 전쪽 시각 미달
해결이나 #7813 종료를 뜻하지 않는다. 최종 통합 source의 전체 검증은 별도로 남는다.


## 통합 검증 후 테스트 lint 보정 — 2026-10-12

`issue_synthetic_wrap_trailing_separator_alignment`의 마지막 가시 glyph 조회를
`Iterator::last` 대신 역방향 `rfind`로 작성했다. 소유 줄·내용·정렬 기대값과 허용 관계는
그대로이며, 동일 production `467bfd1d8`의 frozen library로 실제 회귀 1건을 재실행해
통과했다. 새 `issue_reflow_empty_float_host_line` 2건의 포맷도 정식 suite 준비 뒤 보정했다.

현재 production의 작은 검증 문서 17개 Native 전쪽 TSV는 모두 측정됐고, 쪽수 불일치나
90% 미만 페이지가 없다. 최저값은 curly quote 98.50387%다. fresh WASM의 같은 17개
검증은 진행 중이며, 미주 편집본과 교육과정 전체 검증의 남은 실패를 통과로 세지 않는다.
절대 px 기반인 기존 #7816 검사의 관계형 재작성도 이 결과와 직접 판독을 연결해 계속한다.

## 고정 px 기대값을 관계형 검사로 전환 — 2026-10-12

production `467bfd1d87a8801632a253c595065d58edb63138`의 동결 Native CLI와 같은 source에서
새로 빌드한 web WASM으로 #7816 합성9개 전쪽 TSV를 재산출했다. 관련9개와 #7824 대조군 모두
최저99.83377% 이상이며 쪽수 불일치·누락이 없다. indent room 경계와 바탕글20pt의 소유 문단
영역을 Print/Native/fresh WASM 같은 배율로 확대해 줄나눔·내용 순서·간격을 직접 확인했다.
원본 TSV는 ignored output의 `current-small-native-results-467.json`,
`current-small-wasm-results-467.json`과 `current-small-owned-region-review/`에 연결한다.

이 선행 증거 뒤 `tests/cases/issue_7816_char_unit_paragraph_margins.rs`의 절대 px 원점 표를
제거했다. 같은 문구의 기준 문단과 변형 문단을 대조해 CHAR/HU 비율, 바탕글 크기의2배 관계,
양쪽/왼쪽·가운데/오른쪽 정렬 불변식, 과도한 들여쓰기 취소, 내어쓰기 후속 줄, 위/아래 간격의
귀속을 검사한다. 문단 전체 글자 순서·내용 복원과 대조하는 문단의 쪽 소속도 확인한다.
고정 px 영역으로 대상 글자를 선택하지 않으며 반올림 범위는 해당 글자의 실제 advance에 상대적이다.
SVG 기준선의 불필요한0.1px 반올림도 제거했다.

수정 전 `f22cdb0359cd0c0c400e1b0348c7f8c63f53be0c`의 검증된 동결 library에 최종 검사 본문을
연결해 실제 실행했다. 빌드는 성공했고 과도한 CHAR 들여쓰기 취소 assertion에서 FAIL했다.
현재 production에서 최종 정식 nextest는1건 실행·1PASS·202SKIP, exit0이다.
`char-margin-relational-before-final-build.log`, `char-margin-relational-before-final.log`,
`char-margin-relational-focused-3.log`를 보존했다.

최종 검사 파일 SHA-256은 `68d02504b9da845e2c8b606ebb39ad438667f74077b274e2c2150e84efab0d65`다.
초기 focused 실행은 포맷 뒤 source weight 변경으로 runner의 suite 배정과 생성 harness가 달라
0건 실행 오류였다. 이를 통과로 세지 않고 최종 포맷 뒤 `--prepare`로 재생성해 실제 실행을 확인했다.
파생 파일은 stage하지 않았다.

같은 고정 base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`를 다시 fetch했으며 변경이 없었다.
fmt/native Clippy/WASM lib Clippy/workspace build/workspace all-targets Clippy/manifest base 비교/
source unit tier base 비교는 모두 exit0이다. `relational-final-lint-results.json`과 단계별
`relational-final-*.log`에 command·base·검사 파일 SHA·종료 결과가 있다.
전체 회귀 재실행과 다른 원 PR의 미달 문서 수용은 이 focused 통과로 대신하지 않는다.

실제 로그는 ignored `output/pr-review/planet6897-followup-20261011/`의
`curly-host-line-all-targets-clippy-3.log`(전체 workspace/all-targets, exit 0),
`trailing-separator-clippy-fix-focused.log`(1/1),
`trailing-separator-clippy-fix-manifest-3.log`(고정 base `bfa28a77`, exit 0),
`current-small-native-results-467.json`이다. generated suite/manifest는 제출하지 않았다.

## 2026-10-14 원 PR head 변동 감사

GitHub에서 이 PR은 OPEN이며 최신 source head는 `9bbcd5935a19d3a4402cb89c8bef40eed423b1f3`다.
통합 브랜치의 기존 적용 범위에 새 체리픽을 추가하지 않았다. 원격 head에는 다른 PR의
merge와 후속 보정도 포함돼 있으므로 두 tree의 전체 차이를 이 PR 고유 변경으로 보지 않는다.
기존 통합 source의 테스트·시각 결과를 최신 원 PR head의 검증으로 보고하지 않는다.
읽기 전용 fetch ref와 commit 목록은 ignored
`output/pr-review/planet6897-followup-20261011/c37-source-pr-remote-head-drift.json`에 보존한다.
현재 통합 branch의 수용 판정·PR·CI·merge는 미완료다.
