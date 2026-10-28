# planet6897 PR #6702, #6732 통합 시각 증적

## 범위와 재현 기준

- 기준 devel: `d1831146587b1ac2346f9ed1216a64c2943a02f9`
- 통합 후보: #6702 원 code/report와 test/evidence, #6732 source head를 provenance-preserving
  `-x` cherry-pick으로 누적하고 #6702 test fixture를 공개 canonical sample로 보정한 상태
- 출력기: macOS native-skia `rhwp export-png --profile print`
- 두 후보 export는 모두 794 x 1123 PNG다.

## #6702 셀 host 문단 text

| 자료 | 파일 | SHA-256 | 직접 확인 범위 |
| --- | --- | --- | --- |
| canonical HWPX | `samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx` | `e7b147f7cea66c97bed79085a3d89c2656037e0f711232f659ed3c7344984f62` | 공개 fixture와 manifest 대상 |
| 기여자 before | [before p30](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6702/source/contributor-before-p30.png) | `16383794f553df3cc3fc6cd2a62f1984c6dc7fa03fa1c627bb5a25c7034514a2` | target caption 미표시 |
| 기여자 after | [after p30](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6702/source/contributor-after-p30.png) | `d74413502a071420b45927cd7b696b50ec4298521df7c47f3b5fbdc352649b05` | target caption 표시 |
| 기여자 oracle | [oracle p30](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6702/source/contributor-oracle-p30.png) | `3f6912ef2b19228e0965240944bc193e4ed2ca9cd252e59e5617783973f66a9e` | source 비교 기준 |
| 통합 후보 | [candidate p30](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6702/candidate-p30/80550-agricultural-machinery-act-amendment.png) | `c4b0078b7c3cb4cdda10ac365689f04e648a11484e72394e68019e98a554ff64` | target caption 표시 |

- 후보 p30에서 `<향후 10년간 폐농업용 지게차 해체 수익 계산>` 표제가 표 아래에 존재함을 직접 확인했다.
- 이번 후보 export에는 일부 한글 glyph의 폰트 대체가 보인다. 따라서 이 표는 target caption의 존재를
  확인하는 증적이며, contributor oracle과의 full-page pixel match나 `rhwp-studio` 렌더의 대체 증거가 아니다.

## #6732 saved-tail bottom seat

| 자료 | 파일 | SHA-256 | 직접 확인 범위 |
| --- | --- | --- | --- |
| canonical HWP | `samples/issue5941/1490000-201600081_roadmap_research.hwp` | `a06f46ec3f175c7cfa84eb3178b8b3fbdf78e94f71b31d7d87f3417a2617dae9` | 공개 fixture |
| 통합 후보 | [candidate p304](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6732/candidate-p304/1490000-201600081_roadmap_research.png) | `5d5ebc292cb8d6af044469b7fd61b5a953e7f05f50f1669c43f16b5be65ee62d` | 마지막 logical page 비공백 |

- 기준 devel native binary는 305 logical pages, 통합 후보 native binary는 304 logical pages를
  보고했다. 후보의 마지막 logical page export에서 표의 마지막 행과 footer가 나타나며 빈 tail page나
  육안상 clipping은 보이지 않는다.
- image에 보이는 footer `- 293 -`은 문서가 인쇄한 physical label이고 export logical page index 304와
  다른 값이다. 둘을 같은 page number로 해석하지 않는다.
- 본 batch에서는 Hancom PDF 변환, overlay, pixel score를 새로 수행하지 않았다. source PR의 historical
  Hancom page-count 주장을 재검증한 것이 아니며, 이 증적은 native candidate의 page-count와 마지막 페이지
  상태로 한정한다.

## 결론

- #6702은 공개 fixture를 반드시 읽도록 메인터너 보정한 뒤 target caption 존재를 확인했다.
- #6732는 공개 fixture에서 source regression이 기대한 305 -> 304 logical-page 변화를 확인했다.
- 두 항목 모두 통합 PR의 최신 head CI가 성공하기 전에는 source PR close/comment 또는 full-fidelity
  판정을 하지 않는다.
