# PR #6191 시각 검토 증적

## 생성 원칙

- 한컴 기준 PDF는 `hwp2024-mcp-convert` client로 **한 건씩** `start -> status(succeeded) -> download` 순서로 생성했다.
- 저장 metadata가 2022 이하이거나 미상인 문서는 engine `2020`, 2024 문서는 engine `2024`를 사용했다.
- rhwp PDF는 같은 fixture와 회귀 대상 페이지를 `rhwp export-pdf`로 산출했다. #6168/#6169 fixture는 SVG PDF backend가 `InvalidImage`으로 실패해 Native Skia PNG를 대신 사용했으며, pristine `upstream/devel`에서도 같은 오류를 재현했다.
- `raster/`에는 144dpi 한컴/rhwp PNG와 absolute-error diff를 보관한다. 픽셀 수는 서로 다른 폰트와 raster backend 영향이 있으므로 단독 합격 기준으로 쓰지 않는다.

| 원본 PR | 회귀 fixture | 기준 engine | 검토 페이지 | 한컴 PDF SHA-256 | rhwp 산출 |
| --- | --- | --- | --- | --- | --- |
| #6183 | `hwp3-table-caption.hwp` | 2020 | 1 | `48940a3579d0fb3a9545c68e0b3d00da1094c9eea67773a1d781b958c7319305` | `rhwp/issue_6078_rhwp.pdf` |
| #6160 | `39819_press_release_header_slice.hwp` | 2020 | 1 | `223bed964c02e92569266680a1684a4c637764f79d9a977f8df10d1e58b6b9fc` | `rhwp/issue_6110_rhwp.pdf` |
| #6158 | `56345_regulatory_impact_analysis.hwp` | 2020 | 7 | `104edf3dc9939ced25dcb004b371fd778847f48a6e1679cfbbe699a205dfa05a` | `rhwp/issue_6111_rhwp_p007.pdf` |
| #6169 | `156483831_poster_title_above_offset_float.hwp` | 2020 | 1 | `ede561decf069abb84fcbc271d620c62836d43cce8c8221e760f5d2157d8168e` | `raster/issue_6133/rhwp-native/` |
| #6168 | `156731730_contact_table_logo_overlay.hwpx` | 2020 | 1 | `b0163847d534459f875dd0fd9ad76396b3b096ae026b2d3eafef128a35c3e20d` | `raster/issue_6134/rhwp-native/` |
| #6177 | `156544683_title_row_underfit.hwp` | 2020 | 1 | `878aad6d325bc8b4d054432174ab90d8bb5ea9071a0c223421c354aa68cff267` | `rhwp/issue_6135_rhwp.pdf` |
| #6162 | `156462405_smart_expo.hwp` | 2020 | 7 | `f7e14fdd1f3e623fb8a21daedea5ad3ab0a89b4a6455b958798b341308b91478` | `rhwp/issue_6140_rhwp_p007.pdf` |
| #6170 | `156555538_securities_settlement_review.hwpx` | 2020 | 9 | `deaa386157a7207a0271d13566a45f1ae10245c9fed0045c1ee71270a1a7e726` | `rhwp/issue_6143_rhwp_p009.pdf` |
| #6166 | `156583583_press_release_logo_band.hwpx` | 2020 | 1-2 | `729a45074c914a540babf548b258e779864c310c40decbd607015b0a2301ba8a` | `rhwp/issue_6146_rhwp.pdf` |
| #6165 | `156741101_press_release_band.hwpx` | 2024 | 1 | `195cab9913f310adb910a4a4b56f72004f8df4525b5cc66f7388b7ddd72e2ed2` | `rhwp/issue_6147_rhwp_p001.pdf` |
| #6163 | `2026_oss_rst.hwp` | 2024 | 1 | `9b0e38ad4943a3cc86ff947ffc7c7d2fa6aee288037b69a6a936c69a12f6255a` | `rhwp/issue_6156_rhwp_p001.pdf` |
| #6161 | Canvas2D table-cell underline | local Vite | page 9 | N/A | `studio/issue_6117_canvas2d.png` |

## SVG PDF backend 한계

`issue_6133`과 `issue_6134`의 SVG PDF export는 `SVG->chunk: InvalidImage`으로 실패했다. source fixture만을 pristine `upstream/devel` checkout에 넣어 같은 명령을 실행해도 같은 오류가 났다. 따라서 통합 PR의 변경으로 발생한 회귀가 아니다. 두 fixture 모두 Native Skia PNG export는 성공했으며, renderer regression과 Native Skia lib test도 통과했다.
