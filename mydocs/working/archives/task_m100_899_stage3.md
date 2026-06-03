# Task m100 #899 Stage 3 - 작업지시자 판정용 산출물

## 1. 목적

Stage 2 GREEN 구현이 실제 HWP 저장 산출물에서 한컴/rhwp-studio 시각 판정 기준을 만족하는지 확인한다.

대상 샘플:

```text
samples/hwpx/business_overview.hwpx
```

## 2. 산출물

작업지시자 판정용 HWP:

```text
output/poc/hwpx2hwp/task899/stage3/business_overview.hwp
```

생성 경로는 프로젝트 규칙대로 `output/` 아래에 둔다.

## 3. 생성 방식

실제 저장 경로와 같은 `DocumentCore::export_hwp_with_adapter()`를 사용했다.

절차:

```text
business_overview.hwpx
-> DocumentCore::from_bytes()
-> export_hwp_with_adapter()
-> business_overview.hwp
-> DocumentCore::from_bytes() 자기 재로드
```

생성 로그:

```text
input=samples/hwpx/business_overview.hwpx
output=output/poc/hwpx2hwp/task899/stage3/business_overview.hwp
bytes=13312
reloaded_sections=1
reloaded_border_fills=8
```

파일 정보:

```text
size: 13K
sha256: 117750eae1cd61d6fd5a8e4cf2d0588c95d697552b1bc7f406e6ed9d8e109cff
```

## 4. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 열어 확인한다.

```text
output/poc/hwpx2hwp/task899/stage3/business_overview.hwp
```

판정 항목:

- 한컴 에디터에서 파일 손상 판정 없이 열리는지
- rhwp-studio에서 다시 열리는지
- 셀 배경색이 유지되는지
- 셀 배경 무늬가 `무늬없음`으로 적용되는지
- 특히 원본 HWPX의 BorderFill 5, 6, 7에 해당하는 색상 셀에서 불필요한 무늬가 보이지 않는지

## 5. 현재 자동 검증 상태

통과:

```text
cargo test --test hwpx_to_hwp_adapter task899_business_overview_cell_backgrounds_use_no_pattern -- --nocapture
cargo test --test hwpx_to_hwp_adapter
cargo test parser::hwpx::utils::tests::test_parse_hatch_style -- --nocapture
```

결과 요약:

```text
task899 RED -> GREEN
hwpx_to_hwp_adapter: 31 passed; 0 failed
hatchStyle mapping test: ok
self reload: ok
```

## 6. 다음 단계

작업지시자 판정 결과에 따라 Stage 4를 결정한다.

예상 분기:

- 통과: 최종 테스트/문서 정리 후 커밋 준비
- 실패: 한컴/rhwp-studio 증상 기준으로 `pattern_type`, `faceColor`, `hatchColor`, alpha 처리 중 어느 필드가 문제인지 추가 probe 작성

## 7. 작업지시자 판정 결과

판정일:

```text
2026-05-14
```

결과:

```text
통과
```

판정 내용:

- 한컴/rhwp-studio 시각 판정 통과
- 셀 배경색과 배경 무늬없음 정규화가 의도대로 동작

다음 Stage 4는 최종 회귀 테스트, 문서 정리, 커밋 준비 단계로 진행한다.
