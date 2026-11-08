# 이슈 #7642 Stage 58 — 붙은 글꼴 명령의 대소문자 혼용

Issue: #7642

## 독립 출력에서 확인한 결함

Stage 57 source `8ddb3b16963815c71da5e049f6b4ab8c2a8d111a`의 11월
구분선 위 9·미주 사이 8·구분선 아래 7 HWPX 17쪽은 Native 실루엣 97.36303%다.
본문 298의 미주 내부 13을 4배 확대하니 RHWP 첫 수식에 `Rm`이 추가로 보이고,
뒤의 분수와 본문이 겹친다. 독립 한컴 Print PDF의 같은 수식에는 이 글자가 없다.
수식의 원문은 다음과 같다.

```text
bar{rmOQ_1 }= bar{RmOM_1 }- bar{rmQ_1 M_1} = root5 over 5
```

원본·기준·최신 이미지와 명령은 Stage 57 기록 및 ignored
`stage57-current-selected-direct/all-31/metrics.json`에 연결했다.
원문을 고치거나 선언 폭을 늘려 문서를 맞추지 않는다.

## 생산·소비 경로와 원인

`tokenizer.rs::read_command`의 붙은 글꼴 명령 조회는 rm/RM·it/IT·bold/BOLD의
두 철자만 matches_at으로 분리한다. 일반 fallback의 GLUE_SAFE에는 이 글꼴
접두사가 없어 RmOM을 하나의 Command로 남긴다. parser의
parse_command_fallback은 분리된 명령의 대소문자를 이미 무시하지만 RmOM 전체는
글꼴 명령이 아니어서 Text가 된다. 이 AST의 글자가 equation layout 폭·장식·
실제 SVG에 전달된다. 수식 개체의 저장 영역은 원래 글꼴 전환을 전제로 하므로
추가 글자는 문단의 뒤 본문과 겹칠 수 있다.

21개 원본 HWPX를 읽어 따옴표 문자열 밖의 붙은 글꼴 명령을 조사했다.
9개 문서·27개 수식에 대소문자 혼용 접두사가 있다. 11월 2022 HWPX 및
7개 다른 2024 변형에도 RmP/RmC/RmOM이 반복된다. XML 조사 자료는
`stage58-mixed-font-prefix-source-census.json`에 보존했다.
이 조사는 tokenizer 실행·HWP 전체 조사나 각 수식의 Print 판정을 대신하지 않는다.

## 다음 수정 묶음과 검증

붙은 글꼴 전환 bold/it/rm은 parser와 같은 ASCII 대소문자 규칙으로 분리하고,
토큰의 원래 철자와 피연산자·첨자·장식 순서를 보존한다. 기존 times/sim의
분리 조건, 따옴표 문자열과 비명령 식별자는 이 수정으로 확대하지 않는다.
특정 문서·문단 번호나 수식 폭을 조건으로 넣지 않는다.

일반 rm/RM·공백 있는 Rm·붙은 Rm/rM·it/bold의 혼용·첨자 및 따옴표 문자열을
수정 전후 토큰/AST로 대조한다. 해당 입력들의 기존 정확한 검사는 유지한다.
이미 1000줄을 넘는 tokenizer.rs를 수정할 때는 테스트 모듈을 동일 모듈의
하위 파일로 분리해 코드 파일 크기 원칙도 지킨다. 새 배치 회귀는 전체 관련
Native/fresh WASM 90% 이상 및 확대 판독 뒤에 관계형으로 추가·변경한다.

현재 같은 source의 fresh WASM 빌드가 실행 중이다. source 해시를 바꾸지 않고
ignored `stage58-mixed-font-prefix-correction-proposal.json`에 수정안을 준비했다.
실제 빌드 owner 종료를 확인한 뒤 원본 파서 증거와 수정 묶음을 적용하고,
Native/fresh WASM 영향 페이지·대조군을 새로 출력한다. 아직 구현·최종 통과가 아니다.

## 19쪽의 같은 호스트 점유 소유 조사

최신 19쪽은 81.85884%이며 새 review를 직접 표시했다. 오른쪽 아래 문27의
시작·그림·풀이가 앞당겨진다. 독립 PDF bbox의 문26·문27 시작은 각각
270.024·690.504이며 같은 Native TextLine은 270.1·672.5다. 앞 문항은 맞지만
다음 제목이 약 18만큼 일찍 시작한다. 이는 관측값이며 고정 기대값이 아니다.

