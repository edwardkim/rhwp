# 이슈 #7642 — 최신 devel 통합 후 현재 브랜치 전수 검증

이전 단계: [최신 devel 통합](task_m100_7642_stage14.md).
브랜치: `fix/equation-font-metrics-pagination-20261007`.
정책 비교 base: `74f9b71b15a2db18b17c37ceab6cf2b3c20c03c3`.

## 범위와 순서

1. 현재 source와 파생 integration suite로 전체 nextest를 실행한다. 공유 target은
   `/home/tsjang/rhwp/target/pr-review`, release-test, threads 12, no-fail-fast를 사용한다.
   실행 head·명령·최종 exit·실패 이름과 원본 매핑은 ignored output에 기록한다.
2. 같은 source의 Native/fresh WASM 시각 검증과 독립 Print PDF·확대 판독을 확인한다.
   이전 source로 실행 중인 36개 Native 비교는 이전 코드의 진단 자료로 보존한다.
   그 결과를 현재 source의 시각 통과로 대신하지 않는다.
3. 실패한 원본의 검증 범위를 확인한다. 고정 base에서도 이미 90% 미만인 원본에 등록되어 있던
   회귀 검사가 부적합한 경우에만
   [새 이슈 이관](../manual/pr_review/visual_fixture_evidence.md#90-미만-회귀-대상의-별도-이슈-이관)에
   따라 별도 등록·검사/원본/Print PDF 이관 후 해당 이슈에서 보정한다. 이번 수정으로 발생한
   실제 회귀는 이관으로 해결했다고 보고하지 않는다.
4. 시각 선행 조건과 확대 판독을 충족한 대상은 고정 px 대신 소유·순서·보존·포함·겹침 관계로
   검사한다. 관련 검사 변경 후 전체 회귀와 필수 Rust lint·정책 base 비교를 완료한다.
5. fresh WASM·Studio 산출물과 대표 시각 증적을 확보한 후보로 새 PR을 생성하고,
   code CI 성공 뒤 review/오늘할일 trailing commit과 trailing CI를 확인한다.
   검증된 정확한 head를 merge한 뒤 정식 후속 처리를 완료한다.

로그·TSV·중간 JSON·nextrun은 `output/pr-review/issue7642/`에만 보존한다.
현재 PR 준비 완료·전체 회귀 통과·최종 시각 수용을 선언하지 않는다.

## 이관 조건의 명확화

추가 사용자 지시에 따라 이관 조건을 고정한 `upstream/devel`에서 이미 90% 미만인 원본에
등록되어 있던 회귀로 한정했다. 현재 source-frame 시각 자료는 이전 feature source이므로
현재 upstream base의 미달 증거가 아니다. 현재 수정으로 새로 생긴 회귀는 현재 branch에서
해결하며, base의 동일 입력 검증이 없으면 이관 대상 판정을 보류한다.
현재 nextest는 `1e75981cf` source의 실행을 유지한다. 이번 추가 변경은 지침·기록만이며
source/test 변경 없이 검증을 계속한다.

지침 단서 변경 검증: 수정한 13개 문서의 상대 링크 검사와 `git diff --check`가 exit 0이다.
Rust source/test 변경은 없다. 현재 source 검증의 포맷·manifest/단위-test 정책·Studio tsc는
각각 exit 0이며, Studio node 검사는 1815 PASS / 0 FAIL / 2 SKIP, 82.524초로 종료했다.
전체 nextest는 아직 실행 중이며 완료·통과로 기록하지 않는다.

## 전수 회귀 종료, 요청 52개 시각 검증과 output 정리

실행 source `1e75981cf`와 같은 Rust source의 head `e45d2461c`에서 전체 nextest가 종료했다.
결과는 **10450 PASS / 90 FAIL / 50 SKIP**, 검사 1122.891초, 최종 exit 100이다.
컴파일 오류는 없으며 전체 회귀 통과로 판정하지 않는다. 이름·원본 후보·실패 본문은 ignored
`stage15-current-validation/nextest.log`, `current-failures.json`, `nextest-provenance.json`에 있다.

추가 사용자 목록 52개를 모두 검증 범위에 포함했다. 모든 원본이 현재 Git 작업공간에 있고,
48개 기존 Print PDF의 원본·PDF 해시와 실제 Print 방법을 확인했다. 추가 4개
`exam-kor-1p` / `exam_social-p1`의 HWP와 HWPX는 지정 MCP engine2020에서 Print로 출력했고
모두 1쪽이다. 원본을 대체하지 않으며 새 PDF도 commit에 포함한다.

현재 Native는 binary SHA-256 `03f6268968ab1bb16e30efba8690cddaf67ce249dc2e4a643c8fdc1f6026dbd1`로
고정해 52개 전쪽 TSV를 순차 산출 중이다. `exam_math_8.hwp`는 현재 83.49525%이며 수식 조건의
가로 간격이 Print와 다르다. 90% 미달을 통과로 표시하지 않았다. 같은 래스터로 표준 review와
3배 확대 이미지를 생성해 사용자에게 직접 표시했다. 정상 대조 `exam-kor-1p.hwp`는 98.89162%다.
이 값은 현재 Native의 해당 입력·해당 페이지이며 52개 전체 또는 fresh WASM 통과가 아니다.

기준 `upstream/devel` (`74f9b71b1`)의 별도 sparse worktree를 준비했다. 최초 빌드는
manifest의 `vendor/svg2pdf`가 checkout 범위에서 빠져 exit101이었고, vendor를 추가한 뒤
같은 공유 target의 순차 Native 빌드를 재실행 중이다. source snapshot은 바꾸지 않았다.
42개 기존 대상의 원본이 base와 바이트 동일함을 확인했지만, base의 시각 결과는 아직 없어
기존 미달/현재 수정의 회귀/이관 가능 여부를 판정하지 않았다.

사용자의 output 정리 지시에 따라 이전 feature source 진단을 종료하고 현재 작업을 보존했다.
종료 진단 SVG 100개 9.43GiB와 이전 단계의 파생 SVG·래스터·임시 실행 파일·rlib·HTML
17454개 69.60GiB를 정리했다. 원본/Print PDF·대표 review/overlay/zoom/compare/diagnostic PNG·
로그/TSV/JSON/소스·중요 보정 전후 binary와 현재 stage15 전체를 보존했으며 공유 target은 건드리지
않았다. `output`은 사용자 확인 86GB에서 정리 후 8.3GB, 디스크 여유 91GB다.
삭제 파일 목록은 ignored `old-diagnostic-svg-cleanup.json`, `obsolete-output-cleanup.json`에 있다.
현재 Native 산출로 output 용량은 이후 다시 증가할 수 있다.

현재 52개 Native, base 대조와 fresh WASM·확대 판독·실패 보정·관계형 회귀·필수 lint가 남았다.
새 PR·CI·merge 준비 완료를 선언하지 않는다. 오늘할일/self review는 code CI 후 trailing 시점에 작성한다.
