# #9022의 연속 그림 호스트 반례 보정

## 수정 전 실행과 출처

- 정확한 base `53774cc1508591b6ac794950f0063452fbaf6e87`와 호환 인자 보정 후보
  `959afe38afe282f2f179a232fe06e61c44b74fd2`를 각각 새로 빌드했다.
- 동일 입력·독립 Print PDF·96 DPI·compat 2024·원본 글꼴 환경에서 canonical Visual Sweep을
  실행했고 두 경로 모두 1쪽 raw 실루엣 **60.20161%**였다. 기존 base의 결함도 재현되므로
  #9022가 새로 만든 회귀라고 표현하지 않는다. 이 PR이 제안한 그림 뒤 글 배치의 경계 결함이다.
- reviewer의 공개 반례는 [postmelee의 고정 커밋](https://github.com/postmelee/rhwp/tree/4300c96b5919ea4e08c831c9693413a3364462c0/mydocs/pr/assets/issue_8695/pr9022-host-repro)에서 확보했다.
  `cell-two-square-picture-hosts.hwpx` SHA-256은
  `f9a54247bc0588c07b2fc4e03a48292c885568e93a5890743a15287267d25fbd`이다.
  둘째 그림만 제거한 `cell-second-host-without-picture.hwpx`의 SHA-256은
  `8c53a7c32a0fe70b3215c5d5f1e69368e6db318d5e988602d66b84e8c5024c9d`이다.
- 기준 `pdf/pr-review-all-open-20261116/cell-two-square-picture-hosts-hwpx-2024.pdf`는
  reviewer가 출력한 독립 한컴 2024 `13.0.0.3901` Print, 1쪽이다.
  managed direct DLL host, `frame_print_to_pdf_ex_one_up`, preprocess none,
  job `55bf659d-0e75-4b52-b199-47ab928c1a57`, SHA-256
  `dc2f185262f958a670db989ec50e844e354640afca8cbae41f678b660acedf4c`.
  웹한글기안기 PDF가 아니며 유효한 기존 Print를 재사용했다.

## 원인과 구현 후보

수정 전 큰 그림은 y 137.9..235.2px, 작은 그림은 y 137.9..153.9px이고 뒤 글 `가나다라`도
y 137.9px에 놓였다. 독립 Print는 작은 그림과 글을 큰 그림 아래에 놓는다.
`CellSquareStackTextFloor`가 호스트를 하나만 보관해 새 작은 그림 호스트가 앞 큰 그림을
덮어썼고, 새 그림 호스트 자체는 앞 장애물의 회피를 거치지 않고 반환했다.

보정 후보는 같은 칸의 앞 호스트를 보존하고 각 호스트 원점을 칸 좌표로 옮긴 그림 합집합에서
빈 글자 폭을 판단한다. 새 그림 호스트도 앞 장애물을 피한 뒤 자신의 원점을 등록한다.
앞 그림을 피해 내려간 빈 호스트 줄은 뒤 문단이 그 줄의 전진을 소비하도록 하며,
측정과 그리기는 기존 공용 `paragraph_top`을 계속 함께 사용한다.
문서명·문단 번호별 예외나 점수·기준 PDF 변경은 넣지 않는다.

보정 후 실제 빌드·Native/fresh WASM 출력·집중 회귀와 원 PR의 대조군을 확인하기 전에는
해결 또는 수용 완료로 판정하지 않는다. 영구 회귀 추가는 해당 쪽의 독립 Print 시각 선행 조건을
확인한 뒤 내용 순서·그림/글 겹침·칸 포함 관계로 보호한다.


## 최종 제품 보정의 집중 결과

제품 source `167921b3c5`의 Native/fresh WASM에서 두 그림 원본과 뒤 그림만 지운
대조군은 모두 100.00%다. 큰 그림 뒤로 작은 그림이 내려가고 이름표는 칸 안에 보존되며
END는 표 다음 본문에 남는 것을 직접 판독했다. 기존 5건과 새 관계형 2건은
최소 글꼴 환경에서 7건 모두 통과했다. 두 독립 PDF의 이름표 문자 상자는 동일하다.
작은 그림 높이는 빈 호스트 줄의 전진 안에 들어가므로 그림을 지워도 빈 문단까지
삭제한 것으로 처리하면 안 된다. 원본 그림 겹침 검사는 보정 전 실제로 실패했다.
