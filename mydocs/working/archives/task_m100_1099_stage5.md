# Task M100-1099 Stage 5 작업 기록

## 1. 목적

Stage 4에서 `exam-kor-1p`의 파일손상과 문단 경계선 문제가 해결되었다.

이번 단계는 같은 수정이 다음 샘플에도 유지되는지 확인하기 위한 확장 검증 산출물을 만든다.

```text
samples/hwpx/exam-kor-2p.hwpx
samples/hwpx/exam-kor-3p.hwpx
samples/hwpx/exam-kor-4p.hwpx
samples/hwpx/exam_kor.hwpx
```

## 2. 적용된 핵심 contract

이번 확장 검증은 Stage 3과 Stage 4에서 확정한 두 축을 포함한다.

```text
1. LAST_PAGE master page는 SectionDef child가 아니라 BodyText tail의 level-1 LIST_HEADER로 저장한다.
2. HWPX hh:border/@connect는 HWP5 PARA_SHAPE attr1 bit 28로 저장한다.
```

## 3. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task1099/stage5_multisample_regression/
```

생성 파일:

| file | size | rhwp reload |
|---|---:|---|
| `exam-kor-2p.hwp` | 687,104 bytes | ok, sections=1, pages=2 |
| `exam-kor-3p.hwp` | 690,176 bytes | ok, sections=1, pages=3 |
| `exam-kor-4p.hwp` | 692,736 bytes | ok, sections=1, pages=4 |
| `exam_kor.hwp` | 6,979,584 bytes | ok, sections=3, pages=20 |

## 4. 한컴 판정 요청

| file | 한컴 판정 유형 | 문단 경계선 | 바탕쪽 출력 | 지문 박스 출력 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage5_multisample_regression/exam-kor-2p.hwp` |  |  |  |  |  |  | 2페이지 축소 |
| `output/poc/hwpx2hwp/task1099/stage5_multisample_regression/exam-kor-3p.hwp` |  |  |  |  |  |  | 3페이지 축소 |
| `output/poc/hwpx2hwp/task1099/stage5_multisample_regression/exam-kor-4p.hwp` |  |  |  |  |  |  | 4페이지 축소 |
| `output/poc/hwpx2hwp/task1099/stage5_multisample_regression/exam_kor.hwp` |  |  |  |  |  |  | 전체 샘플 |

## 4.1 웹 wasm 판정

작업지시자 판정:

```text
rhwp-studio에서는 samples/hwpx/exam-kor-4p.hwpx 까지는 저장 후 한컴 에디터에서 정상 로드된다.
```

의미:

```text
1. Stage 3/4에서 반영한 master page tail contract와 paragraph border connect contract는
   웹 wasm 저장 경로에서도 4페이지 축소 샘플까지 유효하다.
2. 다음 확인 대상은 전체 samples/hwpx/exam_kor.hwpx 저장 결과다.
3. 전체 샘플에서만 문제가 발생하면 4p 이후 구간의 SectionDef, master page tail,
   BodyText tail, 또는 추가 control contract를 비교한다.
```

## 5. 실행한 검증

Stage 4 구현 후 다음 검증을 통과했다.

```text
cargo fmt --check
cargo test document_core::converters::hwpx_to_hwp::tests::header_footer_nested_tables_are_materialized
cargo build --bin rhwp
```

Stage 5에서는 생성된 HWP를 `rhwp info`로 다시 로드해 페이지 수를 확인했다.

```text
exam-kor-2p.hwp: sections=1, pages=2
exam-kor-3p.hwp: sections=1, pages=3
exam-kor-4p.hwp: sections=1, pages=4
exam_kor.hwp: sections=3, pages=20
```

## 6. 다음 판단

```text
1. 네 파일이 모두 한컴에서 정상으로 열리면 #1099의 핵심 파일손상 contract는 해결로 본다.
2. 문단 경계선/바탕쪽/지문 박스/마지막 페이지 출력 중 잔여 차이가 있으면 해당 축만 별도 stage로 분리한다.
3. 전체 exam_kor에서만 문제가 발생하면 축소 샘플과 전체 샘플의 Section/BodyText tail 차이를 비교한다.
```
