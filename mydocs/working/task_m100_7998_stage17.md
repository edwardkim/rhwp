# #7998 외곽 테두리 잔여의 편집 소유

- 승인: stage16의 직접 시각·편집 구조 통과 후 다음 절편 진행.
- 시작 head: `e5d2fb0df`.

## 원인과 적용 경계

현재 외곽 stroke의 셀 밖 반쪽은 페이지 Rectangle로 보존된다. 같은 원본 path의 셀 안
조각만으로는 외곽 변경을 완결할 수 없어 setter가 전체 변 변경을 거절한다. source의
stroke 증거·축·물리 분할 보존으로 입증한 페이지 잔여를 논리 셀 변에 연결하되, 실제
페이지 좌표와 paint owner는 바꾸지 않는다. 내용 그림·흰 fill·복합 bitmap은 bbox만으로
삭제하지 않는다. 이번 절편은 단일 축 opaque stroke 외곽의 closure까지이며 fill/compound
일반화와 새 행열 계승은 다음 경계다.

입력은 기존 공개 occluded-inner-dot·thin-fill-occluder이며, 무편집 기대값은 원본 PDF다.
명시적 native 변 교체는 실제 편집 HWPX의 독립 한컴 Print로 대조한다. 페이지 잔여는
root Table 뒤의 안정된 슬롯에 두어 제거 시 Table index를 바꾸지 않는다. 같은 source의
모든 물리 조각이 인증됐을 때만 외곽 잔여에 편집 권한을 부여한다. 페이지 parent 저널은
Table 내용의 복제/덮어쓰기 없이 현재 Table을 보존하며 문단 슬롯과 잔여를 함께 복원한다.

수정 전 오류·원본 분할 좌표는 이전 증적의 outer-diagnosis.txt에 있다. 새 증적은 ignored
output/pr-review/issue7998/p4-outside-border/에 보존한다. 최종 출력·저장·편집 검증 전에는
완료를 선언하지 않는다. 전체 회귀/lint/package와 최신 devel 통합은 P5에 유지한다.

## 공통 소유와 실제 편집 소비

- table_paint::partition은 positive stroke path·단일 축·원래 중심선/셀 변의 일치와
  physical subtraction/conservation을 확인한다. 셀 안 조각이 모두 인증되거나 native
  border로 소비됐고, 남은 물리 조각도 중복 없는 root 셀 구간으로 모두 귀속될 때만
  outside owner를 발행한다. fill·복합 Picture·불확정 연장/모서리는 인증하지 않는다.
- structured_lowering은 실제 page-anchored Rectangle의 좌표·크기·z-order를 유지하고
  CommonObjAttr::outside_cell_border_paint에 논리 Table/셀/변/source를 연결한다.
  이 잔여만 모든 root Table/본문 frame 뒤에 놓아 삭제 시 표 인덱스를 보존한다.
  cell border 밖에서 clip을 확장하거나 잘못된 좌표를 clamp하지 않는다.
- border_paint는 기존 4MiB 한도에서 V1을 계속 읽고, 외곽 링크가 있는 문서는 V2를
  쓴다. 기본 record 65바이트에 외곽 record만 Table frame 지문 32바이트를 더한다.
  셀 치수/앵커·shape 좌표/스타일 지문과 유일 ID를 확인하고 양 adapter가 같은
  stream/entry에 보존한다. 실물 보고서는 43,892 record/2,856,992바이트다. 한컴의
  소유 편집까지 지원한다는 뜻은 아니며 기존 content-loss 보고를 유지한다.
- set_cell_properties_native의 preflight는 선택한 변의 실제 셀 및 페이지 조각을 함께
  계획한다. BorderFill/공유 이웃 갱신과 함께 실제 controls를 제거한다. paint에서
  숨기거나 배경으로 덮지 않는다. 페이지 parent 저널은 Table의 cells/zones 복제본을
  보관하지 않고 현재 live Table을 합쳐 슬롯/원문/줄 정보를 복구한다. 이미 앞에 놓인
  외곽 조각 때문에 Table 인덱스가 바뀔 입력은 변경 전에 거절한다.
- shape 좌표/표 앵커 불일치, 잘못된 undo 대상과 순서, 동시 셀 크기 변경은 preflight에서
  거절한다. 이 변경은 source paint와 스타일/슬롯 소유의 소비이며 줄 나눔·pagination
  컷·요구 높이를 바꾸지 않는다. 측정은 기존 셀/글줄 결과를 계속 소비한다.

