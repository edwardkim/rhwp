# Task M100 #949 Stage 28: renderingInfo -> SHAPE_COMPONENT raw_rendering 구현 후보

## 1. 목적

Stage 27에서 확인한 핵심 단서는 `hy-001`의 글상자 안 그림 `SHAPE_COMPONENT` 크기 차이다.

```text
정답 HWP: 292 bytes
Stage 26 생성본: 196 bytes
```

이 차이는 HWP5 `SHAPE_COMPONENT` rendering block의 matrix count가 `1 -> 2`로 달라질 때의
96 bytes 차이와 일치한다. 따라서 Stage 28에서는 HWPX `<hp:renderingInfo>`를 렌더링 결과만
합성하지 않고, HWP5 `SHAPE_COMPONENT` raw rendering block으로 materialize하는 후보를 구현했다.

## 2. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
```

구현 규칙:

```text
1. hp:renderingInfo의 transMatrix를 HWP5 rendering block의 기준 trans matrix로 사용한다.
2. scaMatrix/rotMatrix 쌍 개수를 HWP5 rendering count로 사용한다.
3. rotMatrix 없이 scaMatrix만 끝나면 identity rotation을 보완한다.
4. scaMatrix 없이 rotMatrix가 나오면 identity scale을 보완한다.
5. 생성된 raw bytes를 ShapeComponentAttr.raw_rendering에 저장한다.
6. HWP5 writer는 raw_rendering이 있으면 기존 fallback 추정보다 이를 우선 사용한다.
```

추가 테스트:

```text
parser::hwpx::section::tests::test_rendering_info_materializes_hwp5_raw_rendering_count
```

테스트는 `transMatrix + 2개의 sca/rot pair` 입력에서 다음을 검증한다.

```text
raw_rendering length = 2 + 48 + 2 * 96
raw_rendering[0..2]의 count = 2
matrix 값이 f64 little-endian으로 보존됨
```

## 3. 생성 결과

생성 파일:

```text
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-02.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-03.hwp
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy-001.hwp
```

rhwp 재로드 결과:

| file | size | rhwp reload |
|---|---:|---|
| `hwpx-h-01.hwp` | 374,784 bytes | ok, sections=2, pages=9 |
| `hwpx-h-02.hwp` | 32,256 bytes | ok, sections=2, pages=10 |
| `hwpx-h-03.hwp` | 38,400 bytes | ok, sections=2, pages=9 |
| `hy-001.hwp` | 88,576 bytes | ok, sections=1, pages=2 |

진단 산출물:

```text
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/h03_stage28_section0.jsonl
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy_stage28_section0.jsonl
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/h03_shape_bundles_w8.md
output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy_shape_bundles_w8.md
```

## 4. 구조 검증

### 4.1 `hy-001`

`hy-001`의 핵심 후보였던 `SHAPE_COMPONENT#21`은 정답 HWP와 동일하게 292 bytes가 되었다.

| record | oracle | Stage 26 | Stage 28 | 판정 |
|---|---:|---:|---:|---|
| `SHAPE_COMPONENT#21` | 292 | 196 | 292 | rendering count 크기 회복 |

다만 payload hash는 아직 정답과 다르다.

```text
oracle hash:    4a59f707df17ea84
generated hash: e7661c58f937ee4e
```

또한 주변 header 크기 차이는 남아 있다.

| record | oracle | Stage 28 | 판정 |
|---|---:|---:|---|
| `CTRL_HEADER#20` | 46 | 46 | size 동일, payload 다름 |
| `LIST_HEADER#23` | 47 | 34 | size 다름 |
| `PARA_HEADER#24` | 24 | 22 | size 다름 |

해석:

```text
renderingInfo -> raw_rendering materialization은 hy-001의 292-byte 축을 회복했다.
하지만 파일손상 여부가 해소되는지는 한컴 판정이 필요하다.
만약 여전히 파일손상이라면 다음 축은 LIST_HEADER/PARA_HEADER tail 또는 GenShape payload다.
```

### 4.2 `hwpx-h-03`

`hwpx-h-03`의 글상자 내부 child picture component는 정답처럼 196 bytes를 유지했다.
즉 Stage 28 변경이 모든 그림을 무조건 292 bytes로 부풀리는 문제는 만들지 않았다.

