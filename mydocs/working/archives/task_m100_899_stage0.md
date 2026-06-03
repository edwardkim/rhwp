# Task m100 #899 Stage 0 - 셀 배경색 + 무늬없음 정규화 조사

## 1. 이슈

GitHub issue:

- https://github.com/edwardkim/rhwp/issues/899

제목:

- 셀 배경색 지정 시 배경 무늬없음 정규화 오류

## 2. 재현 샘플

작업지시자 제공 샘플:

```text
samples/hwpx/business_overview.hwpx
```

현재 브랜치에서 이 파일은 untracked 상태다. 작업지시자가 제공한 재현 샘플로 보고 Stage 1에서 추적 파일로 포함할지 결정한다.

## 3. HWPX 내부 확인

`business_overview.hwpx` 구조:

```text
Contents/content.hpf
Contents/header.xml
Contents/section0.xml
```

문제 축은 `Contents/header.xml`의 `hh:borderFills`와 `Contents/section0.xml`의 표 셀 `borderFillIDRef` 연결이다.

### 3.1 셀 참조

`section0.xml`에서 표 셀이 다음 BorderFill을 참조한다.

```text
borderFillIDRef="5"
borderFillIDRef="6"
borderFillIDRef="7"
borderFillIDRef="8"
```

확인된 용도:

- `5`: 올리브그린 배경 (대제목 번호 셀)
- `6`: 연한회색 배경 + 왼쪽 초록선 (대제목 텍스트 셀)
- `7`: 파랑 배경 (소제목 번호 뱃지)
- `8`: 하단선만 (소제목 텍스트 영역)

### 3.2 winBrush

`header.xml`에서 배경색이 있는 BorderFill은 다음 형태다.

```xml
<hc:winBrush faceColor="#DAEEF3" hatchColor="#999999" alpha="0"/>
<hc:winBrush faceColor="#7B8B3D" hatchColor="#999999" alpha="0"/>
<hc:winBrush faceColor="#F2F2F2" hatchColor="#999999" alpha="0"/>
<hc:winBrush faceColor="#4472C4" hatchColor="#999999" alpha="0"/>
```

특징:

- `faceColor`는 존재한다.
- `hatchColor`는 존재한다.
- `hatchStyle` 또는 무늬 종류를 나타내는 별도 속성은 없다.

## 4. 현 코드 관찰

### 4.1 HWPX header parser

파일:

```text
src/parser/hwpx/header.rs
```

현재 `winBrush` 파싱은 다음 필드만 처리한다.

- `faceColor` -> `SolidFill.background_color`
- `hatchColor` -> `SolidFill.pattern_color`
- `alpha` -> `Fill.alpha`

`SolidFill`은 `Default`로 생성되며, 모델 기본값은 다음과 같다.

```text
pattern_type = 0
```

따라서 HWPX에 `hatchStyle`이 없으면 `pattern_type=0`이 그대로 남는다.

### 4.2 HWP serializer

파일:

```text
src/serializer/doc_info.rs
```

`serialize_fill()`은 SolidFill을 HWP로 쓸 때 다음 순서로 기록한다.

```text
background_color
pattern_color
pattern_type
```

즉 HWPX 파서에서 `pattern_type=0`으로 남으면 HWP 저장 결과도 `pattern_type=0`이 된다.

### 4.3 기존 정합 선례

파일:

```text
src/document_core/html_table_import.rs
```

CSS 배경색으로 BorderFill을 생성하는 경로는 단색 배경에 대해 이미 다음 값을 사용한다.

```rust
pattern_type: -1_i32, // 무늬 없음
```

과거 PR #788 기록:

```text
mydocs/pr/archives/pr_788_review.md
```

표 셀 경로에서 `pattern_type` 의미를 잘못 해석하면 셀 배경이 검정 또는 패턴처럼 처리되는 문제가 있었다. 이 이슈는 렌더링 경로가 아니라 HWPX -> HWP 저장 직렬화 경로에서 유사한 의미 불일치가 재발한 것으로 본다.

## 5. 우선 가설

HWPX `winBrush`에 `faceColor`가 있고 `hatchStyle`이 없으면, HWP 저장용 IR에서는 다음처럼 정규화해야 한다.

```text
FillType::Solid
background_color = faceColor
pattern_color = hatchColor 또는 0
pattern_type = -1
```

즉 `pattern_type=-1`이 한컴 HWP 저장에서 “무늬없음”으로 해석되는 값이며, 현재 `0`으로 남는 것이 작업지시자 관찰 버그의 직접 원인일 가능성이 높다.

## 6. Stage 1 계획

Stage 1은 RED/확인 단계로 진행한다.

1. `business_overview.hwpx`를 테스트 fixture로 사용할지 확정한다.
2. HWPX 파싱 직후 `doc_info.border_fills[4..7]`의 `pattern_type`을 검사하는 RED 테스트를 작성한다.
3. 현재 값이 `0`임을 확인한다.
4. 기대값을 `-1`로 둔 테스트를 추가한다.
5. 아직 production 코드는 고치지 않는다.

## 7. Stage 2 후보 수정

Stage 1 RED가 확인되면 다음 중 더 좁은 곳을 수정한다.

우선 후보:

```text
src/parser/hwpx/header.rs
```

`winBrush` 파싱 시 무늬 종류 속성이 없으면 `solid.pattern_type = -1`로 초기화한다.

주의:

- 실제 `hatchStyle`/패턴 속성이 있는 HWPX 샘플에서는 해당 값을 보존해야 한다.
- `faceColor="none"`인 BorderFill까지 무리하게 SolidFill로 보존할지 여부는 별도 확인한다.
- #888의 문단/쪽 배경 정규화와 충돌하지 않아야 한다.

## 8. 인수 조건

- `business_overview.hwpx` 셀 배경색 유지
- 한컴 에디터에서 셀 배경 무늬가 `무늬없음`으로 표시
- HWPX -> IR -> HWP 저장 결과가 파일 손상 없이 열림
- `cargo test --test hwpx_to_hwp_adapter` 통과
- 기존 #888 샘플 `basic-table-01.hwpx`, `expense_report.hwpx` 회귀 없음
