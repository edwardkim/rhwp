# Task m100 #899 Stage 4 - 최종 회귀 확인 및 커밋 준비

## 1. 목적

Stage 3 작업지시자 판정 통과 이후 최종 회귀 테스트와 커밋 대상 범위를 확정한다.

## 2. 작업지시자 판정

Stage 3 산출물:

```text
output/poc/hwpx2hwp/task899/stage3/business_overview.hwp
```

판정 결과:

```text
통과
```

의미:

- 한컴/rhwp-studio 시각 판정 통과
- 셀 배경색 유지
- 셀 배경 무늬가 `무늬없음`으로 적용됨

## 3. 최종 테스트

### HWPX -> HWP 어댑터 전체 테스트

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
31 passed; 0 failed
```

### hatchStyle 매핑 유닛 테스트

```text
cargo test parser::hwpx::utils::tests::test_parse_hatch_style -- --nocapture
```

결과:

```text
ok
```

실행 중 기존 경고가 출력되었으나 이번 변경과 무관하다.

기존 경고:

- `duplicated attribute`
- `unused_parens`
- `non_snake_case`
- `unused_must_use`

### 공백 검사

```text
git diff --check
```

결과:

```text
통과
```

### WASM 빌드

```text
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
성공
```

빌드 로그 요약:

```text
[INFO]: :-) Done in 2m 07s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

산출물:

```text
pkg/rhwp.js
pkg/rhwp_bg.wasm
```

### rhwp-studio 최종 동작 판정

작업지시자 판정:

```text
동작 테스트 성공
```

## 4. 코드 변경 요약

변경 파일:

```text
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
src/parser/hwpx/utils.rs
tests/hwpx_to_hwp_adapter.rs
```

변경 규모:

```text
4 files changed, 83 insertions(+), 4 deletions(-)
```

핵심:

- OWPML `winBrush/@hatchStyle` -> HWP `pattern_type` 매핑 추가
- `hatchStyle`이 없는 HWPX solid fill은 `pattern_type=-1`로 정규화
- `business_overview.hwpx`의 셀 배경 BorderFill 5, 6, 7 회귀 테스트 추가

## 5. 커밋 대상

코드/테스트:

```text
src/parser/hwpx/header.rs
src/parser/hwpx/section.rs
src/parser/hwpx/utils.rs
tests/hwpx_to_hwp_adapter.rs
```

샘플:

```text
samples/hwpx/business_overview.hwpx
```

작업 문서:

```text
mydocs/working/task_m100_899_stage0.md
mydocs/working/task_m100_899_stage1.md
mydocs/working/task_m100_899_stage2.md
mydocs/working/task_m100_899_stage3.md
mydocs/working/task_m100_899_stage4.md
```

`output/` 산출물은 작업지시자 판정용이며, 저장소 추적 대상이 아니다.

## 6. 최종 판정

Stage 4 완료.

다음 단계는 커밋 및 PR 준비다.
