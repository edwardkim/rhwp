# 묶음 그림 외곽선의 공통 프레임과 독립 Print

`e787be830763d871703e025aa7adae499f954618`의 Native082·최적화 fresh WASM033에서
같은 원본·독립 PDF를 검증했다. 입력·PDF·TSV·PNG SHA와 실제 명령은
[provenance.json](provenance.json)에 있다. 원본과 PDF는 이미 추적하는 아래 파일을 재사용했다.

- [원본 HWP](../../../../samples/2025%20행정업무운영%20편람%28최종%29.hwp)
- [한컴2024+KoPub Print PDF](../../../../pdf/2025%20행정업무운영%20편람%28최종%29-hwp-kopub-2024.pdf)
- [Native 전체383쪽 TSV](native_silhouette.tsv) / [fresh 전체383쪽 TSV](wasm_silhouette.tsv)

양쪽 전체383쪽의 PDF 쪽수가 같고 누락·90% 미만 쪽은 없다. 전쪽 raw 최저는
Native92.66367%, fresh92.65487%다. 94쪽 raw는Native98.64499%, fresh98.64029%이며
색상 경계 보정 전2px 실루엣 값이다. 엄격 RGB 일치율로 해석하지 않는다.

| 실제 출력 | review | standalone overlay |
| --- | --- | --- |
| Native082 94쪽 | [review](native_review_094.png) | [overlay](native_overlay_094.png) |
| fresh033 94쪽 | [review](wasm_review_094.png) | [overlay](wasm_overlay_094.png) |

원문 구역2 문단657·661의 묶음 자식 Picture는 실선을 갖지만 이전 경로는 bitmap만
추가했다. 공통 그림 테두리 모듈이 최종 ImageNode의 좌표·변환 프레임을 소비하도록
연결했다. 그룹에 이미 적용한 원본 회전·대칭을 다시 적용하지 않는다. crop·선 종류·
기본 물리 두께·쪽 수용 조건은 바꾸지 않았고 문서별 예외나1px 최소선은 추가하지 않았다.

같은 배율의 독립 Print·Native·fresh에서 두 그림의 닫힌 외곽, 기존 빨간 주석 상자의
쌓기 순서와 주변 본문을 직접 판독했다. 얇은 선의 raster 농도 차이는 잔여 차이로 기록한다.
독립 Print의 실제 vector 테두리 경로 bounds를72pt→96DPI로 환산한
[independent_coordinates.json](independent_coordinates.json)을 사용한다. 엔진 좌표를
기대값으로 복사하지 않았다. 94쪽의 양쪽98%·직접 판독 조건에서만 ±2px 좌표를 고정한다.
문서 전체98% 조건으로 확대하지 않으며, 이후에도 이쪽은 양쪽98% 이상을 유지해야 한다.

당시의 정식 회귀 2함수(`tests/cases/group_picture_outline_frames.rs`)는 원문 bin-data
소유로 그림과 묶음을 찾는다. 외곽선 한 개·그림과 같은 틀·bitmap 바로 위의 순서·기존
주석의 보존과 뒤 쌓기 순서를 검사하고 독립 PDF bounds의 ±2px를 함께 보호한다.
같은 최종 정식 소스를 Native081/082에 strict 컴파일해 이전2FAIL·보정 뒤2PASS를
확인했다. 두 build exit0이고 이전 실패는 외곽선 누락이다. 이전1255의 runner 인자
오류를 제품 재현으로 세지 않았으며 수정한1257의 실행과 test source SHA를 보존했다.
새 source의 자동 suite 수집·고정 devel 기준 manifest·unit-tier 검사도 통과했다.
정식 자동 수집 경로 `regression_suite_014`의 nextest 실행도 2PASS·0FAIL이다.
전체62조건860쪽 Native/fresh 비교에서 고정 devel 대비0.1%p 이상 하락은 양쪽 모두0쪽이다.

