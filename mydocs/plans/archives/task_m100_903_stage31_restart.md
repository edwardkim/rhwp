# Task m100 #903 Stage31 Restart Plan

## 1. 목적

`samples/hwpx/hwpx-h-01.hwpx`의 HWP 저장 기능 작업을 Stage30 결론 기준으로 다시 정렬한다.

이번 재시작의 목표는 다음이다.

```text
Stage30에서 확정한 두 구현 항목만 기준선으로 고정한 뒤,
Stage31부터 한컴 파일 읽기 오류 문제를 다시 분석한다.
```

## 2. 현재 상태

현재 브랜치:

```text
task/m100-903-stage31-clean
```

현재 브랜치는 이전 조사 상태를 다음 커밋으로 보존한 뒤 분기했다.

```text
73e01003 chore: checkpoint task 903 investigation
```

작업트리는 깨끗하다.

## 3. Stage30에서 고정할 결론

Stage30에서 유효한 기준선으로 유지할 결론은 두 가지다.

```text
1. 마지막 9페이지 미출력
   - 원인: DocProperties.section_count 누락
   - 구현: HWPX -> HWP 저장 전 실제 section 개수로 보정

2. 표/셀 세로 배치 비정상
   - 원인: HWPX ParaShape margin 계열 값 누락
   - 구현: paraPr/margin 자식 요소형 값을 ParaShape margin 필드로 매핑
```

Stage30 구현 범위:

```text
- src/document_core/converters/hwpx_to_hwp.rs
  - DocProperties.section_count 보정
  - raw_data 제거 및 DocInfo 재직렬화 dirty 처리

- src/parser/hwpx/header.rs
  - paraPr/margin 속성형 파싱 유지
  - paraPr/margin 자식 요소형 파싱 추가
```

## 4. Stage30 범위 밖으로 분리할 것

다음 항목은 Stage30 결론의 직접 구현 범위가 아니므로 이번 재시작 기준선에서 분리한다.

```text
- Stage31~36 probe 문서와 출력 산출물
- table/object record materialization 실험 코드
- FileHeader/압축 probe
- Stage34/35/36 baseline 비교 probe
- serializer/control.rs의 table/object 호환성 실험성 변경
- section.rs의 별도 텍스트/entity/그림/표 실험성 변경
```

주의:

```text
XML entity 텍스트 보존(`<`, `>`, `&`)이나 embedded BinData 정규화처럼
이전 대화에서 중요하다고 판단된 항목이 있더라도,
Stage30 재시작 기준선과 섞지 않는다.
필요하면 Stage31 이후 별도 소단계로 다시 계획한다.
```

## 5. 작업 순서

승인 후 다음 순서로 진행한다.

### 5.1 소스 정리

```text
1. 현재 커밋의 소스 diff를 Stage30 필수 구현과 그 외 변경으로 분리한다.
2. Stage30 필수 구현만 남긴다.
3. Stage31~36 실험성 테스트 헬퍼와 probe 생성 테스트를 제거하거나 보류한다.
4. Stage30 검증에 필요한 테스트만 남긴다.
```

남길 테스트 축:

```text
- section_count 보정 검증
- ParaShape margin child 파싱 검증
- Stage31 clean adapter 산출물 생성 검증
```

### 5.2 문서 정리

```text
1. Stage30 문서는 기준선 문서로 유지한다.
2. Stage31 문서는 Stage30 구현 검증 및 파일 읽기 오류 발견 문서로 다시 작성한다.
3. Stage32 이후 문서는 새 계획 전까지 보류 표시하거나 새로 작성한다.
```

### 5.3 산출물 재생성

프로젝트 규칙에 따라 작업지시자 판정용 파일은 `output/` 아래에 생성한다.

새 Stage31 재검증 산출물:

```text
output/poc/hwpx2hwp/task903/stage31_restart/
```

예상 파일:

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

### 5.4 내부 검증

최소 내부 검증:

```text
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01 -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_stage31_restart_generate_impl_verify -- --nocapture
```

검증 항목:

```text
- DocProperties.section_count == document.sections.len()
- DocProperties.raw_data 제거
- DocInfo raw_stream_dirty 처리
- ParaShape margin child 값 파싱
- rhwp-studio 재로드 기준 9페이지 유지
```

### 5.5 작업지시자 판정 요청

작업지시자에게 다음을 판정 요청한다.

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

판정 항목:

```text
- 한컴 에디터 파일 읽기 오류/파일손상 여부
- 마지막 9페이지 출력 여부
- 표/셀 배치 정상 여부
- rhwp-studio 재로드 여부
```

## 6. 승인 후 하지 않을 것

승인 없이 다음 작업은 하지 않는다.

```text
- table/object record 직렬화 구현 추가
- serializer/control.rs 호환성 추정 수정
- Stage32 이후 새 probe 대량 생성
- 기존 결론과 섞이는 임의 구현
```

## 7. 성공 기준

이번 재시작 단계의 성공 기준은 한컴 최종 정상화가 아니라 기준선 재정렬이다.

```text
1. 소스가 Stage30 필수 구현만 포함하는 상태로 정리된다.
2. Stage31 산출물이 새 output 경로에 재생성된다.
3. 내부 검증 결과와 한컴 판정 결과가 Stage31 문서에 명확히 기록된다.
4. 한컴 파일 읽기 오류가 남는 경우, 그 문제를 Stage32의 단일 주제로 다시 계획한다.
```

