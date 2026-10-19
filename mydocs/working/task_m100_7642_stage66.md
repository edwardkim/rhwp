# #7642 Stage 66 — 재조판 이력과 실제 저장 쪽 경계의 분리

Issue: #7642

## 독립 근거와 원인

Stage 65의 빈 줄 앞 간격 수정 뒤, 같은 빌드에 포함할 쪽 경계 원인을 조사했다.
55개·853쪽 및 관련 회귀 전체 범위는 그대로다.

`samples/hwp3-sample16-hwp5.hwp`의 독립 2024 Print 26쪽은
488 `Ⅴ. 도입장비 내역서` → 489 소제목 → 490 표 순서다.
현재 Native는 488을 25쪽 끝에 두고 26쪽을 489부터 시작한다.
26쪽 review 34.07849%를 직접 표시했으며 제목 누락과 표 전체 원점 차이가 보인다.

486은 NO_LS 본문, 487은 저장 빈 줄, 488은 새 쪽 첫 저장 줄이다.
487의 vpos 68144HU + 높이 1600HU + 간격 960HU = 70704HU이며
실제 488 진입 흐름 942.72px와 같다. 488은 vpos 852HU에서 새로 시작한다.
`stored_page_boundary_invalidated_by_reflow`는 빈 줄을 거슬러 NO_LS 이력만
발견하면 이 실제 일치까지 무효화했다. 저장 경계는 재조판 이력만으로
유효하지도 무효하지도 않으며 현재 실제 소유 흐름과 대조해야 한다.

## 공통 판정과 소비 경로

저장 줄의 끝/다음 첫 줄 → 실제 소비 높이와의 기존 경계 대조 →
section/stored_reset의 전진, paragraph/whole_fit의 통째 수용,
paragraph/split_entry의 시작 경계를 같은 helper로 판정한다.
기존 near-top 및 저장 끝/실제 흐름 대조를 재사용하며 새 문서 지문이나
특정 보정값을 추가하지 않는다. dirty 첫 문단은 권위 있는 근거로 인정하지 않는다.

whole_fit은 기존 판정에서 쓰던 실제 점유 높이를, entry와 split은 해당 단계의
실제 current_height를 전달한다. 부동 개체 점유는 whole_fit의 기존 계산을 따른다.
새 반환값은 경계 무효화 여부이며 실제 쪽 전진은 각 소비 단계의 기존 가드가 맡는다.
현재 높이와 저장 끝이 다르면 NO_LS 이력이 무효화하는 기존 동작을 유지한다.
개체 owner·저장 본문 owner의 기존 분기는 유지한다.

## 반례와 실행 계획

재조판으로 줄 수가 바뀌어 저장 끝과 실제 소비 높이가 어긋난 경계,
새 쪽 상단이 아닌 부분 되감김, dirty 정보, #7160 전체 3쪽,
기존 미주 정상 경계를 함께 확인한다. sample16 6/22/23/26/41쪽을 먼저
비교하고 26쪽에서 제목·표·뒤 문단 순서와 원점을 직접 판독한다.
Stage 65~66을 함께 Native 빌드해 서로 다른 원인의 보정을 묶어 검증한다.
이 기록 시점에는 새 코드의 빌드/Visual Sweep이 아직 완료되지 않았다.
90% 미만인 상태에서 정식 회귀 기대값·golden을 변경하지 않는다.

## 실제 빌드와 추가 진단

묶음 source `a7397b428a2064e7787e704961b746cf4b54f30a`의 Native 빌드는
exit 0, 425.682826초다. frozen 바이너리 SHA-256은
`09b5f7a8de43d5c86637f201abd88bbd1a9e6f5bf46a45a251aa8710c367e68c`이다.
fmt check와 Native Clippy도 각각 exit 0이다. 전체 필수 lint 완료를 뜻하지 않는다.
ignored output의 blank-line-origin-native-provenance.json 및
blank-line-origin-native-lint.json에 명령·파일 해시·실행 결과를 보존한다.

직전 81989aafb의 대표 fresh WASM 9개 대상은 모두 90% 이상으로 종료됐다.
이 결과는 a7397b428의 WASM 검증으로 재사용하지 않는다.
기존 #7160 대조 3쪽은 81989aafb와 4ad34f7f2가 모두 86.51542%이며
실제 PNG SHA-256도 동일하다. 제목 문자열 추정 제거로 생긴 차이가 아니며,
기존 미달을 정상 일치로 승인하지 않는다. 첫 시도는 browser navigation
30초 한도를 넘겨 실패했다. 종료 확인 후 대형 글꼴 SVG에 맞는 기존 180초
옵션으로 재실행했고 동일 결과를 확인했다.

23쪽 첫 문단 450은 NO_LS, 해석된 앞 간격은 15.093333px다.
4ad34f7f2 실제 SVG의 첫 기준선은 90.36px, Print는 105.440023px로
차이가 앞 간격과 일치한다. 측정은 format의 spacing_before를 예약하지만
paint의 layout_partial_paragraph → layout_composed_paragraph 경로가
재조판된 첫 줄의 column-top 간격을 생략한다. 저장 vpos 재사용을 막는
조건과 실제 흐름 간격 소비를 같은 상태로 사용하고 있어 추가 보정이 필요하다.
원본/Print/overlay 첫 문단 확대 PNG를 직접 표시했다. 이 진단은 새 Native
코드의 개선 결과가 아니다. 공백·양끝 정렬과 내부 기호 모양도 별도 차이가
보여 세로 간격 보정만으로 완료를 선언하지 않는다.

관련 원본·Print의 언어별 글자 속성도 읽었다. shape 55의 자간은 7개 slot
모두 -20%이며 슬롯 간 자간 차이가 원인이라는 추정을 채택하지 않는다.
frozen parser metadata 라이브러리를 사용한 진단은 compile/run exit 0이며,
정식 renderer 회귀 검사와 구분한다. 원시 font size는 행렬을 함께 읽었다.
자료는 stage66-page23-* 및 stage66-distribution-before-after.json에 보존한다.

폐기된 81989aafb 전체 Native/WASM 작업은 정확한 owner PID와 자식 argv를
확인한 뒤 종료했다. 종료 뒤 해당 28개 PID가 모두 사라진 것을 확인했으며
입력·PDF·기존 PNG/TSV는 보존했다. stage66-superseded-whole-stop-ledger.json이
실제 처리 증거다. 전체 55개를 완료했다고 보고하지 않는다.
