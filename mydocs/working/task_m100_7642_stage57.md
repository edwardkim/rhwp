# 이슈 #7642 Stage 57 — 저장 되감김 컷의 실제 앞부분 수용

Issue: #7642

## 새 출력에서 확인한 실패

source `fb627e5345607de8556fba1ef1c2acf28b484161`의 11월 구분선 위 9·미주 사이 8·
구분선 아래 7 HWPX는 새 Native 집중 검증에서도 17쪽 65.82691%, 18쪽 40.56787%다.
같은 실행의 12·13·14쪽은 각각 95.74986%, 98.06971%, 96.79279%다.
일부 페이지 통과를 문서 전체 통과로 판정하지 않는다.

새 `review_017.png`를 직접 확인했다. 왼쪽 단의 마지막 수식 한 줄이 본문 외곽선 아래에
표시되고, Print에서는 그 줄이 오른쪽 단 첫 줄이다. 뒤의 설명·문28 그림과 풀이도
그 한 줄만큼 앞당겨진다. 이미지와 점수는 ignored
`stage15-current-validation/endnote-prefix-contract-focused-native/all-31/all-31/`에 있다.

## 독립 원본과 실제 컷

원본은 `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwpx`다.
독립 기준은 `pdf/issue7642-failed-regression-print/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7-hwpx-print-2024.pdf`다.
본문 298의 미주 내부 13은 스타일 3, 명시적 ColumnBreakType=None이다.
원본 스타일 3의 attr1=10240이며 style_resolver의 비트 16·17·18에 따른
widow_orphan·keep_with_next·keep_lines는 모두 꺼져 있다.
저장 글줄 네 개의 시작 위치는 58802·60332·0·2522로 중간에 되감긴다.
첫 줄과 둘째 줄의 선언 높이는 각각 2205지만 vpos 차이는 1530이다.
원본 속성·수식 스크립트는 `stage57-nov-source298.json`에 보존했다.

최신 `dump-pages`의 가상 문단 800은 17쪽 왼쪽 `[0,2)`와 오른쪽 `[2,4)`로
방출된다. 같은 source의 render tree에서 왼쪽 둘째 TextLine은 동적으로 읽은
bodyArea 하단을 35.133만큼 초과한다. 이 값은 진단 관측값이며 회귀 기대값이나
고정 허용값이 아니다. 소유자는 양쪽 모두 본문 298·내부 13이다.
`stage57-current-endnote-overflow-audit.json`에 bodyArea·소유·줄 상자와 비교 근거를 남겼다.

## 생산·소비 경로와 다음 보정 범위

`initial_fit.rs`의 새 공통 수용 조회는 internal_vpos_rewind를 제외하므로
이 문단의 `actual=None`이다. 실제 로그의 초기 컷은 `Some(1)`이지만 최종 방출은
2줄이다. `paragraph.rs`의 저장 되감김 처리 뒤 실제 앞부분을 확인하는
`reflowed_endnote_prefix_split`은 기존 컷보다 큰 후보만 검색한다.
이 실행은 end=3의 점유 1070.28이 가용 높이 1001.560을 초과함을 확인했으나,
기존 컷 2나 더 작은 컷 1을 확인하지 않고 기존 컷으로 돌아간다.
따라서 실제 점유 조회를 호출했다는 사실만으로 최종 컷의 수용이 입증되지 않는다.

다음 보정은 저장 컷의 확대와 축소 모두에서 같은 실제 앞부분의 수용 결과를 소비해야 한다.
분할 불가·미측정·유효 컷을 구분하고, 실제 한 줄도 들어가지 않는 경우의 전체 이월과
keep_lines·widow_orphan·명시적 경계를 함께 확인한다. 저장 되감김이 독립적인 단 경계를
표현하는 경우를 일반 전체 fit으로 무조건 취소하지 않는다. 개체의 별도 소유·예약 경로도
글줄 조회로 대체하지 않는다. 현재 실행 중인 fresh WASM의 source 해시를 유지하며
보정을 시작하기 전 해당 빌드의 실제 종료를 확인한다.

