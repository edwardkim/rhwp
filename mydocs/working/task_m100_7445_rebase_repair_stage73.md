# #7445 보정 73단계: 어울림 탭 줄의 본문 점유 보존

Issue: #7445. 부모 `f1d29dfcd913ae727d394b5e4a3788130455f3c8`, 고정 devel
`7bb143879dbde80a02fb4a63692556ac28fe46ac`. 기존 각주 문서와 기존 정상 대조군만 확인했다.
새 HWP 보정 대상은 추가하지 않았다. 이 단계는 전체 PR 검증 완료가 아니다.

## 원 해결과 독립 근거

#4090/#4094의 `9f564bbeea841a6d03c3c38b67f45a7439b95323`는 Square 표의 높이를
본문에 무조건 더하지 않고 옆 차선을 배제 밴드로 소비한다. #6128은 후속 어울림 본문의
저장 끝점을 측정과 paint에서 보존하며, #7280 `d32884bfb4`는 그 Query와 상태 확정을 분리했다.
이 계약을 유지하며, 실제 공간을 점유하는 빈 글줄을 보이지 않는 표시와 구분했다.

입력은 `samples/task1725/text_footnote_tail_overpagination.hwp`, SHA-256
`d335c26cbcd57b157c929ed36596d95a6676bea5624348baae3bff3b55a705d9`다.
독립 Print는 `pdf/task1725/text_footnote_tail_overpagination-hwp-2024.pdf`, SHA-256
`03ddf66494ebd1b25cfe49344039f0c6e324a0a837d3d4dce9f26bc14141e8af`, 전242쪽이다.

원본 CFB/레코드를 독립적으로 읽은 201쪽의 host4348은 마지막 저장 줄 vpos35328HU,
그 뒤 탭 문단4349는37248/39168/41088HU의3줄, 다음 주석4350은43008HU에서 시작한다.
각 줄은 높이1200HU+간격720HU이며 앞뒤·내부 사다리가 정확히 닫힌다.
독립 PDF의 주석 첫 줄 잉크 상단486.61368pt도 이 위치에 대응한다.

실제 진단에서 현재 단의 base0, col_anchor0, 흐름 펜496.64px는 탭 첫 줄37248HU와
정확히 일치했다. 표는 Para/Square이며 별도의 square_band_top은 없었다.
전체 탭 문단을 `is_empty_para`만으로 높이 없이 흡수해3줄76.8px를 잃었고, 주석과 후속 표가
앞당겨졌다. 실제 상태 확인 뒤 원인을 확정했으며, 밴드 부재나 host의 첫 줄 좌표만으로
본문 소유를 추정하지 않았다. 관측용 임시 진단 소스는 복원했고 최종 구현에 포함하지 않았다.

## 공통 생산·소비 보정

`stored_empty_flow_lines`가 기존 단일 빈 줄 검증 조건으로 원본 줄 묶음을 확인한다.
기존 단일 줄 소비자들은 계속 정확히1줄만 받는다. 닫힌 source box Query는 앞뒤뿐 아니라
내부 모든 줄 간격도 확인한다. 합성 줄·편집·불연속·쪽 경계·유효하지 않은 높이는 수용하지 않는다.

`TypesetState::stored_wrap_rows_start_at_flow_pen`은 현재 단의 확정 source 축에서 첫 줄이
실제 미소비 흐름 펜과1HU 이내로 일치하는지 조회한다. 이 증거와 닫힌 줄 묶음이 모두 있을 때
일반 본문 경로에 남긴다. 절대 위치 표의 기존 계약도 보존한다.
`wrap_flow`는 같은 저장 차선 anchor를 등록하고, 일반 문단 측정·항목 배치·paint가
동일한3줄을 소비한다. 개체 높이 재가산, 문서별76.8px 보정, clamp·누락 숨김은 없다.

## 실제 검증

96DPI / Print / compat2024 / PDFium153 / 원래2px raw 기준이다.