[기존 TAC·선 경계 증거](../stored_tac_source_frame_20261021/README.md)와 함께 읽는다.
111/242쪽의 기존 독립 좌표·내용 관계 검사와 crop·회전·그룹 해제·활성 셀·편람 경계
24검사는 Native082에서도 통과했다. 이 폴더의 완료를 전체45개 PR 통합 검증·CI·승인으로
표현하지 않는다. 전체 통합의 다른 실패와 차트·원 PR별 검토는 계속 진행한다.


## 2026-10-25: 원본 등록 환경 재검증과 선 중심 경로 검사 보정

기존 검사 등록에 사용한 같은 원본·독립 PDF의 SHA를 유지하고, runtime
`8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`의 Native224와 최적화 fresh WASM063으로
전 383쪽을 다시 대조했다. 쪽수는 모두 같고 누락·90% 미만 쪽이 없다. 이 작업에서는
행정업무운영 편람의 renderer를 새로 수정하지 않았다.

| 경로 | 전체 raw 최저 | 94쪽 raw | 242쪽 raw | 전체 TSV / 대표 PNG |
| --- | ---: | ---: | ---: | --- |
| Native224 | 95.28875% | 99.08163% | 99.55922% | [TSV](current-20261025/tsv/native.tsv) / [review](current-20261025/png/native/review_094.png) / [overlay](current-20261025/png/native/overlay_094.png) |
| fresh WASM063 | 95.96573% | 99.08481% | 99.56120% | [TSV](current-20261025/tsv/wasm.tsv) / [review](current-20261025/png/wasm/review_094.png) / [overlay](current-20261025/png/wasm/overlay_094.png) |

94쪽의 Native/fresh review·overlay를 직접 열어 두 그림의 닫힌 외곽과 원문 주석·
본문 순서를 확인했다. 얇은 선의 농도와 bitmap raster 차이는 남으며, 이 검사가
PDF와 선 굵기·RGB까지 완전히 같음을 입증한다고 쓰지 않는다.

기존 `outline.bbox == image.bbox`는 stroke의 중심 경로와 bitmap의 바깥 경계를
혼동했다. 공통 경로 `append_picture_image_node → make_picture_border_frame →
pre_rotation_bbox → source_geometry.viewport → Rectangle`은
`b7dbcbb594e07bec917ce2b23102a79744883e69`부터 최종 이미지 틀을 소비하고, 기본 얇은
선의 중심 경로는 선이 그림 안쪽에 놓이도록 반 굵기만큼 안에 있다. 이전 같은 모양의
bbox만 비교한 검사는 이 계약에서 실패한다.

원본 두 그림의 기본 실선 속성을 확인한 뒤, 중심 경로에서 실제 stroke 반 굵기를
확장한 네 잉크 경계가 그림 틀과 같은지 관계로 검사한다. 선 굵기나 관측된 차이
0.188976…px를 상수로 고정하지 않는다. 부동소수점 연산 오차만 크기에 비례해 허용하며,
독립 PDF 좌표의 ±2px, 원본 소유·그림/선의 유일성·쌓기 순서·주석 보존은 유지한다.
이 관계를 두꺼운 선이나 모든 그림의 보편 계약으로 확대하지 않는다.

같은 원본의 두 정식 검사를 실제 실행해 **2 PASS / 0 FAIL**, ignore·실행 누락 0건이다.
focused native Clippy와 rustfmt도 통과했다. 제품 수정이나 baseline 완화로 실패를
없앤 것이 아니며 이전 3589의 두 실패와 이번 3627 결과는 검사 보정의 전후 기록이다.
[현재 명령·해시·검사 소스](current-20261025/provenance.json)를 연결한다. 전체 PR,
전체 Rust 또는 새 base 대비 기능 보존 검증이 완료됐다는 뜻은 아니다.

## 현재 통합 후보의 범위

위 내용은 당시 실행의 역사적 기록이다. 사용자의 범위 지정에 따라 행정업무운영 편람과
해당 회귀 파일은 현재 통합 PR 검토에서 제외·제거했다. 이 자료를 현재 후보의 제출 증거나
활성 검사로 연결하지 않으며 삭제된 회귀 파일과 파생 참조를 다시 생성하지 않는다.
