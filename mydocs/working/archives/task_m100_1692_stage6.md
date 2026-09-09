# Task #1692 Stage 6 - SO-SUEOP p5 머리말/꼬리말 및 HMapsi OLE 보정

## 시작 상태

- 직전 커밋: `6d70ad462 task 1692: SO-SUEOP 미주 페이지 경계 보정`
- Stage 5에서 `samples/SO-SUEOP.hwp`, `samples/SO-SUEOP.hwpx` export PDF는 기준 `pdf/SO-SUEOP-2024.pdf`와 같이 46쪽으로 유지됐다.
- Stage 5에서 p43-46 미주 흐름은 기준 PDF의 주요 페이지 경계와 맞췄다.

## 수정 내용

- HWPX `DISTRIBUTE_SPACE` 문단 정렬을 글자별 분산이 아니라 공백 기반 `Justify`로 파싱했다.
- 머리말/꼬리말 문단 compose 경로에서 페이지 번호 AutoNumber를 필드 마커와 같은 단계에서 치환하게 했다.
- HWP3 머리말 단일 줄 `Justify`도 머리말 영역 폭 기준으로 공백 분산되도록 렌더 조건을 보정했다.
- HWPX `HMapsi` OLE는 일반 EMF/DIB preview가 없으므로, 첫 페이지에 한해 `Preview/PrvImage.png`를 페이지 좌표로 배치하고 OLE bbox로 clip하는 fallback을 추가했다.

## 검증 결과

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 issue_1692_so_sueop_hwpx_title_ole_renders_from_embedded_preview -- --nocapture`
- `env CARGO_INCREMENTAL=0 cargo build -q --bin rhwp`
- p1 triptych: HWPX는 기준 PDF의 왼쪽 세로 제목이 복원되고 OLE placeholder가 사라짐.
- p5 triptych: HWP/HWPX 모두 머리말 우측 `박전현선생` 분리 배치와 꼬리말 `협성고등학교 5` 렌더를 확인함.

## 남은 제약

- `samples/SO-SUEOP.hwp`의 p1 제목 객체는 HWP3 원본 내부에 데이터가 없다. 파서 실측 결과 `type=1`, `name="00000000.OOO"`, `n_ext=0`, BinData는 `Link (ID: 1, ext: OOO, loaded: 0 bytes)`다.
- 따라서 HWP 단독 렌더에서는 기준 PDF의 p1 세로 제목을 실제 이미지로 복원할 원본 바이트가 없다. 다음 stage가 필요하면 HWP3 missing OLE/link 객체에 대한 정책을 별도로 정해야 한다.
- 이 stage의 다음 커밋에는 이 stage 문서 하나만 포함한다.