| 범위 | Native | fresh WASM |
| --- | --- | --- |
| 원본 전242쪽 | 최저92.66506%, 미달0 | 최저92.53301%, 미달0 |
| 보정 전201쪽 | 71.82002% | 71.95793% |
| 보정 후201쪽 | 94.40016% | 94.32516% |
| 기존 논문 전7쪽 | 최저97.75440%, 미달0 | 최저97.70269%, 미달0 |

양쪽 모두242/242쪽이며 누락·중복 쪽이 없다. PNG/PDF PNG/RenderTree도 각242개를 감사했다.
부모 대비 raw가 바뀐 쪽은 양쪽 각각201쪽뿐이며 나머지241쪽은 값이 같다.
논문 전7쪽 TSV는72단계와 양쪽 바이트가 같다.

고정 devel Native의 같은 원본 전242쪽은45·201쪽 미달, 최저74.38471%였다.
후보는 devel 대비 최대 하락0.63154%p, 2%p 이상 하락0쪽이다. 201쪽은 devel보다도 개선됐다.
이 Native 비교를 fresh WASM base 비교로 대신하지 않는다. frozen devel Native는605ccfd9의
보존 바이너리이며7bb143879까지 runtime source·assets·글꼴·Cargo·내부 crate·vendor·build.rs의
동일성을 현재 Git diff로 재확인했다.

관련 기존 어울림·빈 문단·그림·전폭 표·줄 소유 대조군은 **44 PASS / 0 FAIL**, 실행2.340초다.
`issue_7445_wrap_blank_source_rows::closed_tab_rows_keep_their_space_before_the_following_note`는
부모 CLI에서 실제1 FAIL(탭 줄0개/기대3개), 후보에서1 PASS였고 정식 suite에서도
**1 PASS**(1.899초)를 확인했다. 새 검사는 양쪽 전242쪽90% 이상 이후 등록했다.
소유 문단을 원문 내용으로 식별하고 source 간격·줄 개수·주석 본문 전체의 순서·누락·중복을
확인한다. 201쪽은98% 미만이므로 절대 px 핀을 추가하지 않았다.

Native/fresh WASM의201쪽 review·overlay를 독립 Print와 직접 열어 대조했다.
주석이 표 오른쪽 차선에서 시작해 아래 전폭으로 복귀하고, 후속 별표2와 표도 아래에 보존된다.
표 괘선·일부 글자 가장자리와 수식/첨자 주변 미세 차이가 남으며 RGB 일치로 보고하지 않는다.

## 생산 신원·증적·잔여

- Native SHA-256: `e9908523191429df98fedf4f6b15afec0ecea7dcb7c3723b41215e5a1ac08b13`.
- fresh WASM: `5f254d76307ba9be5b10f5c81af29c940a51c3ad5523218b6fb0c9169fc4bee1`.
- JS: `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화
  통과가 아니다. pkg·Studio/public·보존 package의 JS/WASM hash가 같다.
  실제 browser 초기화·전체 제품 소비 검증은 아직 남아 있다.
- [전쪽 TSV·전후 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-7bb143879-20261023/wrap-blank-source-rows/README.md).
- ignored 원본: `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage73-wrap-source-slots/`.
  실제 상태·진단·명령·비교·hash는 `state.json`, `wrap-source.log`, `model.log`,
  `base-runtime-equivalence.json`, `candidate/producer.json`, `focused-result.json`,
  `consumer-visual-result.json`, `blank-wrap-result.json`, `new-regression-result.json`,
  `native-develop-comparison.json`, `final-audit.json`에 보존한다.
  완료 SVG/font_policy만 묶음별 SHA manifest 후 정리했으며 원본·PDF·PNG·TSV·로그는 유지한다.
- fmt·manifest·unit-tier 검사를 고정 base와 비교했다. 생성 suite/manifest는 제출하지 않는다.
- 기존 비트맵 정상 대조군의 색상 raw94.57727%(devel98.85995% 대비4.28268%p 하락),
  faint grid42.89522%는 현재 후보에서도 남았다. 새 보정 대상으로 확대하지 않고 기존 잔여로
  계속 처리한다. 전체 frozen 문서·최신 전체 Rust/필수 게이트·WASM base 대조·실제 소비 검증과
  최신 devel 확인을 끝내기 전 PR을 제출하지 않는다.