직전 본문 379·미주 내부 5·가상 문단 917의 원본 text는 개행 한 개이며,
TAC 그림 줄과 빈 줄 두 개의 저장 lineSeg가 있다. 마지막 빈 줄 높이 900과
후행 간격 452가 원본에 남는다. Native TextLine도 그림 줄 473.2~642.3 뒤에
빈 줄 648.3~660.3을 표시한다. 소유가 같은 FullParagraph와 Shape가 차례로
방출되고 Shape의 흐름 전진은 0이다. 다음 제목은 본문 393·내부 0·가상 918이다.

paragraph_layout은 실제 소비한 빈 줄의 끝을 last_item_flow_line_bottom에
전달하지만 column item 루프는 다음 Shape 앞에서 그 값을 NaN으로 지운다.
뒤의 occupied_end와 origin_after_gap이 같은 호스트의 빈 줄 끝 대신 그림 끝을
소비할 수 있다. 원본의 빈 줄+후행 간격 전진이 관측 차이와 대응한다.
이것은 코드 검토의 원인 후보이며 항목별 실제 소비 값의 수정 전후 확인이 필요하다.
원본·다음 미주·정확한 코드 경로와 관측값은
`stage58-nov-tac-empty-tail-ownership-diagnostic.json` 및
`stage58-nov-source379.json`·`stage58-nov-source393.json`에 보존했다.

보정은 같은 문단에 이미 배치된 TAC 호스트 글줄의 점유 끝을 후속 제어 항목에서도
이어받게 해야 한다. 가시 콘텐츠와 빈 줄 점유를 구분하고 다른 문단·단의 결과를
누적하지 않는다. 고정 높이를 더하거나 제목을 clamp해 맞추지 않는다. 같은 column
scratch 측정과 실제 배치가 이 결과를 소비하는지 함께 확인한다. TAC 그림 뒤 빈 줄,
빈 줄 없는 그림, 같은 호스트의 복수 제어 항목과 새 소유자의 첫 문단을 대조한다.
대조군을 조사한 뒤 이 보정을 붙은 글꼴 명령과 같은 구현 묶음에서 확인한다.

## 원점 소비 뒤의 저장 간격 재가산 확인

9월 미주 사이 20 HWP 19쪽 문28의 Print bbox는 802.984이며 Native는 858.3이다.
직전 본문 320·미주 내부 18·가상 974의 실제 글줄은 668.7~727.5다.
미주 간격은 5669HU, 즉 약 75.59이므로 실제 점유 끝 뒤의 한 번 예약은
약 803.09로 Print에 대응한다. 원래 저장 원점에서는 가상 974의 시작
1050933+높이 4410과 다음 제목 1059480의 차이가 4137HU, 즉 55.16이다.

layout의 owned_endnote_origin을 HeightCursor가 먼저 소비해도, 뒤의
큰 수식 tail·후속 큰 TAC 경로에서 saved_head_gap_px를 y_offset에 다시 더한다.
실제 출력의 초과 55.16과 대응한다. 같은 helper를 먼저 사용한 것만으로 최종
원점 공유가 끝나지 않은 사례다. 완성 미주 경계의 실제 점유 결과를 얻었으면
그 뒤의 저장 좌표 기반 추정 간격·축약 분기를 사용하지 않는다. 실제 점유 결과가
없는 경로는 기존 미측정 경로로 남겨 정상 대조군에서 별도로 검증한다.

## 구현과 수정 전 진단

앞 source 8ddb3b169의 fresh WASM은 1074.56초에 exit 0으로 종료됐다.
고정 WASM SHA-256은
`fe33db6db11248fbf5fb98faf964ccb507c0db7733014dea3f1abe8065c768cd`이며,
JS는 `a043bdc9047d3ff5d89f55ccc21846f017ac109dad6de25b2885e43c44ee08da`다.
원래 source 파일 해시와 package/public 해시를 확인하고 owner·wrapper·wasm-opt가
종료된 뒤 수정 묶음을 적용했다. 이 고정 package의 시각 검증은 앞 소스의
증거로 실행하며 새 구현의 시각 완료로 대신하지 않는다.

