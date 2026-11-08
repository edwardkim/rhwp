# #7998 P4 다섯 번째 절편 — 보존 paint의 저장·재소비

- 기준 checkpoint: `ae05679f50298b1c202a8c49b855ebec8e4d885d`.
- 승인: 2026-10-18 다음 절편 진행 승인. B16을 우선 처리한다.
- 선행 [stage5](task_m100_7998_stage5.md)의 연속 실선·행열 편집 통과와 실패 증거는 보존한다.
- 로컬 증거 root:
  `/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-paint-save`.
  ignored `output/pr-review/issue7998/p4-paint-save`로 연결한다.

## 수정 전 규칙·원인 계층과 반례

PDF의 경계·배경 paint가 실제 셀 소유라면 표/셀의 편집 이동을 따르면서 텍스트 줄의
내용 높이와 구분돼야 한다. 가림 도형 때문에 원본 그림으로 유지한 경계를 Solid로
메우거나, 음수 좌표를 0으로 clamp해 화면만 맞추지 않는다. 원본 stroke·틈·가림·문자와
소속·순서를 보존하고 한컴의 실제 저장/인쇄 소비까지 대조한다.

현재 생성은 `structured_lowering::lower`의 원본 partition → `table_paint::anchor`의
cell bounds + padding 원점 차감 → 첫 cell Paragraph의 controls에 넣는 경로다.
그림은 `Column/Para`, BehindText, non-TAC다. 실측 첫 부모 셀의 padding.top은 945 HU,
가로 경계는 y=-970 HU, 세로 경계는 x=-25/y=-945 HU와 height=9000 HU다.
Native의 실제 도형 원점은 `layout_cell_shape_with_parent_path`가 inner-area/para 원점과
signed offset을 더한다. 비-TAC 그림의 높이는 `cell_wrap_object_visual_bottom` 등에서
별도로 소비된다. 공통 helper를 수정하기 전에 해당 값의 최종 소비까지 확인한다.

동일 source HWPX를 한컴 2020으로 HWP 재저장한 결과, 부모 Table.common.height는
18000→19890 HU로 늘었고 Cell.height=9000, padding.top=945와 그림의 offset/size는
보존됐다(`probe-before-resave.log`). 두 행의 위 padding 합 1890 HU가 차이와 대응한다.
독립 Print의 위치/선 누락은 모델 속성 유실 하나로 단정하지 않는다.

추가로 현행 [스펙 정오표 31절](../tech/hwp_spec_errata.md)을
확인했다. `flowWithText`는 한컴 UI의 **쪽 영역 안으로 제한**이며 단순한 ‘문단 추종’ 플래그가
아니다. 이전 stage5에서 해당 비트의 변경 후 인쇄 개선을 관측한 것은 유효하나 그 의미를
문단 추종으로 설명한 부분은 정정한다. 속성 이름만으로 소유/원점 계약을 추정하지 않는다.
실제 한컴 재저장본은 signed bit pattern을 보존했으므로 XSD의 nonNegativeInteger만으로
음수 좌표 저장 실패를 결론내리지 않는다. 직전 signed 문자열 진단은 반복하지 않는다.

이번 작은 대조는 root/child 경계, 실제 틈과 가림, padding과 host 문단, 그림의
상대 기준과 영역 제한을 분리한다. 조건을 좁히는 예외 대신 잘못된 원점/점유 가정을
보정한다. 실제 경계와 새 코드가 정해진 뒤 Native/fresh Docker WASM·양 저장·한컴 Print,
실제 Studio 편집/재열기를 확인한다. 새 fixture/회귀는 시각 선행 조건 뒤 등록한다.
P4의 별도 지역 격자·병합, 미확정 본문 메트릭과 P5 전체/최신 devel 게이트는 남는다.


## 실제 수정과 소비 경로

| 변경 값/규칙 | 생산 → 측정 → 실제 배치·최종 원점 |
|---|---|
| 첫 텍스트의 앞 띠 | `structured_lowering.rs:1003–1035`가 앞 띠를 ParaShape.spacing_before·첫 LineSeg.vertical_pos로 보존한다. Cell.padding을 가짜 텍스트 원점으로 만드는 fixed_lines 첫 항목 조기 반환을 제거했다. |
| 저장 lead와 개체 anchor | `cell_paragraph_spacing.rs:24`의 기존 `paragraph_before`가 실제 저장 줄·스타일에서 flow_before/stored_lead를 구한다. 새 `stored_anchor_top`은 저장 첫 텍스트 원점에서 그 lead만 뺀다. 높이 측정 `height_measurer.rs:1785`와 실제 높이/정렬 `table_layout.rs:3120`이 같은 결과를 소비한다. 기존 ‘그림 아래로 밀린 줄’은 prev_extent 경로를 유지하며 #2226 대조군을 실행했다. |
| 최종 shape 좌표 | `table_layout.rs:8637`의 비-TAC Para 경로는 text compose 전 para_y를 쓰고, `table_cell_content.rs:684`가 셀 inner-area 및 signed offset으로 실제 원점을 만든다. 현재 source는 Column/Para·Top·BehindText이고 header 음수 clamp·inline/다음 그림 주석 보정의 대상이 아니다. 측정의 wrap-bottom과 실제 top+height가 같은 정상 재저장 파일에서 닫히는지 확인했다. |
| 그림/경계의 물리 소유 | `table_paint.rs:392–520`은 bitmap뿐 아니라 채운 stroke quad도 물리 셀과 교차시킨다. 외부 잔여는 page owner로 유지한다. 축 소유에 따른 확장은 진짜 Line 및 native BorderFill로 입증된 조각에만 적용한다. native로 대표한 조각은 원본을 두 번 그리지 않으며 틈·가림은 강제 Solid로 바꾸지 않는다. |

