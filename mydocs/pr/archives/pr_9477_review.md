# PR #9477 검토

[planet6897 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9477)

## 검토 버전

- 원 PR head `6757868292e2aa398d99a7098a0285c621eb6dff`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `1b061b84162feb1ad367ade20dffc5c5ec984f5c` | `d128eb0073704b1afd519dfd006b7c45b8977cac` |
| `1c06467d6de2fcd53ee0e5999cd9777c5e3aadd1` | `22edd8d972488233a20c0772dd3e39a743be0e5a` |
| `f2d8c3275f377455a00eced51276dfbb0a7ae403` | `a466d3935758f5a87e979d89a74b1becdf2e98cf` |
| `244576dc4148762b4713f95b68758f932622957d` | `34d8ed2f9b38479d2ae91de4e9e47f3dae47bc28` |
| `6757868292e2aa398d99a7098a0285c621eb6dff` | `e13e4803c02a63f73dcbdc41011dd84d33ba2b09` |

## 원인과 수정

#8506의 1쪽 표지에서 단 정의·문단 기준 제목 표·글자처럼 취급한 빈 그림이 한 문단에 들어 있습니다. 저장 호스트 줄이 제목 표보다 앞에서 끝나는데도 제어 목록을 표 하나로 제한해 기존 저장 오프셋 프레임을 적용하지 못했습니다. 제목 표가 아래로 밀리고 두 줄이 셀 안에 담기지 않았습니다.

배치는 `ParagraphFloatPlacement::from_empty_stored_offset_host`에서 저장 오프셋 프레임을 적용합니다. 측정은 같은 저장 호스트 프레임이 유효할 때 행 높이를 산정하는 기존 중첩 표 경로에서 표의 끝 높이를 반영합니다. 저장 줄 유효성을 검사하고, 별도의 떠 있는 개체 또는 두 번째 표가 있으면 이 프레임을 사용하지 않습니다. 제목 표와 두 줄의 소속·오프셋·높이·다음 셀을 관계로 검증하며, 기준 PDF의 2쪽을 고정형 쪽수 검사로 확인합니다. 신규 샘플의 기존 본문 넘침 1건은 해당 샘플의 원장 한 행에 등록했습니다. [회귀 검사](../../../tests/cases/issue_8506_cell_float_title_table.rs) · [본문 넘침 원장](../../../tests/fixtures/body_overflow_baseline.tsv)

## 독립 Print 대조

공개 원본과 한/글 2020 Print PDF를 96 DPI, 호환 모드 2022, raw 2px 실루엣으로 전쪽 대조했습니다. PDF에 내장된 글꼴에 대응하는 TTF와 굴림체 컬렉션을 공급했습니다. 1쪽의 빨간 위 괘선과 파란 아래 괘선 사이 제목 두 줄, 날짜, 부서명을 직접 대조했습니다. 2쪽의 제목·표·본문이 담긴 두 경로의 PNG SHA-256은 수정 전후 각각 같았습니다. [공개 원본](../../../samples/issue8506/36410942_approval_cover_title.hwpx) · [한/글 Print PDF](../../../pdf/issue8506/36410942_approval_cover_title-2020.pdf)

| 쪽 | 원 PR Native | devel Native | 후보 Native | 원 PR WASM | devel WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 98.47% | 50.28% | 100.00% | 98.59% | 50.34% | 100.00% |
| 2 | 97.65% | 99.96% | 99.96% | 98.20% | 99.97% | 99.97% |

원 PR 제출 TSV와 후보의 대응 행 4개를 대조했습니다. [전쪽 대조 TSV](../assets/issue_8506/integration/source-page-comparison.tsv) · [Native 확인 이미지](../assets/issue_8506/integration/native/review_001.png) · [WASM 확인 이미지](../assets/issue_8506/integration/wasm/review_001.png)

후보에서 `cover_title_table_sits_at_its_paragraph_offset_and_grows_the_row`를 실행해 통과했습니다. #6697 중첩 표 세로 오프셋과 #1510 같은 문단의 복수 표 대조군도 통과했습니다.

## 병합 후 원 PR 안내

두 줄 제목의 배치, 원 PR 및 devel과의 두 쪽 비교, 같은 한컴 Print를 사용한 Native·fresh WASM의 review와 overlay를 안내합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