수정 전 Native library로 원본 글꼴 전환 계약의 tokenizer/AST 진단을 실행했다.
RmOM·rMOM·Itx·iTx·BoLdX 5개는 정상 소문자 계약과 달라 exit 1이다.
rm/RM·공백 있는 Rm·일반 첨자·rmbar·times/sim 및 따옴표 문자열 대조군은
보존됐다. `stage58-font-prefix-contract-before.json`·log와 probe 원문에
출처·명령·SHA를 기록했다. 이 진단은 문서 전체 시각 검증을 뜻하지 않는다.

붙은 글꼴 전환의 원래 철자를 보존하면서 대소문자 혼용을 분리한다.
기존 tokenizer unit 모듈은 내용·검사 이름을 유지한 채 tokenizer/tests.rs로
옮겼다. tokenizer.rs는 1000줄 이하로 유지하며 기존 정확한 기대값을 바꾸지 않았다.

ParagraphLineBoxEnd는 실제 Full/PartialParagraph에서 생산한 글줄 점유 끝을
같은 호스트의 TAC Shape에서도 이어받는다. 다른 소유자·독립 제어 항목으로
바뀌면 이어받지 않는다. layout_column_item 뒤에서 이 결과를 다음 항목의
occupied_end에 전달한다. column scratch 측정과 실제 column 배치가 같은
호출 경로를 소비한다. EN_LINE_OWNER 로그로 생산·이어받은 끝을 대조한다.
가시 콘텐츠 하단과 빈 줄이 점유한 흐름 줄 하단은 여전히 별도로 보존한다.

owned_endnote_origin을 지역 결과로 보존하고 HeightCursor에 전달한다.
이 결과가 있으면 뒤의 세 저장 좌표 기반 gap 추정/축약 경로를 건너뛴다.
확인한 원점을 뒤 분기에서 다시 이동하지 않으며, 미측정·단 경계의 기존
경로는 이 결과의 통과로 대신하지 않는다. 새 source의 Native/fresh WASM·
소유 실행·정상 대조군은 다음 빌드에서 검증한다. 배치 회귀·fixture·baseline의
기대값은 이 구현 묶음에서 변경하지 않았다.

## 검증 실행 상태

구현 commit은 `de11af0c78010970fea549380b3eecfdf0eb1fc3`다.
source unit 정책은 기준 base `79abd9e49675f4286eaed38197f7e837afb744e7`을
명시해 exit 0으로 종료됐다. 검사 결과는 4198 tests / 298 modules이며
`stage58-unit-policy-check.json`과 log에 명령·base/head를 보존했다.
이 기준은 현재 고정한 로컬 base이며 최종 PR base가 바뀌면 다시 대조한다.

새 Native 빌드는 고정 target/pr-review에서 detached owner로 실행한다.
source 파일 해시·commit·명령은 `endnote-owner-contract-native-provenance.json`,
실제 owner는 `endnote-owner-contract-native-detached-owner.json`에 기록한다.
빌드 종료 뒤 owner의 바이너리 고정·해시를 기다리고, 집중 Native 검증과
fmt/Clippy를 수행한 뒤 fresh WASM을 순차 빌드하는 절차는
`stage58-validation-pipeline.json`에 남긴다. 아직 빌드·시각 완료는 아니다.

앞 source 8ddb3b169의 Native 집중 검증은 1851.17초에 8개 입력·37개 선택 쪽을
모두 측정하며 종료됐다. 11월 HWPX 19쪽과 9월 미주 사이 20 HWP의
19·21·23쪽이 미달이다. 나머지 6개 문서의 선택 범위 최저는 91.47413~97.60151%다.
전쪽 문서 통과로 확장하지 않는다. 같은 이전 source의 fresh WASM 집중 검증도
고정된 바이너리/package로 계속 실행한다. 새 구현 de11af0c7의 증거와 구분한다.

## 새 구현의 빌드와 실제 경계 결과

source de11af0c7의 Native 빌드는 449.21초에 exit 0으로 종료됐다.
고정 바이너리 SHA-256은
`ba98105a9d9be5bb69c9415847ce0b9d5da97953f4c10d75dac24d81dfbbdabb`다.
Native fmt는 27.02초, Native Clippy는 100.54초에 각각 exit 0으로 종료됐다.
같은 source의 fresh WASM은 이 종료 뒤에 순차 빌드 중이다.

