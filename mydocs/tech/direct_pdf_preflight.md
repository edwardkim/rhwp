---
kind: reference
status: active
canonical: mydocs/tech/direct_pdf_preflight.md
last_verified: 2026-11-06
---

# Direct PDF의 정적 사전검사 계약

Issue: #536, P80. `native-skia`를 활성화한 Native 빌드의
`renderer::pdf::preflight_direct_pdf`는 선택한 `PageLayerTree`들의 정적 거부 사유를
폰트 관리자나 PDF canvas를 생성하기 전에 조회한다. 실제 앱의 내보내기 UI는 이 API를
소비할 수 있지만 이 변경에는 앱 구현을 포함하지 않는다.

```rust
use rhwp::renderer::pdf::{
    preflight_direct_pdf, DirectPdfExportOptions, DirectPdfPreflightLimits,
};

let report = preflight_direct_pdf(
    pages,
    &DirectPdfExportOptions::default(),
    DirectPdfPreflightLimits::default(),
);
if report.is_admitted() {
    // 모든 정적 검사를 통과했다. 실제 export의 Result도 반드시 처리한다.
}
for issue in &report.issues {
    // 분류는 issue.reason으로 하고 message는 표시용으로 사용한다.
    let _ = (&issue.location, issue.reason, &issue.message);
}
```

## 수집 범위와 순서

문서·옵션 검사 다음에 선택한 페이지 순서로 검사한다. 각 페이지에서는 layer를
깊이 우선 전위 순회하고 leaf의 PaintOp를 원래 순서대로 검사한 뒤 페이지 크기를
검사한다. `location.page_index`는 입력 slice 기준 0부터 시작한다.
`node_index`는 각 페이지의 전위 순회 번호, `op_index`는 그 페이지 전체의 PaintOp
번호다. 페이지 크기에는 node와 op 번호가 없고, layer·clip에는 op 번호가 없다.
문서·옵션 오류는 모든 위치 필드가 `None`이다. `operation`은 `rectangle`, `image`,
`pageBackground`와 같은 PaintOp 이름이다.

한 위치의 같은 `DirectPdfPreflightReason`은 한 번만 수집한다. 서로 다른 위치의
같은 사유는 모두 유지한다. 동일 op의 회전·모서리·채우기·불투명도·획 두께처럼
독립적인 잘못된 필드는 각각 수집한다. 잘못된 색상 수·stop·상자에서 파생 shader를
생성하거나 잘못된 shaft에서 화살촉을 도출하지 않는다. 이 경우 파생 검사 대신
선행 조건의 거부 사유를 남긴다.

공개 reason enum으로 분류하며 오류 문자열을 분석하지 않는다. 기존 export도 같은
검사를 사용해 첫 번째 거부 메시지를 종전 형식 그대로 반환한다. `use the svg backend`
안내나 잘못된 좌표 때문에 전체 선택 문서의 export가 실패하는 정책을 유지한다.

## 상한과 잘림

`DirectPdfPreflightLimits`는 다음 범위로 보정한다. 0은 각 최솟값으로 올린다.

| 필드 | 기본값 | 허용 범위 | 의미 |
| --- | ---: | ---: | --- |
| `max_issues` | 256 | 1~4096 | 저장하는 거부 사유 수 |
| `max_work_items` | 100000 | 1~1000000 | 방문한 페이지·layer·op 수의 합 |

실효값은 `report.limits`, 실제 방문 수는 `work_items_checked`로 확인한다.
결과 수가 정확히 상한이고 이후 오류가 없으면 `truncated`는 `false`다. 추가 오류를
발견했는데 저장하지 못하면 `limit_reached == Some(Issues)`가 된다. 다음 방문이 작업
상한을 넘으면 `Some(WorkItems)`가 된다. 두 경우 모두 `truncated`는 `true`이며
`is_admitted()`는 `false`다. 따라서 오류 배열이 비어 있어도 잘린 보고서를 통과로
판정할 수 없다. 완전한 목록이 필요하면 명시적인 상한 안에서 페이지 선택을 나누어
호출하고 각 선택의 페이지 번호를 대응시킨다.

반복자는 한 번에 모든 형제 노드를 복사하지 않으며 재귀 호출 대신 방문 깊이에
따른 stack을 사용한다. 위 상한은 보고서와 방문 수를 제한한다. 이미 메모리에 있는
단일 PaintOp의 path·text 등 전체 입력 바이트나 실행 시간의 전역 상한은 아니다.
기존 export에는 이 진단용 방문·결과 상한을 적용하지 않는다.

## 정적 통과와 실제 성공의 구분