초기 진단의 잘못된 slot 반례는 Top 조각을 이동하고 Left 편집의 거절을 요구했다.
선택하지 않은 다른 변의 변경을 거절할 이유는 없어, 실제 이동한 Top을 바꾸는 반례로
수정했다. 제품 보정 실패로 세지 않는다. 별도로 현재 undo 저널의 대상 문단을 삭제한
호출이 index panic을 내는 실제 경계를 재현했다. get 기반 전수 확인으로 바꾼 뒤 같은
호출은 RenderError를 반환하고 문서를 바꾸지 않는다. missing-target-before/after.log에
실행을 보존했다. 수정 중단한 이전 wasm-opt/Native/PNG는 pre-missing-target 이름으로
구분하고 최종 성공에 합산하지 않았다.

## 전쪽 출력·저장·실제 Studio

| 최종 제품 검증 | Native | fresh Docker WASM |
|---|---|---|
| 실물 두 원본 + 공개 네 대조군 | 6건/17쪽 | 6건/17쪽 |
| Native API 편집 × HWP/HWPX | 6건/6쪽 | 6건/6쪽 |
| 실제 WASM command 편집 × HWP/HWPX | 6건/6쪽 | 6건/6쪽 |
| 전체 raw 최저 | 97.57194% | 97.13390% |
| 편집본/실제 command 저장본의 Print 대비 최저 | 99.12726% | 99.32967% |
| 90% 미달·쪽 누락·쪽수 차이·실행 오류 | 0 | 0 |

validate.py의 실제 명령, results-source/edited/actual.json과 각 silhouette.tsv,
validation-summary.json에 연결한다. 양 backend의 전체 29쪽 TSV 원문은 이동 가능한
화면의 native-all-pages.tsv·wasm-all-pages.tsv에 보존했다. 90%만으로 직접 판독을
대신하지 않고 Print/Native/fresh WASM/실제 CanvasKit에서 왼쪽 선 교체·없애기·첫 셀
네 변과 자식 표/다른 셀·앞뒤 본문을 같은 영역에서 확인했다.

한컴 2020(11.0.0.9136)으로 HWP/HWPX **6파일 모두** 정상 열기·Print를 완료했다.
pdf_print_method=0, pdf_output_mode=hancom2020_pdf_driver_one_up을 각 status에
확인했다. print-sources.json·print-hwp-sources.json은 입력/PDF SHA·job·제품·Print
경로를 기록한다. undo 오류 반환만 보정한 뒤 입력 SHA가 모두 같음을 확인해 이 여섯
독립 Print PDF를 재사용했다. 최종 Native/WASM PNG는 새 제품으로 다시 산출했다.

Studio 7793의 실제 SetCellBorderFillCommand 세 경로에서 X 삽입/삭제 뒤 undo로
원래 HWPX 바이트를 복원했고 redo는 첫 편집 바이트와 같았다. 각 snapshot 슬롯은 0,
page error는 0이다. 양 저장 6파일의 실제 WASM 재열기·Native 재소비에서 cell/변/source/
외곽 owner·물리 지문이 Native API 저장본과 같았다(studio-result.json·
wasm-native-readback.log). user-cdp-tabs.json에 실제 사용자 Chrome의 새 세 탭과 새
WASM SHA·1쪽 표시를 기록했다. Studio 7700은 변경하지 않았다.

## 기존 출력·의미 구조의 유지와 직접 판정

이전 stage16과 비교 가능한 원본 16쪽은 **각 backend의 raw 점수가 모두 같다**.
다만 Native/WASM 각각 11쪽만 PNG 바이트 동일이며 다섯 쪽은 source를 셀 구간별로
나눈 선 접점의 소수 픽셀 농도가 다르다. source-png-differences.json과 비교 HTML 하단의
8배 확대에 원본/현재를 함께 보존했다. 무변경 PNG 전체 동일이라고 보고하지 않는다.
원래 Helvetica의 한컴 대체 글꼴·점선 농도 차이도 계속 명시한다.

실제 CLI로 두 원본을 stage16/현재에서 각각 재가져와 보고서 15개·견적서 1개의 의미 표,
중첩/보류 개수·source span과 본문 문단 연결을 대조했다. 새 물리 조각 때문에 발급된
instanceId만 달라지고 문단 내용의 source 연결/상태는 같다. 실제 두 HWPX의 재소비에서도 문단 원문·문단/글자 서식·Table/셀 기하의 재귀 signature가 이전 절편과 동일하다(semantic-actual-readback.json). 단순 픽셀 유지 때문에 표
승격이 positional fallback으로 사라진 결과가 아님을 source-semantic-summary.json에
구분했다. R2의 장평 범위로 보류된 문단을 새로 확정했다는 뜻도 아니다.