아직 새 코드 보정이나 최종 통과 판정은 아니다. 전체 Native/fresh WASM 90% 이상과
확대 판독 전에는 해당 문서의 회귀 기대값·baseline을 변경하지 않는다.

## 집중 검증의 실제 종료

이 source의 all-31 Native 집중 실행은 956.79초에 종료됐다. 선택한 7쪽은 모두
측정됐고 누락은 없다. 최저 40.56787%, exit 1이며 17·18·19쪽이 90% 미만이다.
21쪽은 97.09637%지만 앞쪽 미달을 상쇄하지 않는다. 전체 쪽수는 Print와 같은
21쪽이다. `completed.json`과 페이지별 metrics를 보존했고 새 17·18쪽 review를
직접 표시했다. 동일 source fresh WASM은 아직 빌드 중이다.

## 9월 18쪽의 별도 원점 차이

같은 source의 9월 미주 사이 20 HWP 17쪽은 96.65449%로 개선됐지만
18쪽은 79.55629%, 19쪽은 87.59770%로 미달이다. 새 18쪽 review와 원본 소유자
기반 4배 확대 `stage57-note29-p018-spacing-diagnostic-review.png`를 직접 표시했다.
문29의 설명과 수식 순서가 보존돼도 수식 뒤의 본문 간격이 Print와 다르다.

본문 264·미주 내부 2·3·4·5의 원본 저장 vpos는 0·1530·3060·4590이다.
가상 문단 903의 수식은 `X_{1}+X_{2}+X_{3}+X_{4}=11`이고 원본 높이는 1078이다.
원본 JSON은 `stage57-sep-note29-source264.json`에 보존했다. 최신 진단에서
903의 누계·줄 advance는 20.4이고 렌더도 353.8→374.2를 소비하지만,
904의 실제 시작은 380.2로 추가 전진한다. 누계는 이 추가 전진 없이
374.2부터 다음 advance를 더한다. 이 수치는 관측값이며 고정 기대값이 아니다.

이 차이는 저장 컷 2→1의 문제와 별도로 추적해야 한다. 실제 layout의
`y_before_vpos → vpos 보정 → 최종 y_offset → 다음 문단 원점`과 누계의 소비를
대조하고, 문단 903의 정상 간격·원본 저장 사다리·Print를 독립 근거로 사용한다.
특정 문단에 간격을 빼거나 선언 높이를 줄이는 방식으로 맞추지 않는다.
현재는 차이를 검출한 단계이며 이 경로의 원인·수정은 아직 미검증이다.

## 원점 분기와 실제 호출 상태의 추가 확인

고정 Native의 18쪽 render tree를 VPOS·TAC 진단 환경으로 다시 출력했고 두 명령이
exit 0으로 종료됐다. 로그는 `stage57-note29-vpos-diagnostic.log`와
`stage57-note29-tac-vpos-diagnostic.log`에 보존했다. 문단 900~905의 was_tac는
모두 false다. 수식 개체 자체가 TAC 후처리로 기준을 지웠다는 판정은 아니다.

저장 vpos가 문단 901→902에서 되감기면 `layout.rs`의
`current_vpos_rewinds_from_prev` 분기가 prev_layout_para·page/lazy base를 지운다.
902의 실제 위치는 순차 흐름에서 확정되지만 이 원점을 새 저장 기준에 연결하지 않는다.
903은 직전 문단이 없으므로 vpos_adjust에서 위치를 그대로 쓰며, 904에서
텍스트가 비어 있는 수식 문단 903을 기준으로 lazy base를 처음 역산한다.
이때 이미 소비한 후행 간격 452가 base에 다시 포함된다.

실제 904 로그는 `path=lazy prev_pi=903 prev_vpos=848770 prev_lh=1078 prev_ls=452`,
`vpos_end=850300 base=821785 y_in=464.88 end_y=470.91 result=470.91`이다.
903의 실제 시작·advance도 444.5→464.9로 관측했다. 생산한 실제 원점이 뒤 저장
사다리의 재시작으로 전달되지 않는 것이 이 경로의 결함이다.