정적 검사에는 좌표·도형 스타일·채우기·clip·지원 이미지 형식·페이지 크기와
`raster_dpi`를 포함한다. 이미지 형식은 signature로 확인하고 실제 decode는 export에
남긴다. 예를 들어 PNG signature만 있는 잘린 payload는 정적 통과 뒤 실제 export에서
`cannot decode image payload`로 실패한다. 사전검사 결과를 성공한 PDF의 증거로 쓰지 않는다.
페이지 크기는 PDF point로 변환한 실제 f32 값도 양수여야 한다. 양의 f64 값이 f32에서
0으로 사라지면 사전검사와 실제 export 모두 `InvalidPageWidth` 또는 `InvalidPageHeight`로
거부한다. 표현 가능한 작은 페이지에는 별도의 임의 최솟값을 두지 않는다.

실제 이미지 decode, 폰트 탐색·생성, 정확한 글리프 face 준비, text alternative 계약,
RawSvg 준비와 PDF recording은 기존의 strict export 경로에서 별도로 검증된다.
호출자는 `layer_trees_to_pdf*`의 `Result`를 계속 처리해야 한다. 링크·named destination
목록은 이 API의 입력에 포함되지 않는다.

검사 계약은 `tests/render_p37_direct_pdf_export.rs`의
`direct_pdf_preflight_contract`에서 합성 입력으로 보호한다. 이 검사는 실물 문서의
절대 조판 좌표나 한컴 시각 일치율을 주장하지 않는다. 최종 한컴 자료 대조, 전체 corpus,
실제 OS·GPU의 검증은 별도 후속 작업으로 남는다.

## P81: 원형 gradient의 벡터 admission

Type 1 선형에 이어 type 2 원형만 Direct PDF에 허용한다. 배경·사각형·타원·path 모두
Native의 `gradient_paint`가 만드는 Skia radial shader를 PDF canvas에 그대로 기록한다.
결과에는 표준 PDF의 `/ShadingType 3`과 동심 시작·끝 원이 있어야 하며, 단색 대체나
전체 면의 이미지 기록은 이 계약에 포함되지 않는다. Type 3의 기존 원형 근사는 실제
원뿔형의 증명이 아니므로 닫아 둔다. Type 4의 SkSL 사각 shader도 허용하지 않는다.

원형의 물리 중심·반경은 Native와 동일한 `gradient_geometry::radial_circle`을
소비한다. 반경은 중심에서 상자의 가장 먼 모서리까지의 거리다. 가로·세로로 각각
정규화한 타원이나 긴 축의 절반을 반경으로 사용하는 해석은 허용하지 않는다.
기존 한컴 원형 대조군의 근거는
[`verified_radial_print_geometry.rs`](../../tests/cases/verified_radial_print_geometry.rs)에서
확인할 수 있다. 이번 admission은 그 기하를 다른 PDF canvas에 기록하는 제한된 확장이다.

| 항목 | 허용 조건 |
| --- | --- |
| 중심 | 가로·세로 각각 0~100% |
| 상자 | 실제 f32 replay 사각형의 좌표·변·크기가 유한하고 비어 있지 않음 |
| 반경 | 공통 f64 기하와 f32 중심·반경이 유한하며 f32 반경 제곱이 1 이상, 유한함 |
| 원의 경계 | f32 중심±반경이 두 축 모두 유한하고 중심 양쪽에서 서로 구별됨 |
| 색상 | 2~4096개 |
| stop | 빈 목록의 균등 배치 또는 색상 수와 같은 유한·단조·0~1 위치 |
| f32 stop | 서로 다른 원본 위치가 같은 f32 위치로 합쳐지지 않음; 의도한 동일 위치의 hard stop은 유지 |
| opacity | 일반 도형의 기존 0~1 유한 값 계약을 유지 |
| clip | 기존 layer clip 사각형·text clip 검사를 유지 |

잘못된 중심은 `InvalidRadialGradientCenter`, 잘못된 원은
`InvalidRadialGradientCircle`로 보고한다. f32에서 사라지는 색상 구간은
`InvalidExpandedGradientStops`로 거부한다. 좌표·stop·opacity를 clamp해서 admission을
통과시키지 않는다. Pattern fill과 shadow도 기존과 같이 거부한다.

`direct_pdf_radial_gradient_contract`는 PDF의 radial shading·동심 원·이미지 부재,
중심·가로세로비·hard stop·최대 색상 수·잘못된 입력·clip·opacity를 검사한다.
Native의 같은 원형 shader는 독립적인 모서리 거리 규칙의 색상과 같은 거리의 등색선,
clip 밖의 투명도와 clip 안의 alpha로 대조한다. 이 합성 계약을 실물 문서나 실제 GPU의
최종 시각 검증으로 확대해서 해석하지 않는다.