2026-10-19 메인테이너가 비교 화면과 Studio 새 탭의 이번 범위를
**“시각·편집 구조 통과”**로 판정했다. 대상은 root의 단일 축 opaque stroke이며
fill/compound·부분 병합/중첩 이웃·새 행열 상속까지 완료한 판정으로 확대하지 않는다.

## 남은 순서

다음 필수 경계는
fill/compound의 source 역할·표 밖 closure를 같은 편집 계약에 연결하는 작업이다.
새 행열 상속/삭제·복원, 부분 병합·중첩 이웃, R2의 수평 문단 메트릭은 뒤따른다.
P5에서 V2 metadata의 ID 0·이중 역할·owner 이동 같은 추가 음성 입력도 감사한다.
이번 유효 producer/축 경계와 잘림·알 수 없는 flag의 통과를 모든 비신뢰 metadata의
완료 증거로 확장하지 않는다. 전체 회귀/lint/Worker/package와 최신 devel 통합·PR은
아직 실행/완료 범위가 아니다. 원격 push·PR 등록·이슈 종료는 하지 않았다.

## 정식 회귀와 최종 보존

새 tests/cases/issue_7998_pdf_outside_border_editing.rs **7건**은 전쪽 Native/fresh WASM과
직접 판독 90% 이상을 확인한 뒤 등록했다. 자동 manifest가 실제 regression_suite_021에
수집했고 Cargo에서 7건을 실행/통과했다(formal-outside-tests.log). 외곽 교체·양 저장
후 재편집·stale shape/표/앞선 slot·문자 역편집과 두 셀의 undo·누락된 대상·V2 잘림과
알 수 없는 flag·무변경 확인의 소유 보존을 검사한다. 위치 golden이나 절대 px 핀은 없다.

기존 replacement의 `outer` 거절 기대는 이번 독립 Print/직접 판정과 맞지 않아 신규
양성 계약으로 대체했다. 같은 기존 함수의 stale/동시 resize 원자적 거절은 유지하며,
이전 여섯 검사를 regression_suite_014에서 모두 실행/통과했다(formal-replacement-tests.log).
기존 ownership/periodic/border editing/clip/placement/owned lists/#5959의 정식 함수
**25건**도 동일 최종 library의 집중 harness에서 실행/통과했다(existing-guards.log).
총 **38개 서로 다른 회귀 함수**이며 진단 probe나 중복 실행을 합산하지 않는다.
기존 대화상자·TS 코드는 변경하지 않았으며 앞서 검증한 그 소스의 테스트를 재실행하지
않았다. 이번에 실제 command·Native API·양 저장을 새 WASM으로 다시 실행했다.

Native 최종 빌드와 Docker fresh WASM 명령은 native-build-final.log·docker-wasm.log에
보존했다. 오류 경계 보정 후 Native/WASM을 다시 빌드했고, 이후 추가한 테스트·문서는
2798개 runtime source manifest를 바꾸지 않았다. 실제 pkg/public·브라우저 로드 SHA도
같음을 확인했다. JS glue는 이전과 바이트 같아 추적 rhwp.js 변경은 없다. Native/WASM
binary·pkg·실행 로그·개인 원본은 커밋하지 않는다. 49개 상대 asset/link를 확인했다.

| 최종 산출물 | SHA-256 |
|---|---|
| Native | `b1932e42dc964564983902bc3e2e6893b41f876408d1cbaecad0903406454392` |
| WASM (19490214 bytes) | `fadddb8b1d1f80704c656e285471e557a0666d743bbd29d733d0a778e73cde7a` |
| JS glue | `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6` |
| runtime manifest | `ef717c26fbae57f90e1f039ab33bb4d35a3aaa3fc2ef8452257f8dd9058d5ecc` |

증거 root는 저장소 밖
/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-outside-border이며
ignored output/pr-review/issue7998/p4-outside-border로 연결했다.
이동 가능한 직접 검토 화면은 E:\tmp\issue7998-outside-border-review\index.html이다.
제품 수정과 위 검증·판정을 같은 로컬 commit에 묶으며, commit 신원은 해당 증거 root의
committed-provenance.json에 연결한다. 원격 반영은 하지 않았다.
