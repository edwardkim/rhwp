# JinHo-von-Choi PR 3건 로컬 검증 원장

대상은 #9070·#9071·#9072다. 원 기여 head 3개를 보존하고, 사용자 승인에 따라 괘선 매핑·바탕쪽 공통 프레임·원본 TAC 후행 간격을 보완했다. 현재 판정은 **머지 보류**다. 기능 검증과 과거 특별 보호값 재현을 구분하며, 원격 댓글·push·PR·승인 review·merge는 아직 수행하지 않았다.

## 최종 제품과 검증 범위

제품 빌드 source는 `eeaecff59e834f28437c98b509089bbb36a7682d`, 회귀 검증 head는 `3b5e750d66351799fe81c0e1b67c5e78323763f5`다. 뒤의 변경은 검사·증적이며 제품 경로의 Git diff는 0이다. 실제 Native 실행 파일·Docker wasm-pack와 wasm-opt를 마친 JS/WASM 해시는 [전쪽 결과](feature-results.json)와 [source 일치 확인](source-equivalence.json)에 보존한다. 빌드한 SHA를 이후 문서 commit으로 바꿔 적지 않는다.

| 범위 | 경로별 쪽수 | Native 최저 | 최적화 fresh WASM 최저 |
| --- | --- | --- | --- |
| 정상 바탕쪽 11조합, 원 HWPX와 실제 HWP 저장본 | 66 | 100.00% | 100.00% |
| 색 없음 원 HWP와 실제 HWPX 저장본, 2020 Print | 6 | 100.00% | 100.00% |
| 괘선 정상 specimen과 HWP/HWPX 저장 재열기 | 3 | 98.57% | 98.55% |
| 대칭·반전 여백과 정상 한컴의 실제 사각형·표 | 12 | 100.00% | 100.00% |
| 새 devel 기능 보존 7문서 | 41 | 93.30% | 92.46% |
| #8307 원 HWP·정상 HWPX, 신고 당시 실제 2018 Print | 6 | 100.00% | 100.00% |
| 합계 | 134 | 93.30% | 92.46% |

각 입력의 전체 쪽수는 해당 독립 Print PDF와 같다. compat 2022·96dpi·색상 경계 보정 전 2px 내용 실루엣 raw이며, Native Skia와 실제 브라우저 Canvas2D에서 측정했다. 자동 `not_evaluated` 상태를 시각 승인으로 바꾸지 않았다. 7개 괘선 집합·기호, 맞쪽 막대의 용지 경계 절단·너비·원점, 표 외곽과 후속 두 문단의 쪽 소속·내용·빈 줄·순서를 직접 판독했다.

