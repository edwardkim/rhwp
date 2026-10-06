# Task #1270 Stage 1 완료보고서 — 진단 확정

## 범위

- 첨부 HWPX 내부에서 누락 이미지 `image2`의 위치 확인
- `BinData/image2.png` 정상 여부 확인
- 현재 rhwp SVG 출력에서 `image2`가 방출되지 않는 현상 재현
- 렌더 코드에서 캡션 문단의 인라인 이미지 경로가 끊기는 지점 확인
- Stage 2 구현 계획서에서 다룰 테스트 후보 정리

소스 코드는 수정하지 않았다.

## 첨부 HWPX 구조 확인

대상 파일:

```text
/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx
```

`Contents/section0.xml`에서 `image2`는 표의 TOP 캡션 내부 문단에 존재한다.

```text
<hp:tbl id="1490368361" ...>
  <hp:caption side="TOP" fullSz="0" width="8504" gap="850" lastWidth="47591">
    <hp:subList ...>
      <hp:p ... paraPrIDRef="95" ...>
        <hp:run charPrIDRef="88">
          <hp:pic id="1490368362" ... textWrap="TOP_AND_BOTTOM" ...>
            ...
            <hc:img binaryItemIDRef="image2" .../>
            <hp:pos treatAsChar="1" ... horzRelTo="COLUMN" .../>
```

확인 명령:

```bash
unzip -p '/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx' Contents/section0.xml \
  | xmllint --format - \
  | rg -n -C 8 'binaryItemIDRef="image2"|hp:caption side="TOP"|id="1490368362"'
```

`Contents/content.hpf`에도 `image2`는 `BinData/image2.png`로 선언되어 있다.

```text
<opf:item id="image2" href="BinData/image2.png" media-type="image/png" isEmbeded="1"/>
```

`BinData/image2.png` 자체도 정상 PNG다.

```text
Length: 7272 bytes
file: PNG image data, 232 x 37, 8-bit/color RGBA, non-interlaced
```

## 현재 렌더 출력 재현

현재 `local/task1270` 기준에서 1페이지만 SVG로 내보냈다.

```bash
cargo run --quiet --bin rhwp -- export-svg \
  '/private/tmp/issue1270/서울 문화예술단체 지원사업.hwpx' \
  -p 0 \
  -o /private/tmp/rhwp-task1270-stage1-svg
```

결과:

```text
문서 로드 완료: ... (56페이지)
내보내기 완료: 1개 SVG 파일 → /private/tmp/rhwp-task1270-stage1-svg/
```

생성된 1페이지 SVG의 `<image>` 요소 수는 1개다.

```bash
awk 'BEGIN{c=0} {s=$0; while (match(s, /<image/)) {c++; s=substr(s, RSTART+RLENGTH)}} END{print c}' \
  '/private/tmp/rhwp-task1270-stage1-svg/서울 문화예술단체 지원사업_001.svg'
```

```text
1
```

해당 이미지 태그는 다음 좌표와 크기다.

```text
<image x="69.90666666666667" y="109.79999999999998" width="200" height="118.4" .../>
```

이는 첫 셀의 `image1` 크기와 대응한다. 캡션 내부의 `image2` 크기(`hp:curSz width="16100" height="2645"`, 약 214.7px × 35.3px)는 SVG에 별도 `<image>`로 방출되지 않는다.

## 코드 원인 확인

`src/renderer/layout/picture_footnote.rs::layout_caption()`은 캡션 문단을 순회하며 `compose_paragraph()`를 호출한다.

```rust
for (pi, para) in caption.paragraphs.iter().enumerate() {
    let mut composed = compose_paragraph(para);
    ...
    para_y = self.layout_composed_paragraph(
        ...
        None,
        None,
        None,
        None, // 캡션 컨텍스트 — wrap zone 무관
    );
}
```

현재 문제 지점:

- `layout_caption()`은 실제 캡션 문단 `para`를 가지고 있다.
- 하지만 `layout_composed_paragraph()`의 마지막 인자 중 원본 문단 `para`와 `bin_data_content`를 `None`으로 전달한다.

반면 `src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph()`는 이 두 인자를 인라인 이미지 렌더링에 사용한다고 명시한다.

```rust
/// `para`: 원본 문단 (treat_as_char 이미지 인라인 렌더링에 사용)
/// `bin_data_content`: 이미지 데이터 (treat_as_char 이미지 인라인 렌더링에 사용)
```

실제 인라인 이미지 방출 분기도 두 값이 모두 `Some`일 때만 실행된다.

