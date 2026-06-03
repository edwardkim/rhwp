# Task M100-949 Stage 15 Plan

## 목적

Stage 13/14에서 확인한 `hwpx-h-03`의 누락 계약을 실제 HWPX -> HWP 저장 경로에 반영한다.

핵심 계약:

```text
HWPX hp:pic@href
  -> IR Picture.href
  -> HWP CTRL_DATA
     ParameterSet ps_id=0x021b
       item id=0x026f type=ParameterSet
         ParameterSet ps_id=0x026f
           item id=0x0265 type=String
             "http\\://www.korea.kr;1;0;0;"
```

## 작업 범위

1. `Picture` 모델에 HWPX `href` 보존 필드를 추가한다.
2. HWPX parser가 `<hp:pic href="...">` 값을 읽어 IR에 저장한다.
3. HWPX serializer가 IR의 `Picture.href`를 다시 `<hp:pic href="...">`로 보존한다.
4. HWPX -> HWP adapter가 `Picture.href`를 한컴 정답지와 같은 `CTRL_DATA` payload로 materialize한다.
5. nested shape text box paragraph 안의 picture control까지 재귀 처리한다.
6. Picture/Shape `CTRL_DATA`는 `SHAPE_COMPONENT` 자식으로만 출력되도록 serializer 위치 계약을 확인한다.
7. 단위 테스트로 76바이트 ParameterSet 구조와 `hwpx-h-03` 샘플 materialization을 확인한다.

## 비목표

- 표 배치 attr/tail 계약을 새로 추론하지 않는다.
- `CTRL_DATA`를 모든 그림에 임의 생성하지 않는다.
- HWP 출처 roundtrip 동작을 변경하지 않는다.