보정은 유효한 미주 글줄의 저장 되감김을 현재의 실제 글줄 원점에 연결한다.
원점이 바뀔 수 있는 후처리 전에 확정하지 않고, 실제 layout_column_item 호출 직전의
최종 y_offset와 문단 앞 간격을 사용한다. 저장 줄의 유효성·줄 소유가 확인되지 않은
경로, 별도 float·표 배치와 편집 재조판을 이 글줄 원점의 통과로 대신하지 않는다.
측정의 실제 column scratch 배치도 같은 분기를 소비하므로 이 결과와 최종 출력을
함께 대조한다. 위의 분할 컷 축소와 이 원점 연결을 같은 수정 묶음에서 확인한다.

## 같은 수정 묶음의 구체적인 계약

앞부분 조회는 Split(줄 끝) / Overflow / 미측정을 구분한다. 기존 컷보다 큰
후보만 검색하지 않고 모든 유효한 부분 컷을 긴 순서대로 실제 column 배치에 대조한다.
widow_orphan의 앞뒤 줄 수와 keep_lines·명시적 경계도 지킨다. 실제 한 줄도
들어가지 않는 결과는 기존 컷으로 돌아가지 않고 전체 문단을 다음 단에서 배치한다.
스타일·줄 소유·별도 개체 계획을 측정하지 못한 결과와 이를 혼동하지 않는다.
세 호출 지점의 초기 컷·지역 되감김 컷·최종 방출을 함께 변경한다.

원점 재시작은 current_vpos_rewinds_from_prev 분기에서 사다리를 지운 뒤,
실제 layout_column_item에 전달하는 최종 원점에서 수행한다. 유효한 저장 글줄과
같은 줄 소유가 확인된 일반 미주 문단은 현재 원점과 spacing_before를 저장 첫 줄에
연결한다. 뒤 본문의 lazy 경로는 이 기준을 이어받으며 이미 소비한 줄간격을
재역산하지 않는다. 형상·표·별도 float·편집/dirty 저장 정보는 이 계약으로 대체하지 않는다.
명시적 저장 되감김을 삭제하거나 소스 번호·관측 높이를 조건으로 사용하지 않는다.

ignored `stage57-engine-correction.patch`에 다섯 source 파일의 수정안을 준비했고
현재 파일 해시와 `git apply --check`를 확인했다. 아직 source에 적용하지 않았다.
실행 중인 같은 source의 WASM 최적화가 실제로 종료된 뒤 적용·빌드·Native/fresh WASM
재검증한다. 회귀 테스트·fixture·baseline 변경은 이 수정 묶음에 포함하지 않는다.

## 구현 상태

앞 source fb627e534의 fresh WASM은 1670.36초에 exit 0으로 종료됐다.
WASM SHA-256은 `43bf8042072202e9869e2d3ec8e28260402ff81f3944c249a9132eba4e2ddbd5`,
JS는 `a043bdc9047d3ff5d89f55ccc21846f017ac109dad6de25b2885e43c44ee08da`다.
package/public 해시와 source 파일 해시가 각각 일치한 뒤 수정안을 적용했다.
고정된 이전 package의 WASM 집중 큐는 이전 소스의 증거로 계속 실행한다.

세 컷 소비 지점을 EndnotePrefixFit으로 연결했다. Overflow는 미측정과 구분해
최종 방출 전에 전체 문단을 다음 단으로 이월한다. 기존 컷보다 작은 후보도
실제 column의 occupied_bottom을 확인하고 필요한 widow/keep 계약을 지킨다.

layout_column_item 호출 직전 유효한 저장 미주 글줄의 최종 원점을
HeightCursor::seed_stored_flow_origin에 전달한다. 이 메서드는 저장 첫 줄과
실제 원점을 연결한 lazy base 및 확인한 column origin을 함께 기록한다.
사다리가 되감긴 뒤의 textless 수식도 이 원점을 이어받는다. 원래 저장
되감김·원본 입력을 지우지 않았고 특정 문단 번호·높이·고정 하단 여유를
구현 조건에 넣지 않았다. 새 source의 빌드·시각 판정은 아직 미검증이다.

## 새 구현의 실제 실행 결과

