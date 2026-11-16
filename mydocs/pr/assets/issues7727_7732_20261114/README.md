# #7727·#7729·#7730·#7731·#7732 분석·검증 증적 안내

상세 원인 분석은 [단계별 분석·보정 보고서](../../../working/task_m100_7727_stage1.md)에
기록한다. 이 문서는 그 보고서와 커밋된 증적을 연결하는 진입점이다.

## 현재 확인한 결과

최종 제품 source는 `3f7e4340ae3038ea77917ff0f45f816e8342770e`, 비교 base는
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. Native와 fresh WASM 각각 원본5종
**전389쪽이98% 이상**이다. 최종 source의 Mac 최소 글꼴 전체 회귀는
**13,565 PASS / 0 FAIL / 45 skip**이며 필수 Rust14단계도 모두 통과했다.
[최종 Rust 검증](canvaskit-common-3f7e4340/rust-validation.json)과
[새 WASM389쪽 검증](canvaskit-common-3f7e4340/fresh-wasm-389-validation.json)을 따른다.

아래 Native TSV·대표 그림은 source1123f322에서 산출했다. 이후 Rust 제품 차이는 paint JSON의
공통 원형 전달뿐이고 Native/직접 WASM Canvas 그리기 경로는 동일하다. 최종 source로 다시
산출한 WASM389 PNG도 이전 파일과 전부 바이트가 같았다.
[경로 재사용 감사](canvaskit-common-3f7e4340/unchanged-path-reuse-audit.json)에 따라 Native389쪽과
보호3,542쪽의 기존 실행을 재사용하며 과거 실행 SHA를 새 실행으로 바꾸어 쓰지 않는다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 같은 source 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 98.13% (58쪽) | 98.13% (58쪽) | [Native](final-1123f322/native-cbta.tsv) · [WASM](final-1123f322/wasm-cbta.tsv) |
| 권고안 | 22 | 98.69% (7쪽) | 98.68% (7쪽) | [Native](final-1123f322/native-recommendation.tsv) · [WASM](final-1123f322/wasm-recommendation.tsv) |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | [Native](final-1123f322/native-exam.tsv) · [WASM](final-1123f322/wasm-exam.tsv) |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | [Native](final-1123f322/native-api.tsv) · [WASM](final-1123f322/wasm-api.tsv) |
| 각주 표 | 50 | 98.75% (37쪽) | 98.75% (37쪽) | [Native](final-1123f322/native-footnote.tsv) · [WASM](final-1123f322/wasm-footnote.tsv) |

입력·독립 Print PDF의 [해시와 출처](final-1123f322/inputs.json),
[검토 commit과 실행 bytes 일치](final-1123f322/committed-original-input-audit.json),
[Native 생산자](final-1123f322/native-producer.json)·[WASM 생산자](final-1123f322/wasm-producer.json)를 연결한다.
정상 설치 글꼴·96DPI·compat2022·print·raw2px·채널 문턱32를 사용했다. Native는 실제
CLI `export-png`, fresh WASM은 실제 WASM Canvas PNG이며 SVG raster로 대신하지 않았다.
Mac WASM은 사용자 지시의 호스트 `--target web --out-dir pkg --no-opt` 대체 빌드다.
Docker 또는 wasm-opt 검증으로 보고하지 않는다. 기준 PDF·판정 문턱은 변경하지 않았다.

[보호 검증](final-1123f322/protected-validation.json)은 필수 Native 편람383쪽을 포함해
36조건 전3,542쪽 완료, 정본의 공식 두 자리 TSV 하락0·상승0이다. 원본 Native의
[고정 base 비교](final-1123f322/native-fixed-base-comparison.json)는29쪽 상승·3쪽 하락,
최대 하락0.06%p이며2%p 초과 하락은 없다. 세 하락 쪽의 같은 쪽 review/overlay에서
내용·소속·줄·목차 번호 보존을 직접 확인했다.

### 고정 base 및 소비 경로 검증