원래 첫 셀 padding.top=945 HU와 경계의 x=-25/y=-970 같은 혼합 원점이 제거됐다.
마지막에 clamp/숨김으로 맞춘 것이 아니라 흰 가림·셀 밖 절반·자식 점선을 모두 보존했다.
진단 trace에서 기존 한컴 셀 clip은 0.48pt 채움 중 0.24pt만 남겼다. 새 정상 Print와
PDF/Native/WASM의 같은 영역에서 오른쪽 외곽·가림·실제 틈을 확인했다.

한컴이 정상 생성한 `hancom-resaved/occluded-parent-fixed.hwp`는 부모 height=18000 HU,
자식=2800 HU를 보존했다. 같은 파일의 이전 core 실제 높이는 265.2/43.7px,
보정본은 240/37.3px다(`normal-saved-before-after.json`). 저장 첫 vpos=945 HU를
그림의 문단 anchor에 다시 더하던 오류였다. 수동 작성 LineSeg로 수용 조건을 완화하지 않았다.

분할의 컷/소유 유닛·누적 예약·예산 실패 정책은 이번에 변경하지 않았다.
`table_partial.rs:1913/1922`는 기존 cut_units/line_ranges의 content-height max를 계산할 때
동일 styles/lead 결과를 받는다. 이후 cell_was_split 정렬 분기는 그대로다. 단일 쪽 source의
전체 유닛/뒤 문장은 확인했지만, 이 새 앞 띠 표현으로 실제로 쪽 경계를 자르는 복수 rowspan·
빈 밴드·각주 조합까지 실행했다고 하지 않는다. 이 전역 소비 경계의 최종 통합 검증은 P5에 남긴다.
첫 항목이 inline 개체인 경우의 기존 padding 경로도 일반화 완료 범위가 아니다.

## producer·빌드와 검증

위 기준 checkpoint 이후 제품 source 2,445개의 manifest로 빌드 입력을 고정했다.
빌드 후 제품 코드는 바꾸지 않았으며 새 테스트·fixture·문서만 등록했다.

- `source-runtime.json` SHA-256:
  `93ab70ee76264144ea4a444477fb0520fc448b270599b74d030ba5cbb27f92b9`.
- Native Skia: `native-build.log` exit 0. `rhwp-candidate` SHA-256:
  `7c339c1a818c536de8bdf6adc524e3ea2add8834456aa0887e777334169cb565`.
- fresh Docker WASM: `docker-wasm.log` exit 0 / 완료 뒤 복사, `wasm-build-completion.json`.
  19,206,602 bytes, SHA-256:
  `15a40eab4e2b525eb03b6a143c7becc54df33f5c2d53f5bd84e7072d82d837f1`.
  실제 Studio·CDP 응답도 같은 해시다. 사용자 Studio 7700의 파일은 변경하지 않았다.
- 공유 `/home/edward/mygithub/rhwp/target/pr-review` 및 Docker volume을 재사용했다.
  앞선 라이브러리 snapshot은 실제 이전 producer가 아니어서 before 증거에서 제외했다
  (`library-snapshot-disposition.txt`). before는 보존한 B15 CLI와 실제 입력의 실행이다.