source `8ddb3b16963815c71da5e049f6b4ab8c2a8d111a`의 Native 빌드는
837.59초에 exit 0으로 종료됐다. 고정 바이너리의 SHA-256은
`a700b544c54fc499acd36dbf1f6cd7637a4155973536b30d82843606de3cc4ff`다.
`endnote-rewind-contract-native-provenance.json`에서 빌드 전후 source 해시와
고정 바이너리 해시를 확인했다. 앞 절의 fb627e534 출력과 구분한다.

11월 구분선 위 9·미주 사이 8·구분선 아래 7 HWPX의 dump-pages는 exit 0이며
전체 21쪽이다. 같은 원본 소유자 본문 298·미주 내부 13의 가상 문단 800이
17쪽 왼쪽 `[0,1)`과 오른쪽 `[1,4)`로 이어진다. 실제 후보 조회는 end=3의
occupied_bottom=1070.280, end=2의 1036.653, end=1의 1001.347을 같은
가용 하단 1001.560에 대조했다. 기존 컷보다 작은 수용 가능한 컷을 최종 방출에서
사용했음을 확인했다. 수치는 실행 관측값이며 회귀 검사에 고정하지 않는다.

9월 미주 사이 20 HWP의 dump-pages도 exit 0이며 전체 24쪽이다.
문단 872의 17쪽 왼쪽 `[0,1)`·오른쪽 `[1,6)` 소유가 유지된다.
같은 입력 18쪽 render tree 출력은 exit 0이다. 실제 로그의 902 재시작은
source=847240, origin=424.080, base=822237이다. 뒤의 904 lazy 조회가
이 기준을 소비해 y_in=464.88과 result=464.88을 유지한다. 앞 source에서
470.91로 다시 전진하던 간격을 추가하지 않는다.
원본·PDF 해시, 실행 명령과 소유 결과는 `stage57-current-owner-diagnostic.json`,
실제 컷·원점 로그는 `stage57-current-all-31-dump-pages.log`와
`stage57-current-all-11-export-render-tree.log`에 보존했다.

이 실행은 컷·원점 계약의 보정 증거다. 최신 소스의 전체 페이지 90% 이상이나
수식 확대 판독 완료를 뜻하지 않는다. 동일 소스의 Native/fresh WASM 출력과
전체 회귀·관계형 전환은 계속 검증한다.

## 검증 환경 실패와 재실행

Native fmt 검사는 77.96초에 exit 0으로 종료됐지만 뒤의 Native Clippy와
시각 검증은 디스크 공간 부족(ENOSPC)으로 종료됐다. Clippy 로그에 임시
디렉터리 생성 실패가 남고, 일부 큐 JSON도 쓰기 중 비어 있거나 잘렸다.
이를 조판 회귀·시각 통과·Clippy 통과로 판정하지 않는다.

종료된 이전 source의 전쪽 Native/WASM 실행과 집중 WASM 실행에서 중간
SVG만 해시·경로 원장을 남겨 정리한다. 입력·독립 PDF·PNG·TSV·render tree·
로그·고정 바이너리·package와 공유 target/pr-review는 보존한다.
정리 원장은 `stage57-terminal-svg-disk-recovery.jsonl`이며 종료 요약은
동일 stem의 `-summary.json`에 남긴다.

원래 실패 자료를 덮어쓰지 않고 `endnote-rewind-contract-focused-native-retry`에
8개 입력·37개 선택 페이지를 다시 출력한다. 각 문서의 측정 종료 뒤 해당
실행의 중간 SVG를 정리해 공간 누적을 제한한다. Native fmt·Clippy 재실행은
`endnote-rewind-contract-native-lint-retry.json`에 남긴다. 이 검사가 실제로
통과한 뒤 같은 source의 fresh WASM을 순차 빌드한다.
프로세스 소유·단계 상태는 `stage57-recovery-detached-owner.json`과
`stage57-recovery-validation-driver.json`에 기록한다. Cargo 자식 종료 뒤
owner의 복사·해시가 끝나기 전 빌드 실패로 판정했던 경합도 실제 owner를
확인하도록 수정했다. 새 Native 빌드를 중복 실행하지 않았다.

