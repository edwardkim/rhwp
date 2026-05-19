# Task M100 #949 Stage 28 계획서 - HWPX renderingInfo의 SHAPE_COMPONENT materialization

## 1. 배경

Stage 27에서 `hwpx-h-03`, `hy-001`의 파일손상 지점을 정답 HWP와 비교했다.

확인된 사실은 다음과 같다.

```text
1. 문제 지점의 record sequence가 통째로 빠진 것은 아니다.
2. h03의 hp:pic@href -> CTRL_DATA record는 정답지와 위치/level/payload가 일치한다.
3. h03의 2페이지 글상자 주변은 CTRL_HEADER/SHAPE_COMPONENT/SHAPE_PICTURE sequence가 존재한다.
4. hy-001에서는 글상자 안 그림의 SHAPE_COMPONENT size가 정답 292 bytes, 생성본 196 bytes로 다르다.
```

현재 writer의 SHAPE_COMPONENT 렌더링 블록 크기와 대조하면 `196 -> 292` 차이는
`rendering cnt=1 -> cnt=2` 차이와 일치한다.

```text
cnt=1: ctrl_id 8 + ShapeComponentAttr 42 + rendering 146 = 196 bytes
cnt=2: ctrl_id 8 + ShapeComponentAttr 42 + rendering 242 = 292 bytes
```

따라서 다음 후보는 임의 tail 보정이 아니라, HWPX `<hp:renderingInfo>`를 HWP5
`SHAPE_COMPONENT` 렌더링 구조체로 정확히 materialize하는 것이다.

## 2. 문제 정의

현재 구현은 HWPX `renderingInfo`를 다음 방식으로 처리한다.

```text
1. transMatrix/scaMatrix/rotMatrix를 읽는다.
2. 모든 matrix를 하나의 affine 결과로 합성한다.
3. ShapeComponentAttr의 render_* 필드에 합성 결과만 저장한다.
4. HWP5 저장 시 raw_rendering이 없으면 cnt를 group_level > 0 여부로 추정한다.
```

이 방식은 렌더링 결과를 대략 재현하는 데는 도움이 되지만, HWP5 binary record contract에는
부족하다.

한컴 에디터는 HWPX XML을 HWP5 record에 직접 복사하기보다, XML 요소를 내부 C++ 구조체와
조판 객체로 매핑한 뒤 그 구조체를 HWP5 record로 serialize한다고 보는 것이 자연스럽다.
그러면 `renderingInfo`의 matrix 개수와 순서도 단순한 렌더링 힌트가 아니라
`SHAPE_COMPONENT` record envelope를 결정하는 입력이 된다.

## 3. 목표

Stage 28의 목표는 다음을 검증하고 구현 후보를 만든다.

```text
1. HWPX renderingInfo의 matrix sequence를 보존한다.
2. scaMatrix/rotMatrix 쌍 개수를 HWP5 SHAPE_COMPONENT rendering cnt로 사용한다.
3. HWP5 저장 시 group_level 추정이 아니라 source renderingInfo에서 계산한 raw_rendering을 우선 사용한다.
4. hy-001의 문제 SHAPE_COMPONENT가 정답처럼 292 bytes로 생성되는지 확인한다.
5. h03의 기존 196-byte child picture SHAPE_COMPONENT는 불필요하게 292 bytes로 바뀌지 않는지 확인한다.
```

## 4. 구현 후보

### 4.1 모델 확장 최소화

우선 기존 `ShapeComponentAttr.raw_rendering`을 활용한다.

```text
ShapeComponentAttr.raw_rendering
  = HWP5 SHAPE_COMPONENT rendering block bytes
  = u16 cnt + transMatrix + cnt개의 (scaMatrix, rotMatrix) pair
```

HWPX parsing 단계에서 `<hp:renderingInfo>`를 만났을 때, 합성 결과 `render_*`만 저장하지 않고
HWP5 writer가 그대로 쓸 수 있는 `raw_rendering`도 함께 만든다.

### 4.2 matrix count 규칙

HWPX source:

```xml
<hp:renderingInfo>
  <hc:transMatrix .../>
  <hc:scaMatrix .../>
  <hc:rotMatrix .../>
  <hc:scaMatrix .../>
  <hc:rotMatrix .../>
</hp:renderingInfo>
```

HWP5 target:

```text
cnt = sca/rot pair count
translation matrix 1개
scale/rotation matrix pair cnt개
```

`rotMatrix` 없이 `scaMatrix`만 끝나는 경우는 현재 parser와 동일하게 identity rotation을 보완한다.
`scaMatrix` 없이 `rotMatrix`가 나온 경우는 identity scale을 보완한다.

### 4.3 fallback

`renderingInfo`가 없는 개체는 기존 writer 경로를 유지한다.

```text
1. raw_rendering이 있으면 그대로 사용
2. HWPX renderingInfo에서 만든 raw_rendering이 있으면 그대로 사용
3. 없으면 기존 generated matrix 경로 사용
```

다만 Stage 28 구현 후에는 `group_level > 0`만으로 `cnt=2`를 추정하는 경로가 실제로 필요한지
별도 검증한다.

## 5. 검증 대상

생성 대상:

```text
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy-001.hwp
```

구조 검증:

```text
1. hy-001 문제 지점 SHAPE_COMPONENT size가 292 bytes로 바뀌는지 확인
2. h03의 child picture SHAPE_COMPONENT size가 정답처럼 196 bytes를 유지하는지 확인
3. h03/hy-001의 SHAPE_COMPONENT payload hash 변화와 정답지 근접 여부 확인
4. 기존 guard인 hwpx-h-01, hwpx-h-02가 Stage 18 성공 상태를 유지하는지 확인
```

시각 판정 요청표:

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-01.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-02.hwp` |  |  |  |  |  |  | guard |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-03.hwp` |  |  |  |  |  |  | target |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy-001.hwp` |  |  |  |  |  |  | #974 guard |

## 6. 성공 기준

1차 성공 기준:

```text
1. cargo build 성공
2. cargo test 성공
3. cargo fmt --check 성공
4. git diff --check 성공
5. 생성 HWP가 rhwp로 재로드 가능
6. hy-001의 292-byte SHAPE_COMPONENT 계약이 생성본에 반영됨
```

시각 성공 기준:

```text
1. hwpx-h-01, hwpx-h-02의 한컴 성공 상태 유지
2. hwpx-h-03 파일손상 지점이 뒤로 이동하거나 해소되는지 확인
3. hy-001이 #974 guard를 깨지 않는지 확인
```

## 7. 주의점

이번 단계는 `LIST_HEADER`/`PARA_HEADER` tail 차이를 동시에 해결하려고 하지 않는다.
Stage 23에서 header tail만 맞추는 접근은 충분하지 않다는 것이 이미 확인되었다.

따라서 Stage 28은 한컴 내부 구조체 매핑 가설에 맞춰, XML `renderingInfo`를 HWP5
`SHAPE_COMPONENT`의 렌더링 블록으로 정확히 컴파일하는 한 축만 처리한다.

이 계획 승인 후 소스 수정을 진행한다.
