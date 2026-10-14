# #7899 — 제출 검증과 시각 예외 확정

계획: [구현 계획](../plans/task_m100_7899_impl.md). 이전 결과: [단계 3](task_m100_7899_stage3.md). 제품 source는 `a5d56ea64d1b6ccb98ebc19e70a98b9709d1d982`, 기준 devel은 `c0d330737fabae77ca2bf1706cba8514d0e8f1df`다. **로컬 제출 검증을 마쳤고, 원격 CI·self-review trailing 기록·merge는 아직 수행 전이다.**

## 확정한 범위와 남은 차이

작업지시자는 `{` 기호 크기와 다중 행 수식의 약 2px 아래쪽 배치를 이번 PR의 예외로 수용하고 PR 등록·self-review를 승인했다. 추가 수식 AST·행 배치는 변경하지 않았다. 자동 90% gate 통과나 새로운 렌더링 golden을 주장하지 않는다. 다른 9개 저장 대조군의 전쪽 Print/TSV는 미실행이며 저장 계약·독립 한컴 열기 검증과 구분한다.

처음 전체 검사에서 직접 `parse_hwp`의 보전 상태 미기록으로 정상 문서 저장이 거부되는 실패를 발견했다. strict/lenient·password 공통 반환 경계에서 tail 투영을 기록하고, `DocumentCore`의 load fixup 이후 기록과 같은 helper를 소비하게 했다. 기존 raw 캐시 정책·기대값·baseline은 유지한다. 원래 실패 검사와 직접 파서 대조군을 포함한 순수 저장 검사를 통과시켰다.

## 실제 검증 결과

| 범위 | 결과 |
| --- | --- |
| 첫 전체 nextest | 10,294건 중 10,293 통과 / 1 실패 / 45 skipped. 실패 원문과 로그를 보존했다. |
| 직접 파서 보정 후 집중 검사 | 기존 실패 검사 1건 + 저장 계약 10건, 총 11건 통과. |
| 보정 후 전체 nextest | 10,295 통과 / 0 실패 / 45 skipped, 334.106s. |
| Native Skia lib | root 3,859 + default workspace member 182 = 4,041 통과 / 12 ignored. |
| Native Skia 추가 2종 | placeholder 2건, direct PDF 4건 통과. |
| 저장 계약 최종 검사 | 단일 문자 삽입 lint 보정 뒤 Native Skia 설정으로 10건 통과. 제품 source는 바뀌지 않아 전체 결과를 재사용했다. |
| Rust lint·정책 | fmt, Native/WASM32/workspace all-target Clippy, workspace build, 고정 base의 manifest·unit tier 통과. 새 검사에서 `insert_str`→`insert`만 보정하고 해당 lint를 다시 확인했다. |
| 최종 Docker WASM | 공용 target·locked wrapper, 8m08s 성공. pkg/public WASM hash는 `d4c9802ea2a502630952f2a3c1b4a24a3c92c1c6c18ce33fb95174e0101d4b43`로 같다. |
| fresh WASM 저장·재열기 | 원본→A 삽입→저장→재열기에서 5개 수식의 TextLine 소속·기하와 저장 파일 hash가 앞선 검증본과 같다. 저장 hash `e00edfc63c35d5753e1369bad72de5187a485e9a6d0e9406f5746cb7c7cebf52`. |
| 실제 CDP Studio | 새 탭에서 실제 최종 WASM 응답 hash를 확인했다. live/reopened 전체 render tree는 기존 직접 판정본과 같다. Browser 저장본의 BodyText 등 모든 stream도 같다. DocInfo의 캐럿 글자 위치만 16→41로 달라 전체 파일 hash 동일을 주장하지 않는다. |

입력은 [편집 저장본과 Print](../../samples/issue7899/README.md)의 커밋 파일이다. 실행했던 외부 경로의 입력과 SHA-256이 같으며 현재 branch의 blob 포함도 확인한다. `.env.local`은 `.gitignore`에 추가했으며 내용은 읽거나 커밋하지 않았다. private 글꼴, generated glue·suite, 실행 로그와 TSV는 Git에 포함하지 않는다.

## 같은 입력의 devel 대조

양쪽 producer를 각각 Native Skia·Docker로 준비하고 공용 target을 순차 사용했다. 원문/PDF·전체 1쪽·Print·compat 2022·96dpi·실제 공급 글꼴·실루엣 방법을 맞췄다. Decimal로 기준−후보를 계산했으며 같은 경로별 0.01%p 이상 하락은 0쪽이다.

| 경로 | devel 기준 | 제출 제품 source | 기준−후보 | 해석 |
| --- | --- | --- | --- | --- |
| Native Skia 직접 PNG | 64.91873% | 77.01188% | -12.09315%p | 개선, 자동 90% 미달은 명시적 수용 예외로 기록 |
| fresh WASM Canvas2D 직접 PNG | 65.33564% | 75.78392% | -10.44828%p | 두 source 모두 글리프 누락. 기록값 비교이며 정상 출력 판정에 사용하지 않음 |
| fresh WASM SVG 진단 | 67.33301% | 77.20043% | -9.86742%p | 진단 비교 개선. 직접 PNG gate의 대체 증거가 아님 |

최종 Native/Canvas2D PNG가 이전 source `97616c522`의 PNG와 바이트 단위로 같음을 확인해 대표 review·overlay를 재사용했다. renderer·paint source도 그 검증 source와 같다. PNG 내 source 표시와 실제 생성 시점은 바꾸지 않고 재사용 proof와 제출 제품 source를 분리한다. 양쪽 Canvas2D는 HyhwpEQ 적재 후에도 수식 문자를 네모로 표시한다. 이 기존 출력 경로 차이를 중괄호·세로 위치 수용으로 해소했다고 보고하지 않는다.

ignored 증거: 기본 작업공간 `output/pr-review/issue7899-20261014/pr-preparation/`. 전체/focused/lint/Native Skia 로그, candidate/base Docker 로그, `fresh-save-reopen-final.json`, `browser-final/`, `visual-reuse-proof.json`, 각 전쪽 TSV·manifest, `devel-comparison.json`을 보존한다. 기존 `devel-merge-116b/wasm-svg-diagnostic/`의 진단은 재사용 기록에 연결한다. 대표 PNG는 `mydocs/pr/assets/issue7899-save-reopen/`에 보존했다.

## 다음 순서

코드·검증 파일이 바뀌지 않으면 위 실행을 반복하지 않는다. 원격 push·Open PR 등록 뒤 정확한 제출 head의 CI 성공을 확인하고, 채번된 PR의 archive review·2026-10-15 오늘할일을 문서-only trailing commit으로 반영한다. trailing head CI와 별도 merge 승인 전에는 issue를 닫거나 병합하지 않는다.