```rust
if let (Some(p), Some(bdc)) = (para, bin_data_content) {
    if let Some(ctrl) = p.controls.get(tac_ci) {
        if let Control::Picture(pic) = ctrl {
            ...
            let image_data = find_bin_data(bdc, bin_data_id).map(|c| c.data.clone());
            let img_node = make_picture_image_node(...);
            line_node.children.push(img_node);
            tree.set_inline_shape_position(...);
        }
    }
}
```

따라서 현재 구조에서는 캡션 문단의 `treat_as_char` 그림 컨트롤이 `compose_paragraph()`에 의해 TAC 후보로 인식되어도, 실제 `ImageNode` 생성 단계에서 원본 컨트롤과 바이너리 데이터에 접근할 수 없어 렌더링되지 않는다.

## 호출부 범위

`layout_caption()` 호출부는 다음과 같다.

```text
src/renderer/layout.rs:6752
src/renderer/layout/table_layout.rs:1022
src/renderer/layout/picture_footnote.rs:528
src/renderer/layout/table_partial.rs:1628
src/renderer/layout/table_partial.rs:1651
src/renderer/layout/table_partial.rs:1682
src/renderer/layout/shape_layout.rs:535
```

Stage 2에서는 `layout_caption()` 시그니처에 `bin_data_content`를 추가하고, 각 호출부에서 이미 보유한 바이너리 데이터 slice를 넘길 수 있는지 확인한다.

## 메인테이너 방향과 정합

메인테이너 코멘트에서 확정한 이번 범위는 다음이다.

- 캡션 내 `treat_as_char` 인라인 이미지 스레딩
- depth 1 한정
- 캡션 속 그림의 캡션은 렌더링하지 않음
- 플로팅 캡션 이미지는 후속 이슈 범위
- #1270은 이번 PR merge 후에도 close하지 않음

현 코드 진단은 이 방향과 정합한다.

`layout_composed_paragraph()`의 인라인 이미지 경로는 `ImageNode`를 직접 만들고 끝나며, 해당 그림의 caption을 다시 `layout_caption()`으로 넘기지 않는다. 따라서 `para` / `bin_data_content` 스레딩만 적용하면 자연스럽게 depth 1 한정이 유지된다.

## 테스트 후보

### 후보 A — 작은 HWPX fixture 추가

작은 HWPX 샘플을 추가해 다음 구조를 갖게 한다.

- 표 또는 그림에 TOP caption 존재
- caption 문단 안에 `treatAsChar="1"` 그림 1개
- `BinData/image1.png` 포함

테스트는 `HwpDocument::from_bytes()` → `build_page_render_tree(0)` → `RenderNodeType::Image` 수집으로 검증한다.

기대:

- 수정 전: 캡션 이미지 `ImageNode` 누락
- 수정 후: 캡션 이미지 `ImageNode` 1개 방출

### 후보 B — 기존 fixture 기반 보강

기존 `samples/hwpx` 중 캡션 내부 문단을 포함하는 샘플을 찾아 인라인 이미지 포함 여부를 확인한 뒤, 렌더 트리 검사만 추가한다.

장점은 새 샘플 추가가 없다는 점이다. 단점은 캡션 내 인라인 그림이라는 조건을 만족하는 기존 샘플이 없으면 사용할 수 없다.

### 후보 C — 첨부 HWPX 직접 추가

이슈 첨부 HWPX를 fixture로 추가해 재현 테스트를 작성한다.

이 파일은 크기가 크고 본 이슈의 대표 `image2`가 메인테이너 코멘트상 후속 플로팅 범위와 혼동될 수 있으므로 1차 후보로는 부적합하다.

## Stage 1 결론

원인은 `layout_caption()`이 캡션 문단을 `layout_composed_paragraph()`로 넘길 때 `para` / `bin_data_content`를 `None`으로 고정 전달하는 구조로 확정한다.

이번 작업의 구현 방향은 메인테이너 지시대로 다음이 적절하다.

1. `layout_caption()`에 `bin_data_content`를 전달할 수 있도록 시그니처를 확장한다.
2. 캡션 문단 루프 내부의 현재 `para`를 `layout_composed_paragraph()`에 `Some(para)`로 넘긴다.
3. `bin_data_content`도 `Some(bin_data_content)`로 넘긴다.
4. 플로팅 캡션 이미지는 이번 구현/테스트 성공 조건에서 제외한다.
5. 회귀 테스트는 첨부 대형 문서보다 작은 fixture 또는 기존 fixture 기반으로 `ImageNode` 방출을 검증하는 방식이 적절하다.

## 다음 단계

작업지시자 승인 후 `mydocs/plans/task_m100_1270_impl.md` 구현 계획서를 작성한다.
