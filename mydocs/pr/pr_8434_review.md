# PR #8434 리뷰 — fix(parser/hwp5): 선언 0·실제 1문단인 사각형 글상자 목록을 손상으로 거부하지 않는다 (#8432)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8434](https://github.com/edwardkim/rhwp/pull/8434) · `planet6897` → `devel` |
| 원 head | `9835f2309c32812dce0037cb3e654e61d9b44b38` |
| 통합 적용 커밋 | `9276ae3ae6b6ae96890012bdfa6617ec3735c87b, a0517fe6fb5fc6fb333edaffcc10cda266c617fe` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8432](https://github.com/edwardkim/rhwp/issues/8432) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(parser/hwp5): 선언 0·실제 1문단인 사각형 글상자 목록을 손상으로 거부하지 않는다 (#8432)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156690773-declared-zero-textbox.hwp](../../samples/issue8432/156690773-declared-zero-textbox.hwp) · [156690773-declared-zero-textbox-2022.pdf](../../pdf/issue8432/156690773-declared-zero-textbox-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8434/pr8434-156690773-declared-zero-textbox-b4bf7233-native.tsv) · 99.13% (3쪽) | [TSV](assets/planet6897_20261103/pr_8434/pr8434-156690773-declared-zero-textbox-b4bf7233-wasm.tsv) · 100.0% (3쪽) |
| [156761223-declared-zero-textbox.hwp](../../samples/issue8432/156761223-declared-zero-textbox.hwp) · [156761223-declared-zero-textbox-2022.pdf](../../pdf/issue8432/156761223-declared-zero-textbox-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8434/pr8434-156761223-declared-zero-textbox-c541ec05-native.tsv) · 99.97% (3쪽) | [TSV](assets/planet6897_20261103/pr_8434/pr8434-156761223-declared-zero-textbox-c541ec05-wasm.tsv) · 99.99% (3쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

파서의 글상자 소유 문단 검증에 적용합니다. 원본 레코드와 선언 개수·실제 소유 내용을 대조하고, 아래 전쪽 출력으로 내용 보존을 확인합니다.

## 구현 주장과 검증 증거의 대조

사각형 글상자 목록의 선언 문단 수가 0이어도 실제 소유 문단 레코드가 있으면 손상으로 거부하지 않는다.

src/parser/body_text/drawing_text_structure.rs의 Owner::finish_list가 소유 레벨에서 집계한 실제 문단 수를 검증에 사용한다. 개수 선언만으로 내용을 생략하거나 할당하지 않는다.

선언 0·실제 1인 두 실물 입력을 사용한다. 선언 0·실제 0인 기존 손상 입력과 선언보다 문단이 모자란 구조의 거부는 유지한다. base의 파싱 거부와 후보의 전쪽 출력은 서로 다른 결과로 기록한다.

관련 정식 회귀 파일: [issue_8432_declared_zero_textbox_list.rs](../../tests/cases/issue_8432_declared_zero_textbox_list.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156690773-declared-zero-textbox.hwp` 2쪽 · 다른 선언 0 글상자 | 100.00% · [review](assets/planet6897_20261103/pr_8434/pr8434-claim-p002-156690773-declared-native-review.png) / [overlay](assets/planet6897_20261103/pr_8434/pr8434-claim-p002-156690773-declared-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8434/pr8434-claim-p002-156690773-declared-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8434/pr8434-claim-p002-156690773-declared-wasm-overlay.png) |
| `156761223-declared-zero-textbox.hwp` 3쪽 · 선언 0 글상자 | 100.00% · [review](assets/planet6897_20261103/pr_8434/pr8434-claim-p003-156761223-declared-native-review.png) / [overlay](assets/planet6897_20261103/pr_8434/pr8434-claim-p003-156761223-declared-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8434/pr8434-claim-p003-156761223-declared-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8434/pr8434-claim-p003-156761223-declared-wasm-overlay.png) |

## 현재 대표 페이지 직접 판독

[판독 기록·실제 PNG SHA-256](assets/planet6897_20261103/direct-reading-current.md)에 같은 원본·쪽의 관측을 남겼습니다.

- native 2쪽: 선언 0 글상자 입력이 정상 해석돼 본문·담당 부서 표·배너와 정책브리핑 로고를 보존한다. 독립 PDF와 표 원점·뒤 로고의 비겹침·내용 순서를 확인했다.
- wasm 2쪽: fresh WASM에서도 선언 0 글상자 문서의 본문·담당 부서 표·하단 배너 및 로고가 독립 PDF와 같은 순서·위치로 보존된다. 누락/겹침은 보이지 않는다.
- native 3쪽: 선언 0 글상자 입력의 마지막 본문, 담당 부서 표, 정책브리핑·OPEN 로고가 같은 쪽에 차례로 보존된다. 표 외곽과 두 행, 표 아래 로고의 간격·비겹침을 대조했다.
- wasm 3쪽: 선언 0 글상자 입력의 마지막 본문, 담당 부서 표, 정책브리핑·OPEN 로고가 같은 쪽에 차례로 보존된다. 표 외곽과 두 행, 표 아래 로고의 간격·비겹침을 대조했다.

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->

## 최신 리베이스 이후 통합 검증 상태

최신 base는 `f529dc92d4d172bf32390d7749743a45d9e41bfc`, 검증 시작 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`다. 원 PR 39개와 메인터너 보정을 담은
168개 커밋을 충돌 0건으로 리베이스했고 range-diff 전 행이 `=`로 대응했다.
[원 head와 현재 적용 SHA](assets/planet6897_20261103/integration-commits.md)를 구분한다.

최신 source의 최소 글꼴 Rust·Native Skia·lint·최적화 WASM 및 영향 경계 검증은 진행 중이다.
완료 판정이나 최종 승인으로 표시하지 않는다. 검증 중 소스 해시를 고정하고 문서 갱신과 구분한다.
충돌 없는 리베이스 뒤 기존 전쪽 비교를 반복하지 않는 작업지시자의 범위를 따른다.
새 base 기능은 같은 시스템 글꼴 환경의 실제 base/candidate 출력과 영향 경계로 대조한다.
새 base 대비 허용 하락은 0.1%p이며, 보호 PR #8389·#8397은 원 TSV의 소수 둘째 자리
표시값 하락을 허용하지 않는다. 기존 TSV·PNG의 producer는 원래 출처대로 유지한다.

직전 c183 제품에서는 [최소 글꼴 Rust 12,563 PASS와 필수 40개 실행](assets/planet6897_20261103/rust-validation-c183a1ac61.md),
[Native 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/native-validation.md),
[fresh WASM 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md),
[실제 브라우저/API 실행](assets/planet6897_20261103/runtime-validation-c183a1ac61.md)을 완료했다.
양쪽 전쪽 최저는 각각 92.45232%·92.87288%다. 이 결과를 최신 리베이스 제품의 재출력으로
취급하지 않는다. 보호 범위 Native는 17문서·1,770쪽·원 TSV 2,153행의 하락·누락 0행이며,
같은 c183 WASM 보호 범위의 남은 집계를 진행 중이다. [공식 증거 목록](assets/planet6897_20261103/README.md)과
[재실행 명령·입력·Print PDF·조판 세대·글꼴 계약](assets/planet6897_20261103/visual-reproduction.md)을 연결한다.

원 PR별 주장 쪽의 직접 판독·회귀 주석·최종 본문 감사를 마친 뒤 수용 판정을 확정한다.
통합 PR 생성·push·원격 review·comment·close는 아직 수행하지 않았다. PR 생성 전 승인을 기다린다.