바탕쪽·TAC 집중 범위는 양쪽 각 33쪽 모두 100.00%다. 최종 집중 27쪽의 PNG는 앞서 직접 판독한 출력과 같고, 별도 2018 Print 6쪽도 직접 읽었다. [#9070 근거](../pr9070/README.md) · [#9071 관계 검사·확대 PNG](../pr9071/README.md) · [#8307 원본·2018 Print](../pr9071/issue8307/README.md). 새 devel 기능 보존 41쪽씩의 수정 전후 PNG 82쌍도 모두 같다. [보존 결과](incoming-preservation.json).

## Rust·실제 CLI·WASM API

2026-11-15 KST 최종 전체 회귀는 **13,574개 통과·실패 0·기존 skip 45개**다. fmt, root Clippy, WASM32 Clippy, workspace build·all-targets Clippy와 manifest 검사가 통과했다. Native 공식 3종도 **4,073개 모두 통과**했다. [최종 명령·검사별 PASS·노출 글꼴·로그 해시](local-validation.json).

전체·집중·Native 회귀는 `bwrap`에서 시스템·개인 글꼴 공급 경로를 제한하고 `RHWP_FONT_PATH`를 해제해 실행한다. Fontconfig의 실제 노출은 저장소의 오픈소스 5개 파일뿐이다. 시각 검증용 상용 글꼴 공급과 회귀 환경을 섞지 않는다. 최종 집중 126개에는 실제 생성 harness의 새 모듈 3개와 composer가 포함됐다. 전체 회귀에서도 새 검사 3개와 기존 `owned_rowbreak_tac_height_selects_current_or_multirow_frames`의 PASS를 확인했다.

[실제 CLI 본문 편집·HWP/HWPX 각 2회 저장 재열기](cli-repeated-save.json)에서도 바탕쪽 XML·그림 bytes와 다른 쪽 label을 보존했다. [실제 CLI 결과](cli-results.json)는 5개 TAC 입력의 원문·순서·단일 출현과 replace-text의 0 기준 도움말을 보호한다. [최적화 WASM 실제 API](wasm-api-results.json)는 FieldInfo의 본문 전역 문단 13·상태 0x12, 세 구역·중첩 셀 생성과 HWP/HWPX 각 2회 저장·재열기 16개 비교, 잘못된 전역 16의 거부·무변경을 확인한다. 웹 컨트롤의 IR 문자열 API와 CLI의 render-tree 텍스트는 다른 계약으로 검증했다.

## 특별 보호값 재현

편람 HWPX Native 383쪽 `--handbook-gate`는 통과했다. [실제 gate](handbook-gate.json). 별도의 정본 보호 36 dataset·3,542행은 Native 51행·WASM 56행에서 과거 공식값을 재현하지 못했다. 전쪽 90% 이상이지만 특별 하락 금지 gate는 **통과하지 않았다**. [실제 결과](protected-results.json) · [환경·base 대조](protected-environment-review.md).

Native 1,771행은 정확한 최신 devel `53da4d2da2`와 후보의 측정값이 모두 같다. WASM의 미재현 56행도 같은 base의 최적화 빌드와 대조 중이다. 기준 TSV·기대값·비교 영역을 낮추거나 수용 예외를 적용하지 않았다. 사용자가 안내한 HBATANG 파일은 실제로 사용 중이며, 과거 파일과 해시가 다르다는 사실만으로 원인을 글꼴로 확정하지 않는다.

## 앞선 보고 정정

초기 집중 60개는 동적 suite 배정과 오래된 생성 harness가 달라 새 바탕쪽 모듈을 실행하지 못했다. 이를 새 검사 통과에 포함한 보고를 철회한다. 이후 `--prepare` → 실제 harness의 모듈 확인 → 실행 로그의 검사별 PASS 확인을 적용했다.

`b60322c765` 전체 회귀의 두 실패는 원점 0 재시작과 양수 되감김을 구분하지 않은 TAC 판정, 새 HWP 저장 검사가 형식 어댑터를 생략한 문제였다. `eeaecff59e`에서 판정을 보완하고 실제 CLI의 `export_hwp_with_adapter`를 사용했다. 기존 정확한 기대값·배치 assertion은 유지했다. 최종 13,574개 통과가 이 두 실패의 재검증 결과이며, 앞선 source의 통과 수를 현재 결과로 재사용하지 않았다.

[연결 이슈의 해결 범위와 후속 조건](linked-issue-disposition.md). 필수 보호 판정이 해결되기 전 제출·통합 완료를 선언하지 않는다.


## 검증 파일 보존

[실제 사용 입력·PDF 목록](validation-inputs.json)에 113개 파일의 역할·검증 위치·저장소 경로·SHA-256을 기록했다. 변환/재저장본 26개도 검증한 bytes 그대로 정식 증적의 inputs 아래 보존했으며 동일성 확인으로 경로 변경을 기록한다. 50 MiB 이상인 hwp3-sample10 PDF 1개(96,185,197바이트)는 이번 작업 전부터 base에 추적된 파일이다. 해당 파일은 변경하지 않으며 PR diff에 추가하지 않는다. 기본 작업공간의 기존 안정 경로도 같은 SHA-256임을 확인했다. 새 대용량 PDF는 추가하지 않는다. 입력·PDF 113개 모두 `0844c7d115`의 실제 Git blob과 실행 파일의 SHA-256이 같음을 [커밋 대조 기록](input-commit-verification.json)으로 확인했다.