## 명시적 스타일 경계의 검증 범위

21개 HWPX의 원본 header paraPr/breakSetting을 별도로 확인했다.
확인한 스타일에는 widowOrphan·keepWithNext·keepLines·pageBreakBefore가
켜진 사례가 없었다. 이 입력들이 통과해도 해당 속성이 켜진 경계의 검증
증거로 대신하지 않는다. 원본 조사 결과는
`stage57-hwpx-explicit-break-style-census.json`에 보존했다.

## 복구 완료와 최신 17쪽 직접 판독

종료된 실행의 중간 SVG 585개, 32,880,326,349바이트를 정리했다.
`stage57-terminal-svg-disk-recovery-summary.json`의 state=complete와 원장에
경로·크기·SHA-256이 남는다. 이전 source 전쪽 Native 14개·WASM 10개의
유효한 문서별 completed.json은 별도의 terminal-enospc-recovery.json에
연결했다. 일부 큐 JSON의 손상과 미실행 쪽을 완료로 바꾸지 않았다.

재실행한 Native fmt는 44.52초, Native Clippy는 104.53초에 각각 exit 0으로
종료됐다. 이후 같은 source의 fresh WASM을 detached owner에서 순차 빌드한다.
이 단계에서 source를 바꾸지 않았다.

최신 Native 17쪽의 직접 raster는 97.36303%로, 앞 source의 65.82691%에서
개선됐다. 엄격 내용 픽셀은 32.92294%다. 새 review를 직접 표시했고 왼쪽
하단의 이어지는 글줄이 Print와 같이 오른쪽 첫 줄로 이동함을 확인했다.
증적은 `stage57-current-selected-direct/all-31`의 compare·overlay·review와
metrics.json에 있다. 대형 SVG를 만드는 집중 실행에서 이미 완성된 같은
source SVG를 사용했고 기준 PDF 17쪽도 96dpi로 새로 출력했다. 독립 Print
PDF·입력·바이너리·SVG 해시와 실제 명령을 기록했다. compare 디렉터리 생성
누락으로 멈춘 진단 driver는 같은 실행의 완료 PNG로 비교 생성을 마쳤다.

그러나 본문 298·미주 내부 13·글줄 `[1,4)`을 render tree 소유 기준으로 같은
좌표·4배 확대해 확인하니 첫 수식에 `Rm`이 글자로 표시된다. Print에는 해당
글자가 없고 뒤의 분수와 본문이 겹치지 않는다. 최신 RHWP에서는 그 추가 글자가
뒤 분수 위치를 밀어 본문과 겹친다. 따라서 97.36%를 최종 수식 통과로 판정하지 않는다.
새 확대 review와 crop·소유·원본 출처는
`stage57-current-nov-p017-prefix-equation-detail-review.png` 및 동일 stem의
`-provenance.json`에 보존했다. 전체 문서·fresh WASM·관계형 회귀는 아직 미완료다.

## 최신 집중 검증의 추가 결과

11월 HWPX의 7쪽 집중 검증은 601.11초에 종료됐다. 12·13·14·17·18·19·21쪽을
모두 측정했고 누락은 없다. 17쪽은 97.36303%, 18쪽은 96.88649%로 개선됐지만
19쪽은 81.85884%로 여전히 미달이다. 전체 쪽수는 Print와 같은 21쪽이며
이 선택 범위의 gate는 re_review_required다. 문서 전쪽 검증은 아직 아니다.
새 18·19쪽 review도 직접 표시했다. 문서별 completed.json과 실행 로그를 보존했다.

9월 미주 사이 20 HWP 18쪽은 같은 최신 실행에서 97.27269%다. 앞 source의
79.55629%와 같은 좌표의 Print를 3배 확대해 비교했다. 수식 뒤 본문에 추가되던
전진이 사라지고 Print와 같은 간격으로 이어진다. 증적은
`stage57-current-sep-note29-p018-spacing-detail-review.png`와 동일 stem의
`-provenance.json`이다. 이 문서의 다른 선택 쪽과 전쪽 판정은 진행 중이다.