같은 tokenizer/AST probe는 수정 후 exit 0, 실패 0개다. 수정 전 실패한
다섯 대소문자 혼용도 정상 소문자 계약과 같은 AST를 만든다. 따옴표 문자열은
여전히 독립 Quoted 노드이며 일반 rm/RM·첨자·times/sim도 유지된다.
`stage58-font-prefix-contract-after.json`과 log에 source·명령·원문 해시·결과를
기록했다. 이미 통과한 기대값을 바꾸지 않고 실제 파서를 보정했다.

두 입력의 19쪽 export-render-tree를 새 바이너리로 실행했고 둘 다 exit 0이다.
11월 HWPX는 가상 문단 917의 빈 줄 648.3~660.3을 유지한다.
뒤 TAC Shape의 EN_LINE_OWNER는 measured=NaN이지만 같은 호스트가 소비한
owned=660.320을 이어받는다. 다음 문27 제목 918은 690.5에 놓여 Print의
690.504와 대응한다. 앞 source의 672.5에서 실제 빈 줄 점유를 복원한 결과다.

9월 HWP의 가상 문단 974는 실제 글줄 668.7~727.5와 이미 소비한 미주 사이를
유지하며 다음 제목 975가 803.1에 놓인다. Print의 802.984와 대응한다.
실제 EN_RENDER도 974의 흐름 끝 712.4를 975의 시작으로 소비한다.
앞 source에서 저장 gap 55.16을 다시 더해 858.3이 되던 분기가 적용되지 않는다.

이 수치는 독립 PDF와 실행의 관측값이며 회귀 기대 좌표가 아니다.
소유·명령·출처는 `stage58-current-owner-diagnostic.json`, 실제 TextLine 관측은
`stage58-current-owned-line-observations.json`, 생산·소비 로그는 각 입력의
`stage58-current-*-p019-render-tree.log`에 보존했다. 영향 페이지·정상 대조군의
새 Native/fresh WASM 90% 이상 및 확대 판독은 계속 진행한다.

## 새 review 직접 판독

새 source de11af0c7의 Native SVG에서 독립 PDF 19쪽을 96dpi로 새로 산출해
직접 비교했다. 11월 HWPX는 81.85884%→98.00415%, 9월 미주 사이 20 HWP는
87.59770%→97.95371%로 개선됐다. 엄격 내용 픽셀은 각각 16.45283%·16.06705%다.
새 review를 모두 직접 표시했다. 11월은 그림 뒤 빈 줄의 점유가 다음 문27 제목·
그림·풀이에 보존되고, 9월은 문28 앞의 저장 간격 재가산이 사라져 Print의 위치로
이어진다. 각 `stage58-current-selected-direct/<id>/p019/metrics.json`에 원본·PDF·
SVG·바이너리 해시와 실제 raster·PDF 명령을 보존했다.

17쪽은 같은 새 source의 Native 직접 비교에서 97.58115%다. 수정 전 8ddb3b169·
수정 후 de11af0c7·Print를 같은 소유 영역 `[1,4)`과 좌표에서 3배 확대해
`stage58-current-nov-p017-font-prefix-detail-review.png`로 직접 표시했다.
`Rm` 문자가 사라졌고 정체 OM₁로 표시된다. 추가 글자 때문에 뒤의 분수와
본문이 겹치던 결함도 사라졌다. crop의 근거·소유·동일 좌표·배율은 동일 stem의
`-provenance.json`에 남겼다. 다른 수식·미세 간격 판독과 전체 페이지·fresh WASM은
계속 검증하며 이 세 쪽의 개선을 문서 전쪽 통과로 바꾸어 보고하지 않는다.

붙은 그리스 기호와 뒤 변수의 문법 경계를 추정으로 넓히지 않기 위해
16개 합성 대조 사례도 준비했다. 2024 한컴 저장은 성공했고 해당 저장본의
Print job을 모니터링 중이다. `stage58-greek-prefix-probe/input-plan.json`에
생성 방식·template/입력 해시와 사례 목록, `stage58-greek-prefix-probe-mcp-*.json`에
engine·제품·job·출력 해시를 보존한다. 합성 입력의 출력은 원래 검증 문서의
대체 기준으로 사용하지 않는다. Print method와 실제 출력은 아직 확인되지 않았다.
