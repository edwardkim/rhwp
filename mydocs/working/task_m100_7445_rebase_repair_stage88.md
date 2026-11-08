# Stage88: 이어받은 셀 조각의 안 여백과 후속 안내 표 복구

Issue: #7445. 부모 HEAD `544a2932d6e6f9c2146a7f16127a2d8d9e062c43`, 고정 base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 기존 50조건의 범위만 사용했다.
이 단계의 집중 결과이며 전체 회귀·전 50조건·실제 Studio 소비 검증 완료는 아니다.

## 원인과 공통 소비 경로

대상은 `samples/issue3637/press_release_split_cell_nested_table.hwpx`다. 확장자와 달리
실제 HWP5 CFB이며 한컴 2010 저장본이다. 원본 SHA-256은
`e2cb077d51293ae081a83e49c3bf2bca701087fddb4068d29295607b326be410`, 독립 한컴 2020
Print PDF는 `pdf/issue3637/press_release_split_cell_nested_table-hwpx-2020.pdf`,
SHA-256 `731ff37070a7a5fedbd239920323244aecd10d8913a1a49351956263e45bc2cc`다.
원본과 기준 PDF를 교체하지 않았다.

이전 예외는 #6813의 통합 커밋 `6a193a648dba3df6d5c4cffa0182bc02f3e011ff`에서
저장 Square 그림의 중복 높이·조기 이월을 보정하며 들어왔다. 원 PR의 검토·구현 기록과
#5920의 중첩 atom 꼬리 계약을 확인했다. 그 예외는 **행 어디든** 그림이 있으면
이어받은 모든 조각의 셀 안 여백을 0으로 계산했다. 실제 paint는 마지막 조각에도 안 여백을
사용하므로, 그림을 이미 소비한 뒤의 셀 상자·다음 내용 예약만 짧아졌다.

12쪽의 마지막 셀에는 그림이 없다. 원본 문단 218~224의 유닛 높이 합 835.64px를
그대로 셀 높이로 삼았지만 실제 마지막 줄은 상단 안 여백 141HU 아래에서 칠한다.
위·아래 141HU씩인 실제 조각의 안 여백 3.76px가 물리 높이에서 빠졌다.
별첨 본문의 기준선은 독립 PDF와 약 0.014px 차이로 이미 맞았고, 그 아래 안내 표가
약 2.6px 위에 있었다. 글꼴·본문 전체 이동·중첩 표 높이 보정으로 처리하지 않았다.

공통 `row_cut_height`와 `row_remaining_visible_cell_padding`에서 행 전체의 그림 유무로
여백을 제거하던 예외를 없앴다. 활성 컷의 보이는 내용 또는 유효한 빈 줄 상자가 소유한
안 여백을 기존 공통 계산으로 예약한다. `RowEntryQuery::padding` → `row_step`의
내용 예산·수용 높이와 `row_cut_content_height` → `table_partial`의 실제 셀 높이가
같은 결과를 소비한다. 특정 문서·쪽·셀 ID를 선택하거나 paint에 좌표를 덧대지 않았다.
중첩 atom 아래 보이지 않는 이송 간격을 fit에서 분리하는 기존 계약도 변경하지 않았다.

## 실제 시각 검증

96DPI Print, 현재 정본 PDFium 및 raw2px90%를 유지했다. Native release-test 빌드
109.913초, fresh WASM 204.373초는 각각 exit 0이다. WASM은 Mac locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화가 아니다.
루트 pkg와 Studio public의 JS/WASM hash가 일치한다. 브라우저 초기화 응답 검증은 별도다.

| 범위 | Native | fresh WASM |
| --- | ---: | ---: |
| 보도자료 전 12쪽 최저 |94.23037%|94.18582%|
| 보도자료 12쪽 전→후 |87.89806→99.25420%|87.84792→99.25121%|
| KTX 1쪽 |99.36494%|99.38975%|
| low-slack 1쪽 |99.99852%|99.99851%|
| 저장 줄 결측 1쪽 |98.02990%|98.02479%|
| 로드맵 선택153/295쪽 최저 |98.96693%|98.96600%|

보도자료는 양쪽 12/12쪽·누락 0·90% 미달 0이다. 전체 집중 10조건·38쪽에서
90% 미달이 없다. 로드맵은 쪽수 302를 확인했으나 선택 2쪽이며 전쪽 수용을 대신하지 않는다.
수정 전 12쪽은 Stage87의 동결 Native/WASM으로 새로 캡처했고, 해당 동결 producer의
production diff가 부모 HEAD와 같음을 검증했다. 과거 다른 source의 점수를 재사용하지 않았다.

8쪽의 두 중첩 상자와 마지막 문구, 9쪽의 그림·뒤 본문, 12쪽의 별첨·종료 행·하단 안내 표를
review/overlay로 직접 확인했다. 12쪽 Native/fresh WASM 모두 같은 내용·순서와 안내 표를
유지한다. 괘선 두께·글자 가장자리의 래스터 차이와 일부 미세 가로 차이는 남으며
픽셀 완전 일치를 주장하지 않는다.

[전후 TSV·producer·회귀 결과](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/continued-cell-padding/validation.json)
및 [Native review](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/continued-cell-padding/after-native-p012-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/continued-cell-padding/after-native-p012-overlay.png),
[WASM review](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/continued-cell-padding/after-wasm-p012-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/continued-cell-padding/after-wasm-p012-overlay.png)를 보존했다.

## 결함을 검출하는 정식 회귀

전 12쪽 양쪽 90% 및 12쪽 양쪽 99%·확대 판독을 확인한 뒤 기존
`issue_5920_nested_atom_tail_page_fit`에 관계 검사 하나를 추가했다.
마지막 본문 줄의 단일 소유·원본 안 여백까지 셀 내부 포함, 종료 표와 안내 표의 순서,
보이는 글자가 없는 후속 문단과 URL 내용을 확인한다. 독립 PDF에서 얻은 실제 위·아래
이중 가로선 중심 4개를 96DPI CSS 좌표 ±2px로 검사한다. Table bbox만 검사하지 않고
paint에 전달하는 Line을 대조하며, 후보 좌표로 기대값을 만들지 않았다.

같은 검사·기대값에서 수정 전 **3 PASS / 1 FAIL**(실행 0.181초), 수정 후
**4 PASS / 0 FAIL**(실행 0.182초)이다. 수정 전 실패는 마지막 줄과 원본 안 여백이
실제 조각 셀 안에 들어가지 않는다는 관계 위반이다. 기존 두 상자의 8쪽 소속,
9쪽 중복 방지, 4쪽 본문 초과 방지 검사는 양쪽 모두 통과했다. source는 정확히 복원했다.
실제 suite `--prepare` 후 `regression_suite_015`에서 4건을 실행했으며267건의 filter 제외를
실행 통과로 세지 않았다. 원시 로그와 임시 driver는 ignored output에만 보존한다.

## 남은 제출 조건

새 production source에서 기존 50조건 Native/fresh WASM 전쪽, 전체 nextest와 필수 lint/build/
Skia/manifest/unit tiers 및 실제 소비 경로를 계속 확인한다. 이전 head의 전체 통과를
이 단계의 전체 통과로 바꾸어 보고하지 않는다. 지침·입력·기준 PDF·문턱·기존 정상 검사 및
원격 브랜치는 변경하지 않았다.

`cargo fmt --all -- --check`, suite manifest `--check --base-ref d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`,
보고서 상대 링크 검사와 `git diff --check`도 각각 exit 0이다. 전체 필수 게이트의 완료 주장은 아니다.