| record | oracle | Stage 28 | 판정 |
|---|---:|---:|---|
| `SHAPE_COMPONENT#832` | 196 | 196 | size 유지 |
| `SHAPE_COMPONENT#836` | 196 | 196 | size 유지 |
| `CTRL_DATA#833` | 76 | 76 | payload 일치 |

남은 차이:

| record | oracle | Stage 28 | 판정 |
|---|---:|---:|---|
| `CTRL_HEADER#824` | 60 | 60 | size 동일, payload 다름 |
| `SHAPE_COMPONENT#825` | 252 | 252 | size 동일, payload 다름 |
| `LIST_HEADER#826` | 33 | 20 | size 다름 |
| `PARA_HEADER#827` | 24 | 22 | size 다름 |
| `CTRL_HEADER#831` | 176 | 176 | size 동일, payload 다름 |
| `SHAPE_PICTURE#834` | 91 | 91 | size 동일, payload 다름 |
| `CTRL_HEADER#835` | 46 | 46 | size 동일, payload 다름 |
| `SHAPE_PICTURE#837` | 91 | 91 | size 동일, payload 다름 |

해석:

```text
hp:pic@href -> CTRL_DATA 계약은 계속 정답과 일치한다.
renderingInfo count 축도 child picture에는 과잉 적용되지 않았다.
따라서 h03 파일손상이 유지된다면 원인은 outer GenShape/List/Para header 또는 payload 쪽으로 좁혀진다.
```

## 5. 한컴 시각 판정 요청

| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-01.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-02.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | guard |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hwpx-h-03.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 글상자 전까지만 출력, 실패 | 1페이지 페이지네이션 실패 | target |
| `output/poc/hwpx2hwp/task949/stage28_rendering_info_contract/hy-001.hwp` | 파일손상 | 1페이지만 | 성공 | 성공 | 2페이지 마지막 표전까지만 출력, 실패 | 정상 | #974 guard |

작업지시자 판정:

```text
이전과 동일한 결과
```

즉 Stage 28은 `hy-001`의 `SHAPE_COMPONENT#21` size를 정답처럼 292 bytes로 회복했지만,
한컴 파일손상 판정을 개선하지 못했다.

## 6. 실행한 검증

```text
cargo fmt --check
cargo test parser::hwpx::section::tests::test_rendering_info_materializes_hwp5_raw_rendering_count --lib
cargo build
cargo test
git diff --check
```

검증 결과:

```text
모두 통과
```

`cargo test` 결과 요약:

```text
lib tests: 1299 passed, 0 failed, 2 ignored
integration/doc tests 포함 전체 통과
```

기존 경고는 이번 변경과 무관한 기존 경고다.

## 7. 현재 결론

Stage 28은 `renderingInfo`를 HWP5 `SHAPE_COMPONENT` raw rendering block으로 컴파일하는 축을
구현했다.

확정된 것:

```text
1. HWPX renderingInfo의 sca/rot pair count는 HWP5 SHAPE_COMPONENT size에 영향을 준다.
2. hy-001의 196 -> 292 크기 차이는 이 축으로 회복된다.
3. h03의 child picture SHAPE_COMPONENT는 196 bytes를 유지하므로 과잉 보정은 아니다.
```

한컴 판정으로 추가 확정된 것:

```text
1. renderingInfo -> raw_rendering materialization만으로는 h03/hy-001 파일손상이 해소되지 않는다.
2. h03의 child picture CTRL_DATA, child picture SHAPE_COMPONENT size, hy-001 picture SHAPE_COMPONENT size는
   각각 필요한 축이지만 충분조건이 아니다.
3. Stage 23에서 #826/#827 LIST_HEADER/PARA_HEADER를 정답 size/hash로 맞췄어도 h03 파일손상이
   남았으므로, 다음 후보를 header tail 단독으로 되돌리면 안 된다.
```

다음 단계의 우선순위:

```text
1. 한컴이 멈추는 h03 2페이지 글상자 직전 outer GenShape를 기준점으로 잡는다.
2. #824 CTRL_HEADER, #825 SHAPE_COMPONENT, #831 child CTRL_HEADER, #832 child SHAPE_COMPONENT,
   #834 SHAPE_PICTURE payload를 정답지와 field 단위로 해독한다.
3. XML 요소를 한컴 내부 구조체로 매핑한다는 가설에 맞춰,
   HWPX source element -> IR field -> HWP5 record field 연결표를 만든다.
4. 연결표 없이 새 후보를 생성하지 않는다.
```