고정 base의 Native/fresh WASM 전389쪽 대조도 완료했고 모든 경로에서2%p 초과 하락은0쪽이다.
[고정 base 대조](final-1123f322/fixed-base-comparison.json)를 따른다.
실제 Studio Canvas2D/CanvasKit 소비 경로에서 확인한 차이는 문서 끝의 잔여 항목에 기록한다.
CBTA204쪽과 API·각주 문서의 기존 page/work preflight 한도는 변경하지 않았으며,
의도된 Canvas2D 대체 경로를 실제 CanvasKit 통과로 계산하지 않는다.
앞선 Undo5FAIL은 실제 편집 캐시와 Undo 복원/빈 입력의 소유를 구분해 제품에서 보정했고,
최신 전체 회귀에서는 모두 통과했다. 과거 실패·중간 결과를 새 source의 통과로 재사용하지 않는다.

## 원인별 분석과 실제 보정

| 대상 | 확인한 원인과 보정 | 독립 근거·실행 증적 |
| --- | --- | --- |
| CBTA 96쪽 번호 문단 | 마커 높이와 줄 수가 같으면 현재 프레임의 새 문자 경계를 버리던 캐시 수용 조건을 보정했다. 원본 세로 슬롯은 보존한다. Native 97.00%→99.72%, WASM 96.99%→99.74%. | [원인·생산자·회귀](marker-partition/cause.json), [389쪽 전후 비교](marker-partition/page-comparison.json) |
| CBTA 162·166쪽 종료 행 | 근거 없이 남은 쪽 공간을 완료 행에 배분하던 처리를 제거하고, 실제 병합 칸이 소유한 종료 점유만 소비한다. 두 경로 모두 99.94%·99.95%. | [원인·전쪽 결과](complete-row-band/cause.json), [PDF 종료 선 좌표](marker-partition/cbta-pdf-complete-row-close-lines.json) |
| 권고안 4쪽 배경 | RGB 최종값 반올림과 독립 Print의 채널 변화량 절삭 차이를 공통 보간에서 보정했다. 두 경로 모두 97.44%→99.96%. | [원인·독립 인쇄 표본](gradient-channel/cause.json), [MCP 출력 영수증](gradient-channel/gradient-band-color-print-status.json) |
| 영어 시험지 낱말 공백 | 영문 상대 크기를 반각 공백의 기준 em에도 적용하던 경로를 분리했다. 원본 8쪽의 양쪽 최저 99.49%. | [원인·독립 인쇄 표본](half-space-origin/cause.json) |
| 영어 시험지 밑줄 | 글자 위치 변화와 줄 기준선의 밑줄 원점을 분리했다. 원본 장식 run은 보존한다. | [독립 한컴 Print와 전후 검증](underline-baseline/cause.json) |
| CBTA 그림 캡션·API 표·권고안 단 시작 | 소유 그림과 테두리의 동시 이동, 표 앞뒤의 중복 점유, 빈 줄과 다음 물리 프레임의 경계를 공통 소비 경로에서 보정했다. | [상세 분석 보고서의 원인별 규칙](../../../working/task_m100_7727_stage1.md#원인별-보정-규칙) |

위 표의 앞선 단계 결과는 해당 단계 생산자의 증적이다. 현재 전체 쪽수와 Native 최저값은
`final-1123f322/`의 TSV, 최종 fresh WASM은 `canvaskit-common-3f7e4340/`의 TSV를 따른다. `note-prefix-final/`은 앞선 제품 소스의 증적이며,
과거 측정을 새 head의 실행으로 바꾸어 쓰지 않는다.

## 같은 쪽의 review와 overlay

| 보정 영역 | Native | fresh WASM |
| --- | --- | --- |
| CBTA 96쪽 문자 경계 | [review](marker-partition/native-p096-review.png) · [overlay](marker-partition/native-p096-overlay.png) | [review](marker-partition/wasm-p096-review.png) · [overlay](marker-partition/wasm-p096-overlay.png) |
| CBTA 162쪽 종료 행 | [review](complete-row-band/native-p162-review.png) · [overlay](complete-row-band/native-p162-overlay.png) | [review](complete-row-band/wasm-p162-review.png) · [overlay](complete-row-band/wasm-p162-overlay.png) |
| CBTA 166쪽 종료 행 | [review](complete-row-band/native-p166-review.png) · [overlay](complete-row-band/native-p166-overlay.png) | [review](complete-row-band/wasm-p166-review.png) · [overlay](complete-row-band/wasm-p166-overlay.png) |
| 권고안 4쪽 배경 | [review](gradient-channel/native-p004-review.png) · [overlay](gradient-channel/native-p004-overlay.png) | [review](gradient-channel/wasm-p004-review.png) · [overlay](gradient-channel/wasm-p004-overlay.png) |
| 영어 시험지 2쪽 공백 | [review](half-space-origin/native-p002-review.png) · [overlay](half-space-origin/native-p002-overlay.png) | [raw PNG](half-space-origin/wasm-p002-raw.png) · [전쪽 TSV](note-prefix-final/wasm-exam.tsv) |

## 회귀 검사와 독립 편집·저장 검증

기존의 독립 기대값이 정확한 검사는 유지하고, 틀린 수집/순서 기대만 실제 독립 PDF와
원 해결 PR을 추적한 뒤 관계형 검사로 바로잡았다. CBTA118·162·166쪽은 양쪽98% 이상과
독립 PDF 직접 판독을 확인해 ±2 CSSpx 보호를 추가했다. 저장 RIGHT 탭은 +1500HU 독립
Print의 +19.99996 CSSpx 이동, 저장 줄 없는 재조판, 실제 편집·2회 저장, Undo 및 빈 입력을
각각 검사한다. 새 검사와 기존 Undo/bbox 대조군은 최소 글꼴12PASS이며 전체 회귀에도 포함된다.

실제 공개 편집 API로 편집한 뒤 두 번 저장·재열기한 작은 HWP/HWPX 독립 Print 대조군은
[네 경로 검증](final-1123f322/edited-save-print/validation.json)에서 Native99.90%, fresh WASM100.00%다.
원본과 숫자 세 줄의 순서·RIGHT 정렬을 네 쌍 review/overlay로 직접 판독했다.
HWPX는 기존 raw numbering namespace 직렬화 문제를 분리한 canonical prefix 대조군이며,
이 결과를 일반 namespace alias 결함까지 해결한 것으로 확대하지 않는다.

모든 필수 검증과 병합이 끝난 뒤 #7727·#7729·#7730·#7731·#7732의 실제 해결 범위를
재확인하고, 완료 댓글·종료 및 #7445 해당 행의 취소선을 반영한다. 부분 해결 이슈와
대표 #7445 자체는 이 다섯 건의 완료만으로 닫지 않는다. 현재 원격 종료는 실행하지 않았다.

임시 진단기·실행 파일·로그·MCP 대기 자료와 별도 namespace 실패 진단은 저장소의
`output/pr-review/issues-7727-7732-20261114/`에 둔다. 확정된 분석과 제출 증적은
이 디렉터리와 [단계별 보고서](../../../working/task_m100_7727_stage1.md)에 보존한다.
임시 산출물과 비공개 글꼴은 커밋하지 않는다.

### 최종 실제 Studio 소비 검증

공통 물리 원을 paint JSON으로 전달해 CanvasKit 권고안4쪽을 같은 조건에서
**53.71%→98.26%**로 회복했다. 동점 face 선택 보존도 보완했으나 face만 바꾼 분리 진단의
PNG는 동일했으므로 이 점수 상승의 원인으로 설명하지 않는다.
[원인·전후 검증](canvaskit-common-3f7e4340/validation.json),
[review](canvaskit-common-3f7e4340/after/review.png)와
[overlay](canvaskit-common-3f7e4340/after/overlay.png)를 보존한다.

정상 설치 SFNT 프로그램을 canonical Visual Sweep과 같은 공통 Rust 선택 API로 선언한 실제
Studio Canvas2D Print15쪽은 최저98.21%다. 실제 설치 bytes·이름을 공급하는 플랫폼 API adapter를
사용했으며 Chrome의 OS 글꼴 열거 자체가 검증됐다는 뜻은 아니다. 별도 자동 글꼴 선택 조건의
낮은 진단값을 수용 결과와 섞지 않는다. 기존 CK page/work/font preflight와 fallback은 보존했으며,
실제 Canvas2D로 대체된 요청은 CK 통과로 세지 않는다.
[소비 검증](canvaskit-common-3f7e4340/studio-canvas-validation.json)과
[대표7쪽 직접 판독](canvaskit-common-3f7e4340/studio-direct-reading.json)을 연결한다.

최종 Studio 전체 검사는2,791PASS·0FAIL·2skip이며 tsc·build·renderer 계약도 통과했다.
[Studio 검증](canvaskit-common-3f7e4340/studio-validation.json)을 따른다.
일부 글리프 가장자리·테두리 농도·미세 간격 차이는 판독에 남아 있으므로100%로 주장하지 않는다.
