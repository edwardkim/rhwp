# 줄 끝 구분 공백과 작성 공백의 소유

Issue: #4138. 부모 `4441bf1ee33d6609576657facf9cd5df559ff1f0`, base
`a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 공통 줄 구성의 종료 컷을 보정했다.
소스·실행 파일·WASM 해시와 실제 검사 결과는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/interword-space-20261107/validation.json`)에 있다.
이 단계는 브랜치 전체 완료 또는 PR 제출 가능 판정이 아니다.

## 독립 근거와 수정

실제 입력은 기존 `saved-split-1x2.hwp` SHA-256
`4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`이며,
기준은 기존 `pdf/issue4138-ascii-split-20261016-2024.pdf` 197쪽을 재사용했다.
21쪽 문단271의 `하며  다음에`에서 첫 구분 공백이 줄 폭을 넘으면 즉시 종료해 두 번째
작성 공백이 다음 줄로 넘어갔다. 독립 한컴2024 재저장에서는 마지막 줄 시작이 원문
offset61이며, 원본의 저장 offset60과 다르다. 독립 Print도 `다음에` 앞의 공백 없는 위치다.
기준을 후보 출력에서 추정하지 않고 재저장·PDF·원문 인덱스를 교차 확인했다.

맑은 고딕 TTF와 한양신명조 HFT의 독립 Print 36조건을 대조했다. 첫 구분 공백과 연속
작성 공백을 구분하며, 나머지 작성 공백은 이어받은 줄에서 계속 전진한다. 줄 끝 공백을
전부 제거하거나 일정 HU 공차를 늘리는 규칙은 이 대조군과 맞지 않는다.
입력·PDF·원점 출처는 fixture-provenance.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/interword-space-20261107/fixture-provenance.json`)에 보존했다.

`fill_one_interval`의 기존 종료 컷을 적용하기 전에, 현재 줄의 실제 낱말 뒤 첫 ASCII
공백·하나 이상의 추가 ASCII 공백·다음 실제 낱말이라는 연속 소유를 확인한다. 이 경우만
첫 종료를 다음 공백까지 미룬다. 일반 줄 구성과 구간 이어받기가 같은 결과를 소비한다.
문단 끝·명시 개행·선행 공백·개체 슬롯의 기존 정책과 oversized inline-object 분기는
변경하지 않았다. 문서·페이지 분기나 폰트 폭·숫자 공차 변경은 없다.
기존 #7171 및 #7568의 overflowing-space/문단 끝 공백 계약과 #7418 대조를 유지했다.

## 실제 검증

정식 검사 5건은 **수정 전 1 PASS·4 FAIL → 수정 후 5 PASS**다. 관계형 4건은 실제 줄
소속과 36조건의 작성 공백 전진을 검사한다. 고정형 1건은 독립 PDF 첫 `다`의
`(95.947998046875, 431.76812744140625)pt`를 192 DPI로 환산하여 x·baseline을
±2px로 보호한다. 이 쪽의 양쪽 raw가 98% 이상이고 직접 판독을 마친 뒤 추가했다.
기존 trailing-space·표 이어받기·인라인·기호·글상자 검사를 포함한 정식 집중 묶음은
**112 PASS**다. 초기 임시 harness의 잘못된 절대 경로 컴파일 실패는 통과 근거에서 제외했다.

| 경로 | 21쪽 수정 전 | 21쪽 수정 후 | 전197쪽 최저 | 98% 미만 |
| --- | ---: | ---: | ---: | --- |
| Native | 97.42188% | 99.97728% | 94.83034% | 44·129·130쪽 |
| fresh WASM | 97.39579% | 99.97745% | 94.98570% | 44·129·130쪽 |

192 DPI·compat2024·Print·PDFium·raw 2px 기준으로 전197쪽을 다시 산출했다.
누락 0쪽·쪽수 차이 0건·raw 하락 0쪽이다. 각 경로에서 21쪽만 PNG가 바뀌었고 나머지
196쪽은 부모와 바이트가 같다. Native의 자동 공급 캐시 TTF 바이트 차이는 같은 현재
환경·글꼴로 부모와 후보의 21쪽을 다시 출력해 분리했다. 각각 전쪽 PNG와 동일하며
공급 파일 목록·해시도 전후 동일하다. WASM 전쪽 공급 기록은 처음부터 같다.

| 독립 공백 대조군 | 조건 수 | Native | fresh WASM |
| --- | ---: | ---: | ---: |
| wide | 12 | 98.81412% | 100% |
| edge | 16 | 98.36366% | 100% |
| tight | 8 | 97.17452% | 100% |

각 대조군은 원본·PDF·출력 모두 1쪽이다. **tight Native는 98% 미달이며 미완료**다.
맑은 고딕 파일 2종을 각각 우선 공급한 진단에서도 같은 수치였다. 이를 코드 통과나 글꼴
예외로 면제하지 않는다. 이 대조군에는 절대 px 핀을 추가하지 않고 관계 검사만 적용했다.

fresh Native 빌드 169.322초·WASM 151.469초, fmt·workspace all-target Clippy·manifest
2119 sources/8942 static attrs·unit tier 4099건 검사를 통과했다. WASM은 Mac locked
wrapper의 `--target web --out-dir pkg --no-opt` 대체 빌드이며 Docker 최적화가 아니다.
실제 Studio 초기화 응답·브랜치 전체 nextest·최종 편람/백엔드 게이트는 아직 완료하지 않았다.

#8696에서 재사용한 정확한 3종과 글상자 독립 반례 5종도 같은 후보에서 재출력했다.
Native/fresh WASM 각 8건, 합계 16건이 모두 100%이며 기존 기능을 보존했다.

## 직접 판독과 자료

Native/fresh WASM의 같은 21쪽에서 3.4.3 마지막 줄 첫 낱말의 위치와 앞뒤 본문 소속을
확대해 확인했다. `≤`의 미세한 윤곽 차이와 별도 수식 44·129·130쪽 차이는 남아 있다.
남은 차이를 점수만으로 면제하거나 전체 범위 완료로 보고하지 않는다.

| 경로 | 수정 전 review | 수정 전 overlay | 수정 후 review | 수정 후 overlay |
| --- | --- | --- | --- | --- |
| Native | [review](before/native-p021-review.png) | [overlay](before/native-p021-overlay.png) | [review](after/native-p021-review.png) | [overlay](after/native-p021-overlay.png) |
| fresh WASM | [review](before/wasm-p021-review.png) | [overlay](before/wasm-p021-overlay.png) | [review](after/wasm-p021-review.png) | [overlay](after/wasm-p021-overlay.png) |

전쪽 원문: [이전 Native TSV](before/native-silhouette.tsv) · [현재 Native TSV](after/native-silhouette.tsv)
· [이전 WASM TSV](before/wasm-silhouette.tsv) · [현재 WASM TSV](after/wasm-silhouette.tsv).
미달 대조군: [Native review](controls/native-tight-review.png) · [Native overlay](controls/native-tight-overlay.png)
· [WASM review](controls/wasm-tight-review.png) · [WASM overlay](controls/wasm-tight-overlay.png).
실행 로그·임시 스크립트·재저장 진단 파일은 ignored `output/`에만 보존했다.
