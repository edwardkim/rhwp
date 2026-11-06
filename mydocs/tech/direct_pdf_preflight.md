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

실제 이미지 decode, 폰트 탐색·생성, 정확한 글리프 face 준비, text alternative 계약,
RawSvg 준비와 PDF recording은 기존의 strict export 경로에서 별도로 검증된다.
호출자는 `layer_trees_to_pdf*`의 `Result`를 계속 처리해야 한다. 링크·named destination
목록은 이 API의 입력에 포함되지 않는다.

검사 계약은 `tests/render_p37_direct_pdf_export.rs`의
`direct_pdf_preflight_contract`에서 합성 입력으로 보호한다. 이 검사는 실물 문서의
절대 조판 좌표나 한컴 시각 일치율을 주장하지 않는다. 최종 한컴 자료 대조, 전체 corpus,
실제 OS·GPU의 검증은 별도 후속 작업으로 남는다.
