# 수식 첨자와 정렬 칸의 독립 경계 대조군

Issue: #7642

정식 회귀 fixture가 아닌 구조 진단 자료다. 정상 한컴 문서의 스타일·구역을 재사용한 HWPX의 크기·baseline·LineSeg를 비우고 한컴 Office 2020에서 HWP로 정상 저장한 뒤, 그 HWP를 Print했다. 엔진은 `11.0.0.9136`, `PrintMethod=0` (1-up)이다. RHWP 재저장 PDF가 아니다.

공식 [수식 형식 1.3](https://cdn.hancom.com/link/docs/%ED%95%9C%EA%B8%80%EB%AC%B8%EC%84%9C%ED%8C%8C%EC%9D%BC%ED%98%95%EC%8B%9D_%EC%88%98%EC%8B%9D_revision1.3.pdf)의 항·그룹·칸 맞춤·첨자 설명과 함께 대조한다. 특정 화면 좌표를 구현 조건으로 사용하지 않는다.

## 첨자 소속: 21쪽

- 정상 저장 job: `af7fab28-9b2e-43db-9876-7c6784f55f5a`
- Print job: `aa584d54-7fb4-4ba4-acf3-2b2cd54432d8`

- 1쪽 `root_lsub`: `SQRT{x} LSUB i`
- 2쪽 `group_root_lsub`: `{SQRT{x}} LSUB i`
- 3쪽 `root_lsup`: `SQRT{x} LSUP i`
- 4쪽 `group_root_lsup`: `{SQRT{x}} LSUP i`
- 5쪽 `integral_lsub`: `INT LSUB i`
- 6쪽 `group_integral_lsub`: `{INT} LSUB i`
- 7쪽 `integral_sub`: `INT SUB i`
- 8쪽 `left_then_right`: `x LSUB i SUB j`
- 9쪽 `right_then_left`: `x SUB j LSUB i`
- 10쪽 `grouped_blank_prefix`: `{LSUB{i}{A}} + b`
- 11쪽 `pile_bad_first`: `PILE{LSUB{i}{A} # b}`
- 12쪽 `bad_right_prefix`: `SUB{i}+b`
- 13쪽 `bad_sup_prefix`: `SUP{i}+b`
- 14쪽 `words_left`: `abc LSUB ij`
- 15쪽 `group_add_left`: `{a+b} LSUB i`
- 16쪽 `add_left`: `a+b LSUB i`
- 17쪽 `left_rhs_group`: `x LSUB {a+b}`
- 18쪽 `left_rhs_nested`: `x LSUB {y LSUP z}`
- 19쪽 `right_rhs_group`: `x SUB {a+b}`
- 20쪽 `left_missing`: `x LSUB`
- 21쪽 `left_then_sup`: `x LSUB y^2`

## 정렬 칸: 6쪽

- 정상 저장 job: `047b0a96-a98f-4475-b924-c08b0d95fe83`
- Print job: `9c75f788-7f2e-4f8c-9ad3-2f1c72a0dacf`

- 1쪽 `no_marks`: `EQALIGN{a = 1 # abc = 2}`
- 2쪽 `trailing_marks`: `EQALIGN{a & # abc &}`
- 3쪽 `first_without_mark`: `EQALIGN{a # abc &= 2}`
- 4쪽 `last_without_mark`: `EQALIGN{a &= 1 # abc}`
- 5쪽 `empty_left_mark`: `EQALIGN{&= 1 # abc &= 2}`
- 6쪽 `mixed_boundary_gap`: `EQALIGN{{1} OVER {2} &= 1 # abc &= 2}`

## 연산자 템플릿: 8쪽

- 정상 저장 job: `d5fff5f2-561b-475a-8073-eeee57e6e917`
- Print job: `801629ac-334d-4f64-b8c4-650c54c0a76c`

- 1쪽 `sum_left`: `SUM LSUB i`
- 2쪽 `sum_left_sup`: `SUM LSUP i`
- 3쪽 `group_sum_left`: `{SUM} LSUB i`
- 4쪽 `prod_left`: `PROD LSUB i`
- 5쪽 `int_left_sup`: `INT LSUP i`
- 6쪽 `int_left_body`: `INT x LSUB i`
- 7쪽 `a_plus_right`: `a+b SUB i`
- 8쪽 `a_plus_sup`: `a+b SUP i`

## 독립 관측과 판정 범위

- 모든 행에 &가 없으면 각 행은 왼쪽 원점을 유지한다. &가 하나라도 있으면 첫 칸도 공유 열로 맞추며, & 뒤가 빈 칸인 행과 표시 없는 행을 원문 정보로 구분한다.
- `SQRT{x} LSUB i`는 근호의 인자에 붙고, `{SQRT{x}} LSUB i`는 바깥 그룹에 붙는다.
- 큰 연산자 이름 바로 뒤의 왼쪽 marker와 전체 연산자를 감싼 그룹의 왼쪽 marker는 다르다. bare SUM/PROD/INT에서는 왼쪽 글자가 표시되지 않고 바깥 그룹은 표시된다. SUB는 오른쪽 슬롯을 사용한다.
- 왼쪽·오른쪽 슬롯의 입력 순서를 바꾸어도 본체와 각 첨자의 소속이 같다. 묶이지 않은 `x LSUB y^2`는 왼쪽 y와 본체 오른쪽 위 2를 표시한다. `{y^2}` 그룹은 별도 인자다.
- 선행 항이 없는 명령형 첨자는 문법 오류다. 중첩 그룹이나 PILE에서 발생한 오류도 독립 Print에서는 전체 수식이 표시되지 않는다. 원본 스크립트는 보존하고 정상 문법을 위한 기대값과 구분한다.

이 관측은 실제 출력 개선의 근거이며, Native/fresh WASM 전쪽·확대 검증을 대체하지 않는다. 미달 상태에서는 새 회귀나 golden 기대값을 추가/갱신하지 않는다. Stage 182 구조 Print의 정렬 불일치도 다른 자료로 대체해 통과했다고 보고하지 않는다.

## SHA-256

- `alignment-column-hancom-2020.hwp`: `eb71e976a22b8f33ad052e4ba3a2475de5b65734d6114f88211b08267766ee3d`
- `alignment-column-hancom-2020.pdf`: `900e14f2e9ac8edcbc62ee6c8f97b52e7e53dfb7a6ce5430ba0b3bb4e1a3eef9`
- `alignment-column-input.hwpx`: `5baf9edebe45cfd842902c3edb448fb6eadfee4f9ae827250d69654d92c4d111`
- `operator-script-hancom-2020.hwp`: `323478548c44105a68684d752e4a381920570611b306be8784d9e2b1b7d7d1b1`
- `operator-script-hancom-2020.pdf`: `4c01c7d9cb85085221ae8ca25c7a6a078e4af8f3c87e4bd81dccd10f3edb2e2d`
- `operator-script-input.hwpx`: `a75703f3f0025f0bfe1baaac1e410b4c0bc5b6b5c44166fb415f380cc498deae`
- `script-ownership-hancom-2020.hwp`: `e39f9859993427ead20109fc40662574f9f507dde9e5369cd4749749cbe3f768`
- `script-ownership-hancom-2020.pdf`: `ce6b69f435924e323915adea6f955c4d1f3173308d2cbd26dcf9b2d21e4fa476`
- `script-ownership-input.hwpx`: `5d722805ad1af2c05379491d5e0d80a596dcfade4fef1de3c4d7e8d52486fdc4`
