# 미주 간격의 독립 Print 대조군

- 용도: 초기18개 편집 검증에서 나온 미주 간격 차이의 원인 분석. 새 회귀 fixture/golden이 아니다.
- 입력은 현재 fresh WASM 공개 API로 생성했다. XML이나 저장 LineSeg를 수동 수정하지 않았다.
- 수정 전 product source: `9b37446556273d2dae1d1a089645687defb3215b`. WASM SHA `a2cb686c42193dbb9572f7609b11a4875c5b5eaa2df91dfec15aa59e5a620747`.
- 기준은 동일 입력을 실제 Hancom2020 MCP의 PrintMethod0(1-up)으로 출력한 PDF다. GUI/LibreOffice/cairo 재생성은 사용하지 않았다.
- Native/fresh WASM 전4쪽은 각각100/100/99.63964/94.91525%이며, 숫자 삭제 뒤 별도 미주쪽은85.90604%다. 90% 통과한 대조군에서도 복수 문단 및500HU 미주 간격 차이를 직접 확대 확인했으므로 완료·승인 증거로 사용하지 않는다.

| 입력 | 입력 SHA256 | 같은 입력의 Print PDF / SHA256 | Print job |
| --- | --- | --- | --- |
| [endnote-boundary-empty-v95.hwpx](endnote-boundary-empty-v95.hwpx) | `3abb4cc1fd89fa1fa0053c30d6ab849882c061ec9ef9b37bd0ad0dfcc030abef` | [endnote-boundary-empty-v95-3abb4cc1fd89-2020.pdf](../../../../../pdf/semanticist21-20261008/endnote-boundary-v95/endnote-boundary-empty-v95-3abb4cc1fd89-2020.pdf) / `dbd8a53ca220bec0aae39f19aa1e23674e48208c40b78336ec456161b8ae4077` | `c6f31c85-01ff-47bb-b9c0-37d097df9241` |
| [endnote-boundary-short-text-v95.hwpx](endnote-boundary-short-text-v95.hwpx) | `ea85f1e1e5b6dc96084a4ee2c21484d1ed68ae69f7fdadcc9c4aed37d8bcdb5f` | [endnote-boundary-short-text-v95-ea85f1e1e5b6-2020.pdf](../../../../../pdf/semanticist21-20261008/endnote-boundary-v95/endnote-boundary-short-text-v95-ea85f1e1e5b6-2020.pdf) / `b3d1694bc791eedd8d350b3a835fbe58eb98d26dece0676629fdccf5f2da4a11` | `e2be9fac-acf7-48ce-99a1-10be7c3f5864` |
| [endnote-boundary-two-paragraphs-v95.hwpx](endnote-boundary-two-paragraphs-v95.hwpx) | `9368eb3141b5ff179f24be6f05ffbf3e70e62e36f94d007fdc5f1d1dc1f9fcb6` | [endnote-boundary-two-paragraphs-v95-9368eb3141b5-2020.pdf](../../../../../pdf/semanticist21-20261008/endnote-boundary-v95/endnote-boundary-two-paragraphs-v95-9368eb3141b5-2020.pdf) / `ab3aaca52ef701261cf1fceaabf7d1bf894463a41a91f6faf08c535d3eed4347` | `0e33d8a6-39a4-4899-ae25-5b7a60cde41a` |
| [endnote-boundary-explicit-note-gap-v95.hwpx](endnote-boundary-explicit-note-gap-v95.hwpx) | `574239ee15f3a8f03594d840789c5a32418c098d51a417304bd9f1d4442bffde` | [endnote-boundary-explicit-note-gap-v95-574239ee15f3-2020.pdf](../../../../../pdf/semanticist21-20261008/endnote-boundary-v95/endnote-boundary-explicit-note-gap-v95-574239ee15f3-2020.pdf) / `1b323bc4244f4347cedd0f3d0652d8e66f2080b1528b369eb6cb833722cd9061` | `5d3538a1-d91c-4ad5-975d-75b2b388ca45` |

- `pdftotext -bbox-layout` 독립 관측: 미주1→2의 baseline 간격은 빈 미주·짧은 본문에서8.990175pt(약11.9869px), 같은 미주 안의 문단 간격은11.747162pt(약15.6629px), 명시적 미주 간격500HU는14.024673pt(약18.6996px)다. PDF 글리프 bbox의 yMin 차이를 사용했으며 글리프 bbox 자체를 줄 상자로 간주하지 않는다.
- 수정 전 trace: formatter15.6267px을 미주마다 누적하지만 renderer는 본문 뒤첫미주의 구분선 floor 적용 후 HeightCursor가 저장 vpos로 다시 당긴다. 별도 미주쪽에서는 그 당김 없이15.6267px을 소비한다. 명시적 미주간격을 가상 마지막 LineSeg에 쓰는 prepare 경로와 최종 renderer의 vpos 덮어쓰기도 분리해 확인해야 한다.
- TSV·trace·Print job 원문·대표 review는 ignored `output/pr-review/semanticist21-20261008/`에 있으며, 이후 source 변경에는 새 Native/fresh WASM으로 다시 판정한다.
