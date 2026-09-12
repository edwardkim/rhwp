# 단계 1 — 한컴 하이퍼링크 기준 계약 조사

- Issue: [#6963](https://github.com/edwardkim/rhwp/issues/6963)
- 기준: `a3cd825c23d550e0c5f46b4e2eb3735a537f23f2`
- 실행일: 2026-09-10
- 상태: 기준 샘플 조사 및 재검증 도구 완료. 제품 편집/PDF 출력 구현은 아직 시작 전.

## 검증 방법

`tools/hyperlink_oracle_probe.py`는 pypdf로 PDF의 실제 `/Annots`를 읽는다. 화면 문자열의 URL 자동 감지를 링크 보존으로 오인하지 않는다. `--expect-uri` 또는 `--expect-link-count`가 불일치하면 exit 1이다. URI를 외부로 열거나 네트워크로 전송하지 않는다.

```bash
python tools/hyperlink_oracle_probe.py pdf/basic/Textmail-2022.pdf \
  --source samples/basic/Textmail.hwp \
  --expect-uri http://www.hancom.co.kr --expect-link-count 1
```

실행 환경: Python 3.12 / pypdf 6.10.0 (Codex bundled runtime). 저장소 Python venv를 사용할 때는 pypdf를 준비한다.

양성 4쌍 통과(exit 0), 없는 URI와 잘못된 개수를 지정한 음성 2건 통과(기대 exit 1). HWPX Path와 PDF fragment URI 일치 확인. 내부 목적지 p4→p14 확인.

## 샘플별 기준

### samples/basic/Textmail.hwp

- 원본 SHA-256: `3ea41d01844dfe689c58c3aebc4193466df449b954b047c65e6289ded3e6f05c`
- PDF: `pdf/basic/Textmail-2022.pdf`
- PDF SHA-256: `c2d295531e6174142dc0222294bfc0aa0f2066d0d125d70f16c54f36ba685715`
- 생성 정보: `Hancom PDF 1.3.0.550` / `Hwp 2022 12.0.0.4426`
- 페이지 수: 1, Link 주석 수: 1

| 페이지 | action | 주소 또는 목적지 | 원시 Rect (PDF point) |
| --- | --- | --- | --- |
| 1 | `/URI` | `http://www.hancom.co.kr` | `[405.973, 721.646, 495.531, 711.65]` |

### samples/hwpx_sample2.hwpx

- 원본 SHA-256: `188bdfe21f89e117d8897f4102aa6f741962b23a3019ad2aaf7bdc222d90fdb2`
- PDF: `pdf/hwpx_sample2-hwpx-2020.pdf`
- PDF SHA-256: `d69bf2c042f1b3f4713ad838c7d4f7817f2568cea62f1df78731086ebb8e81dc`
- 생성 정보: `Hancom PDF 1.3.0.550` / `Hwp 2022 0.0.0.0`
- 페이지 수: 29, Link 주석 수: 5

| 페이지 | action | 주소 또는 목적지 | 원시 Rect (PDF point) |
| --- | --- | --- | --- |
| 2 | `/URI` | `https://apply.lh.or.kr` | `[276.948, 523.855, 363.644, 512.868]` |
| 8 | `/URI` | `https://apply.lh.or.kr/LH/index.html#MN::CLCC_MN_0010:` | `[326.586, 481.041, 420.252, 471.045]` |
| 9 | `/GoTo` | `p1` | `[301.455, 780.544, 404.981, 770.577]` |
| 11 | `/URI` | `https://apply.lh.or.kr` | `[165.29, 385.076, 249.607, 375.08]` |
| 29 | `` | `실행 목적지 미확인` | `[252.667, 681.125, 310.72, 671.129]` |

### samples/hwp-img-001.hwp

- 원본 SHA-256: `0d632afcea1111c5af14413f10f91d982055a67257dc73e5e5db2c04c1c00f60`
- PDF: `pdf/hwp-img-001-2022.pdf`
- PDF SHA-256: `05b1823e949176cc5309b8788f77dee50fa841d2996804e0a6420a4ea7587250`
- 생성 정보: `Hancom PDF 1.3.0.550` / `Hwp 2022 12.0.0.4426`
- 페이지 수: 1, Link 주석 수: 1

| 페이지 | action | 주소 또는 목적지 | 원시 Rect (PDF point) |
| --- | --- | --- | --- |
| 1 | `/URI` | `mailto:mosfpr@korea.kr` | `[365.146, 284.864, 450.228, 272.886]` |

### samples/hwpctl_Action_Table__v1.1.hwp

- 원본 SHA-256: `7076cb9bf6660acad61840dead62a3baeba43b8cb4afe8501777637b2f82fc0e`
- PDF: `pdf/hwpctl_Action_Table__v1.1-2022.pdf`
- PDF SHA-256: `2dd05189158df29c171ec76234ec0bb3991e00af82c64f4866e734aefca39d87`
- 생성 정보: `Hancom PDF 1.3.0.550` / `Hwp 2022 12.0.0.4426`
- 페이지 수: 16, Link 주석 수: 1

| 페이지 | action | 주소 또는 목적지 | 원시 Rect (PDF point) |
| --- | --- | --- | --- |
| 4 | `/GoTo` | `p14` | `[431.56, 367.285, 519.231, 357.283]` |

## 판정과 다음 구현의 제약

- Textmail p1과 LH p2/p8/p11을 HTTP/HTTPS 성공 기준으로 사용한다. mailto와 내부 GoTo는 확장 비교용이며 현재 UI 범위와 구분한다.
- LH p29의 Link 주석에는 실행 URI가 없다. 링크 개수 5만 맞춰서는 성공이 아니다. p2/p8/p11 각각의 URI와 글자 범위를 대조해야 한다.
- PDF Rect는 한컴 출력에 y 값이 역순인 사례가 있다. 비교 시 min/max로 정규화하되, 원시 값은 위 표로 보존한다.
- `-2020` suffix는 저장소 engine bucket이며 실제 Creator의 제품 버전과 다를 수 있다.
- `Hyper(hwp2010)`는 기존 PR #6466에 destination 경고가 기록되어 있다. 파일이 열린다는 사실만으로 모든 링크 목적지가 유효한 것으로 판단하지 않는다.
- HWP는 이 도구에서 hash만 확인한다. 필드 범위·명령 내용은 후속 코어 parser 검증이 필요하다. HWPX는 ZIP 내부 section XML의 HYPERLINK 필드와 stringParam을 직접 추출한다.
- 대표 Textmail p1 및 LH p8은 기존 조사에서 Poppler raster를 열어 내용과 위치를 확인했다. 이 단계에서는 변경 후 화면이나 PDF 뷰어 클릭을 검증한 것이 아니다.
- 제품 완료는 Studio 편집·저장 왕복·브라우저 PDF·CLI PDF 전체 경로를 구현하고 검증한 이후다.

## 구현 진입점

- 필드/범위: `src/document_core/queries/field_query.rs`, `src/model/paragraph.rs`.
- 선택 영역 geometry: `src/document_core/queries/cursor_nav.rs`.
- PDF 공통 진입: `src/document_core/queries/rendering.rs`; 네이티브 backend: `src/renderer/pdf.rs`.
- Studio 편집: `src/command/commands/insert.ts`, `src/core/wasm-bridge.ts`, operation router.
- Studio 인쇄: `src/command/print-pages.ts`, `src/command/commands/file.ts`.

경로가 `src/command` 또는 `src/core`로 시작하는 마지막 두 항목은 `rhwp-studio/` 기준이다.
