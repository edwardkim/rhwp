# humdrum00001010 통합 시각 증적 - PR #6666, #6668

- 통합 branch: `review/humdrum00001010-green-20260903`
- 검토 head: `ca994cef7f99ddf306df2181f7699046103f6b0b`
- 생성일: 2026-09-03

## #6666 - #2004 그림 stack

| 입력 | 기준 PDF | 페이지 | 결과 |
|---|---|---:|---|
| `samples/issue2004_cell_image_stack.hwp` | `pdf/issue2004_cell_image_stack-2022.pdf` | 4-8 | complete, 5/5 reviewed, automated flags 0 |
| `samples/issue2004_cell_image_stack.hwpx` | `pdf/issue2004_cell_image_stack-2022.pdf` | 4-8 | complete, 5/5 reviewed, automated flags 0 |

`rsvg` rasterizer로 산출했다. Chrome WebFont rasterizer는 macOS display service 오류로 완료되지 않아 실행 증적으로 사용하지 않았다. PDF와 rasterizer의 글꼴·안티앨리어싱 차이 때문에 pixel/ink score를 fidelity 합격률로 쓰지 않았다. 대신 HWP/HWPX에서 그림 identity 및 page 4~8 bounds의 회귀 테스트가 통과했는지와 review PNG의 frame/flow 검토를 사용했다.

- HWP assets: `issue2004-hwp-page-004.png` ~ `issue2004-hwp-page-008.png`
- HWPX assets: `issue2004-hwpx-page-004.png` ~ `issue2004-hwpx-page-008.png`
- asset root: `mydocs/pr/assets/pr_6666_6668_humdrum00001010_20260903/`

## #6668 - SVG layer/legacy parity

`samples/issue-617/exam-kor` page 5를 같은 binary에서 기본 layer backend와 명시적 legacy backend로 export했다. SVG bytes는 다르지만 rsvg raster absolute error는 `0`이었다. `issue6520-layer-vs-legacy-page-005.png`는 두 raster의 side-by-side 증적이다.

이 결과는 canonical layer 경로 전환이 해당 representative legacy output을 시각적으로 바꾸지 않았다는 범위의 증적이다. 기준 PDF와의 전체 fidelity 판정 또는 Studio/WebFont parity 증적은 아니다.

## 제외 범위

PR #6685는 Draft이며 CI 실패 상태라 이 통합 검토와 시각 증적 범위에서 제외했다.
