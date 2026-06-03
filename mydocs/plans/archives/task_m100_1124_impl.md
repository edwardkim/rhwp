# 구현 계획서: Task M100-1124

## 1. 수정 대상

우선 수정 대상:

- `src/parser/hwpx/section.rs`

테스트 추가 후보:

- `src/parser/hwpx/section.rs` 테스트 모듈 또는 HWPX 파서 통합 테스트

필요 시 보조 유틸 확인:

- `src/parser/hwpx/utils.rs`
- `src/renderer/layout.rs`
- `src/renderer/page_layout.rs`

## 2. 구현 방침

### 2.1 `hp:colLine` 파싱

현재 `parse_col_pr(e: &BytesStart) -> ColumnDef`는 `colPr` 시작 태그의 속성만 읽는다. HWPX의 단 구분선 정보는 다음처럼 `colPr`의 자식 요소로 들어온다.

```xml
<hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="1" sameGap="850">
  <hp:colLine type="SOLID" width="0.12 mm" color="#000000"/>
</hp:colPr>
```

정정 방향:

- 기존 속성 파싱은 `parse_col_pr_attrs()`처럼 좁게 분리한다.
- `colPr`가 Empty 이벤트인 경우 기존처럼 속성만 파싱한다.
- `colPr`가 Start 이벤트인 경우 내부를 읽어 `colLine` Empty/Start를 찾고 `ColumnDef.separator_*`를 채운 뒤 `colPr` 종료를 소비한다.
- `parse_section_properties()`와 `parse_control_element()` 계열의 `skip_element(reader, b"colPr")` 중복 소비를 제거하거나 새 파서가 내부 소비를 책임지도록 맞춘다.

### 2.2 `colLine` 속성 매핑

예상 매핑:

- `type="SOLID"` → `separator_type = 1`
- `type="DASH"`/`DOT` 등은 기존 HWP line type 코드에 맞춰 매핑한다.
- `width="0.12 mm"` → 기존 `border_width_to_px()`가 해석 가능한 HWP width code로 변환한다.
- `color="#000000"` → `separator_color = 0x000000`

선 굵기 변환은 이미 HWPX border/fill 파서에 유틸이 있으면 재사용한다. 없으면 이번 증상에 필요한 최소 `0.12 mm` 매핑부터 추가하고 테스트로 고정한다.

### 2.3 검증

구현 후 다음을 확인한다.

```bash
cargo test --lib parser::hwpx -- --nocapture
cargo test --lib renderer::layout -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 3 --debug-overlay -o output/task1124_hwpx_after
rg -n 'x1="396\\.85333333333335".*x2="396\\.85333333333335"|stroke="#000000"' output/task1124_hwpx_after/3-11월_실전_통합_2022_004.svg
```

기대:

- HWPX page 4 SVG에 단 구분선 `<line>`이 생성된다.
- 구분선 위치가 HWP 렌더의 `x=396.853...`에 맞는다.
- HWPX 파서와 기존 layout 회귀 테스트가 통과한다.

## 3. 완료 조건

- `samples/3-11월_실전_통합_2022.hwpx` page 4에서 다단 세로 구분선이 표시된다.
- HWP의 기존 단 구분선 출력은 회귀하지 않는다.
- 수행/구현 계획서와 단계/최종 보고서가 갱신된다.

## 4. 승인 대기

소스 수정 전 승인 대기 상태다.
