# #7445 보정 72단계: 인라인 가로 펜과 세로 줄 소유 구분

Issue: #7445. 기존 정상 대조군과 현재 보정 문서만 검증했으며 새 HWP 보정 대상을 늘리지 않았다.
부모 head는 `0c94eff7b7b092ea3ffe8d1c2ca3f886947675f9`, 고정 devel은
`7bb143879dbde80a02fb4a63692556ac28fe46ac`다. 이 단계는 전체 PR 준비 완료가 아니다.

## 독립 근거와 원인

기존 `text_overlap_baseline` partition12의 `samples/basic/treatise sample.hwp`는
고정 devel에서 겹침 0건, 부모 후보에서 1건이었다. 독립
`pdf/basic/treatise sample-hwp-2020.pdf`와 `treatise sample-2022.pdf`는 5쪽 표 캡션의
같은 위치를 증명한다. 원본 입력 SHA-256은
`1aec0ca0a958d24199591b96916a2535d04f1c654f60dda1027ecbaf4839f828`, 사용 PDF는
`094549735d2ef6410e6d3928099b273ba4e7986e3feed3b16832cd998ff257aa`다.

`03e730e9eae89662c3f7b28b70752ec3b8628147`의 캡션 소유 보정이 기록한
가로 펜은 유효했지만, 그 사실만으로 현재 흐름 높이를 원본 TAC의 세로 원점으로 발행했다.
`owned_row`의 생산 결과를 `typeset.rs`가 `InlineBoxPlacement`로 기록하고,
`layout_table_control_block`이 기존 원본 세로 기준선을 덮어쓰는 경로였다.
직전 본문 끝은 양쪽 y468.4 CSSpx로 같은데, 캡션은 y505.5에서 y475.7로 올라갔다.
원본 CFB의 저장 줄은 vpos27708HU·height8114HU·baseline6897HU이며, 독립 PDF의
한글 캡션 잉크 상단은 379.2364196777344pt다.

## 공통 보정과 대조군

가로 펜만 인증된 결과에는 세로 전진 끝을 발행하지 않는다. 실제 source 줄의 원점과 현재 단의
수용 펜이 1HU 이내로 일치하거나 편집 중 재조판이 소유할 때 세로 프레임을 발행한다.
가로 배치는 기존 소유 결과를 유지하고 표·도형 paint와 첫 TAC 프레임 조회는 공통
`owns_vertical_frame()` 판정으로 세로 덮어쓰기 여부를 결정한다. 공개 struct 필드를 늘리지 않았다.

최초 후보는 기존 집중 검사 60개 중 저장 후 왼쪽 캡션 HWP 검사 1개가 실패했다.
같은 문단의 앞 개체가 실제 수용한 세로 프레임에서 유효한 저장 줄 사다리가 연속됨을 확인해
후속 개체의 세로 축을 잇도록 보완했다. 가로 펜만 있는 앞 개체, 쪽 되감기, 합성 줄,
불연속 사다리는 인증 근거로 쓰지 않는다. 문서별 offset이나 출력 숨김을 추가하지 않았다.

최종 집중 검사는 **60 PASS / 0 FAIL**, 실행 6.638초다. 새 정식 검사
`issue_7445_tac_vertical_ownership::horizontal_tac_pen_preserves_the_independent_caption_row`는
같은 CLI·입력으로 부모에서 실제 1 FAIL, 후보에서 1 PASS다. 정식 suite에서도 1 PASS(1.025초)를
확인했다. 본문 끝 → 캡션 → 소유 표의 순서·한 번의 소유와 독립 PDF 5쪽의 ±2 CSSpx를 함께 보호한다.
새 px 검사는 전 7쪽 양쪽 90% 이상, 해당 5쪽 양쪽 98% 이상과 직접 판독 후 추가했다.
다른 문서나 이전 실패 검사의 기준값은 완화하지 않았다.

## 전쪽 실제 출력

96DPI, Print, 해당 저장본 compat, PDFium153, 원래 2px raw 기준을 유지했다.

| 대상 | Native | fresh WASM | 누락·쪽수 |
| --- | --- | --- | --- |
| 논문 전 7쪽 | 최저97.75440% | 최저97.70269% | 양쪽7/7, 누락0 |
| 보정한 논문 5쪽 | 98.40204% | 98.36191% | 독립 PDF와 캡션 위치 직접 대조 |
| 기존 캡션 10조건×1쪽 | 모두100.00000% | 모두100.00000% | HWP/HWPX, 표/그림, 캡션 삭제·왼쪽 배치 |
| 기존 task1725 전242쪽 | 최저71.82002% | 최저71.95793% | 양쪽242/242, 미달201쪽 |

부모의 논문 5쪽은78.99492%였다. 고정 devel Native 논문 7쪽은 최저97.63374%이며,
후보 Native의 모든 페이지가 그 기준보다 개선됐다. 기존 task1725의 201쪽 미달은 해결하지
못했으며, 이 결과를 전체 시각 통과로 쓰지 않는다. 다른 frozen 문서 전수와 최신 전체 회귀·
게이트·실제 Studio 초기화·소비 검증은 별도 완료가 필요하다.

논문 캡션·표와 직전 본문의 분리를 Native/fresh WASM review·overlay로 직접 확인했다.
한글/영문 캡션의 줄 소속과 앞뒤 내용이 보존되며, 글자 가장자리·머리말 괘선의 잔여 차이는 있다.
왼쪽 캡션 HWP의 세 번째 표 위치와 번호1·3도 양쪽에서 직접 대조했다.
100% raw는 RGB·괘선 진하기의 완전 일치를 뜻하지 않는다.

## 생산 신원과 증적

- Native SHA-256: `dcde7f032b5049aed7b8be213c6972bca806f1eb49875e8298d0e6880e11475f`.
- fresh WASM: `19ce6f866c201c2631cb2344411a9dcdb0a90125be108b82af7ccee7aa64bee9`.
- JS: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드(206.788초).
  Docker 최적화 통과가 아니다. 루트 pkg·Studio/public·보존 package의 JS/WASM hash를 확인했다.
  실제 browser 초기화 응답은 이 단계에서 확인하지 않았다.
- [전체 TSV·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-7bb143879-20261023/tac-vertical-owner/README.md).
- ignored 실행 원본: `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage72-inline-vertical-owner/v2/`.
  `focused-result.json`, `vertical-owner-result.json`, `pixel-test-formal.log`, `producer.json`,
  `wasm-ready.json`, `consumer-visual-result.json`, `asset-audit.json`에 명령·hash·결과를 보존했다.
  완료한 WASM 묶음의 SVG/font_policy만 SHA manifest 후 정리했으며 PNG·TSV·트리·로그는 보존했다.
- fmt·manifest·unit-tier 검사를 고정 base 기준으로 수행했다. 생성 suite/manifest는 커밋하지 않는다.
