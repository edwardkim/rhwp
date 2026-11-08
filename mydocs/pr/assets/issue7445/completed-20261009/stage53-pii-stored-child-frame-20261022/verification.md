# 개인정보 65→66쪽: 저장된 자식 프레임의 부모 컷 소유

- Issue: #7445, 기존 #5846 범위. parent `247988f3e981b9c4d9d56c24a1dc9f98457ec82b`, base `f7abae4a8e70c062c378db06ad64d2d2eacb88cd`.
- 원본: `samples/task2097/75544_pii_bunseok.hwpx`, SHA-256 `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 기준: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, Print 2020, 66쪽, SHA-256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
- 선언 글꼴 환경: `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`. 전체 글꼴 파일/face는 ignored `rebase-7752-20261022/candidate-source.json`의 713개 파일 hash를 실행 직전에 모두 확인했다.
- Native CLI: `native-skia`, `release-test`, SHA-256 `4667a136fc4a83a7c5a7219380407e2d78e0e397567bf79e85fb475f243b01d3`. 소스 patch SHA-256 `08cc26e53d910d7994de2a294c703f250f64a348dbbcfb5fc5feb9371a1f2c96`.
- 비교: canonical `scripts/visual_sweep.py`, Native Skia PNG, 96 DPI, compat 2022, PDFium `152.0.7947.0` / `pdfium_smooth_rgb_v1`, 색상 경계 보정 전 2px raw 90% 문턱, 캡처 총 8. PDF rasterizer/tool/글꼴 hash와 명령은 ignored producer.json에 보존했다.

## 원인과 공통 소비

원 해결 계보는 #5846 → #5904 → #5912(`b9eb551`), #7642(`363ef22d2c7`)다. 기존 `stored_nested_row_frames`가 앞 행 9864 HU + 자식 첫 상자 1794 HU + 부모 위/아래 안 여백 223 HU씩 = 부모 선언 12104 HU를 닫고, 자식 종료와 부모 다음 줄 원점 6262 HU를 함께 증명한다. 이 증명을 현재 정규화 프레임 원장에 전달한다.

`원본 종료 증명 → 저장 프레임/원본 줄 유닛 → mixed 자식 RowCut → 부모 저장 경계 조회 → 단일 유닛 이월 판정 → 같은 컷의 row_cut_height/table_partial` 경로를 확인했다. 부모 글줄 범위가 없는 재귀 자식을 평문 줄로 판정하던 조건을 자식의 정확한 인증 프레임 조회로 바꾸고, 별도 단일 유닛 이월 가드도 같은 판정을 소비한다. 선언이 부모 아래 안 여백까지 닫았다는 사실도 같은 원장으로 기존 padding trim 소비자에 전달한다. 일반 reset과 편집으로 무효화된 저장 정보는 기존 경로를 유지한다. 문서 ID·쪽·높이 상수 분기나 검사 완화는 없다.

## 실행 결과

- 기존 `issue_5846_cut_fragment_defers_partial_nested_tail_row`: 수정 전 원본 칸 `(4, 1)` 소실로 FAIL → 수정 후 PASS. 원본 검사를 변경하지 않았다.
- P47: 관련 123건 모두 PASS, 테스트 22.048초. 아래 안 여백 직접 판독 잔여를 추가 보정한 P48도 **123 PASS / 0 FAIL**, 테스트 **21.433초**, 필터로 제외된 5793건. 전체 nextest가 아니다. #8155 신규 1건과 #3930 기존 2건을 포함한다.
- P48 Native 66/66쪽, 65쪽 `95.40934 → 100.00000%`, 66쪽 `60.81274 → 100.00000%`. 전쪽 최저 46쪽 `96.57165%`, 미달 0쪽.
- Native PNG 변경은 65·66쪽뿐이며 다른 64쪽은 byte 동일하다. 구조 소유별 연결 문자열 1826개는 전후 동일하다. 이 집계만으로 실제 출력이나 전체 순서를 대체하지 않으며, 기존 검사의 쪽/줄 소속·포함·내용 순서·중복·누락·작은따옴표·후속 host 관계를 함께 실행했다.
- 65·66쪽 review/overlay를 직접 판독했다. 첫 줄은 65쪽에, 뒤 네 줄은 66쪽에 있으며, 첫 조각 상하 여백과 종료 외곽을 독립 PDF와 대조했다. `100%`는 2px 내용 실루엣이며 AA·색상까지 동일하다는 뜻이 아니다. 최저 46쪽의 기존 테두리 꼬리 차이는 변경하지 않았다.

[수정 전 전66쪽 TSV](before-native-silhouette.tsv) · [수정 후 전66쪽 TSV](after-native-silhouette.tsv)

| 쪽 | 전 review / overlay | 후 review / overlay |
| --- | --- | --- |
| 65 | [review](before-native-p065-review.png) / [overlay](before-native-p065-overlay.png) | [review](after-native-p065-review.png) / [overlay](after-native-p065-overlay.png) |
| 66 | [review](before-native-p066-review.png) / [overlay](before-native-p066-overlay.png) | [review](after-native-p066-review.png) / [overlay](after-native-p066-overlay.png) |

## 출처와 잔여

현재 Runtime source hash:

- `src/renderer/layout/table_layout.rs`: `309e3061d740e34dc3674377c34ec6a61934fed4c2ec2081a19df78c8cb989ba`
- `src/renderer/layout/table_layout/stored_nested_frame.rs`: `9876754c38800f333953d05e5767ce441289cb5d5ab5ea0168289ca3dd58ee97`
- `src/renderer/render_normalization/stored_nested_text_frames.rs`: `4eb6ae10a1391cc1b35f2c2c852f3ad8e439d999f4345fee38255ab630283d94`
- `src/renderer/typeset/table/scan/runner/row_step.rs`: `faa780521109a75e01a44d973b7607e107fee135e0ee9758b07530415fd469a5`

임시 진단 P43 snapshot 등록은 출력 효과가 없어 수용하지 않았고 P44/P46 추적 코드도 제거했다. 실행별 patch·source/build hash·실제 로그는 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/`에 있다. `pii-closed-parent-padding-v6/nextest.log`, `contract.log`, `tree.log`, `pii-closed-parent-padding-v6-visual/producer.json`, `comparison.json`과 전66쪽 PNG/TSV를 보존했다.

최신 source의 fresh WASM, 전체 nextest, lint/build/Native Skia 3종, manifest/unit tiers, 실제 소비 경로와 fixed base 대비 전쪽 비교는 아직 미완료다. 이 Native 단계로 PR 준비 완료를 주장하지 않는다. 기존 편람242·277쪽을 새 보정 대상으로 확대하지 않았고, HWP/HWPX·PDF·회귀 기대값·지침·원격은 변경하지 않았다.