| 이번 실행 범위 | 결과와 증거 |
|---|---|
| 원본 62개 × PDF/HWP/HWPX | Native/fresh WASM 각각 186건·219쪽, 최저 **97.45634% / 97.41574%**. 90% 미만·누락·오류·쪽수 차이 0. 무편집 양 저장 재열기 PNG는 같은 backend의 직접 PDF 출력과 바이트 동일. `native/results.json`, `wasm/results.json`. |
| 직전 B15와 공통 전쪽 비교 | 각 backend 219쪽 중 126쪽 PNG 동일/93쪽 변경, **0.01%p 이상 하락 0**. Decimal로 원본 TSV를 비교했다. `native-base-comparison.json`, `wasm-base-comparison.json`. 최신 upstream/devel 비교는 P5다. |
| 독립 한컴 Print, default source 2개·Arial source/X 편집/삭제 복원 4개, 양 형식 | 각각 28쪽, 최저 **97.42375% / 96.78954%**. 가림 default source는 **99.12616% / 99.23890%**, 점선 default source는 **99.18575% / 99.29236%**. `native-edited-results.json`, `wasm-edited-results.json`. |
| 정상 한컴 재저장 HWP × 새 source Print | 1쪽씩 **99.11651% / 99.25841%**. `native-extra-visual/hancom-resaved/silhouette.tsv`, `wasm-extra-visual/hancom-resaved/silhouette.tsv`. 실제 source와 한컴 정상 저장본을 구분했다. |
| 실제 Native/Studio API 문자 편집 | 4개 가족의 X 삽입·삭제, HWP/HWPX 재열기·실제 부모/자식 프레임·셀 포함·문자 순서 복원 PASS. `cases.log`, `studio-result.json`. WASM 저장본은 실제 fresh WASM이 생성했다. |
| 영향받는 기존 행·열·테두리 편집/삭제 복원 | 새 source로 Native/실제 Studio API를 실행하고 각각 양 형식 20쪽을 B15 Print와 대조, 최저 **97.42375% / 96.81040%**. `row-column/cases.log`, `row-column/studio-result.json`, `row-column/*-edited-results.json`. |
| 집중 정식 회귀 | 새 `issue_7998_pdf_cell_paint_frame` 2건 + composition 8·border editing 3·table inference 17·body paragraphs 4·#5593/#2226/#6569 각 1 = **37건 통과**. `focused-final-results.json`, `new-regression-final.log`. |

행열 편집의 B15 Print는 이전 source의 잘린 경계도 포함한 자료이므로 현재 완전한 stroke와
차이가 있다. 그 동일 Print 점수는 B15 대비 최대 Native **0.99012%p**, WASM **0.95516%p**
낮아졌으며 위의 원본 PDF 대비 219쪽 무하락 주장과 구분한다. 이를 새 기준으로 덮어쓰지 않았다.
`row-column/row-review.png`에서 이전 Print/이전 Native/현재 Native를 같은 배율로 대조했다.
부모·자식 위치와 앞뒤 문장·빈 새 행은 보존되고 물리 분할된 점선의 양쪽 면적이 복원된다.
새 독립 Arial X 편집 Print의 Native는 8쪽 모두 100%, fresh WASM 최저는 99.22952%이며, 소유/좌표가 맞는 경계와
기존 Print의 잘린 선을 별도로 읽었다. 독립 PDF와의 point 점수만으로 의미 보존을 대신하지 않는다.

명령은 results JSON의 `command` 및 각 실행 로그에 있다. 새 테스트는 기존 자동 수집과
`node scripts/rust-test-suite-manifest.mjs --prepare` / `scripts/run-rust-test.mjs`를 사용했다.
CI 호출·baseline·래칫 허용치는 추가하거나 완화하지 않았다. 첫 테스트 실행은 검사용 문자
수집에서 Shape TextBox를 빠뜨려 실패했고 수집을 고친 뒤 2건 모두 통과했다. 제품 실패나
수정 전 결함 재현으로 세지 않는다. 정상 저장본의 frame 검출은 실제 이전 CLI FAIL/후보 PASS다.

## 직접 판정·fixture와 남은 범위

2026-10-18 메인테이너는 다음 자료와 실제 Studio에서 **“시각·편집 구조 통과”**로 판정했다.

- 비교: `E:\tmp\issue7998-semantic-table-review\p4-paint-save\index.html`
  (`/mnt/e/tmp/issue7998-semantic-table-review/p4-paint-save/index.html`). 상대 링크로
  PDF·저장본·한컴 Print·Native/WASM PNG·TSV를 연결했다.
- Studio 가림: `http://localhost:7793/?review=issue7998-p4-paint-occluded-parent&lang=ko`.
- Studio 점선: `http://localhost:7793/?review=issue7998-p4-paint-periodic-parent&lang=ko`.

새 fixture의 생성·독립 Print·한컴 정상 저장 절차와 SHA-256은
[`paint-save-sources.md`](../../pdf/issue7998/paint-save-sources.md)에 있다.
관계 검사는 실제 프레임/셀 포함·문자 순서·누락/중복·앞뒤 문장을 보호한다.
이전 stage5의 미달 원본·TSV·Print와 실패 기록은 보존했다.

B16의 **source·문자 편집 저장 계약**을 완료했으며 비표준 테두리의 일반 스타일 교체·
새 행열 paint 상속, 지역/병합 격자·unequal span의 독립 근거, 미확정 본문 수평 메트릭과
B08의 원래 Helvetica 글꼴 대체는 남는다. 전체 Rust/lint/package·Worker·최신 devel과의
최종 비교 및 PR 게이트도 P5에서 수행한다. `final_pr_ready=false`이며 P4 전체 완료가 아니다.
원격 push·PR 생성·GitHub 댓글/상태 변경은 수행하지 않았다.

최종 rustfmt와 PDF를 제외한 staged diff whitespace 검사를 통과했다. 새 PDF의 xref 행 끝
공백은 포맷 원본 바이트이므로 수정하지 않았다. 공급 글꼴·비밀 환경 파일·실행 로그는
커밋 범위에 포함하지 않았다. 두 PDF의 생성기 바이트 재현도 확인했다.
