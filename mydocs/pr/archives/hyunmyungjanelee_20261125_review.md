# hyunmyungJaneLee의 셀 안 개체 배치 PR 3건 검토

## 최종 판정

**메인터너 보정 후 수용 가능.** 세 PR은 모두 `devel`을 대상으로 하고 리뷰어는 `edwardkim`이다. 원 head 세 개를 보존한 뒤 기여자가 제공한 HWP·HWPX, 한컴 직접 인쇄, 반례를 기준으로 보완했다. 최초 제출 때 검증한 제품 소스는 `b12b85ad6ec037fff9ce41ac6fc34b8865d93920`, 비교한 `devel`은 `50324d194d1cd7c46d168f07a58515086530f03b`다. [접수·원 head](hyunmyungjanelee_20261125_review_plan.md) · [보완 전 반례](hyunmyungjanelee_20261125_review_impl.md)

#9415의 재저장 글상자와 크기 보호 처리를 병합한 제품 소스 `f3d137d9cbe36cae93f859baac3a29ae4a543f75`에서 같은 입력·한컴 Print로 다시 검증했다. 전체 회귀 14,166건, Native Skia 4,227건과 세 PR 및 새 기준의 집중 회귀 64건을 통과했다. [현재 기준에서 세 PR의 쪽별 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-9415/README.md)

| 원 PR | 기여와 메인테이너 보완 결과 |
|---|---|
| [#9352](pr_9352_review.md) | 칸 밖에 놓인 종이 기준 도형의 위치를 보존했다. 흰 도형 안으로 보이던 표 괘선 130줄이 0줄이 됐다. 글상자 본문의 기존 수직 정렬도 유지했다. |
| [#9384](pr_9384_review.md) | 저장 줄이 없는 사진의 행 높이를 실제 가로 겹침으로 정한다. 왼쪽·가운데·오른쪽 정렬과 음수 오프셋에서도 겹치는 두 사진은 한컴과 같은 순서로 맞닿는다. |
| [#9407](pr_9407_review.md) | 중첩 표의 첫 조각과 이어받기가 실제로 수용한 행·물리 높이를 공유한다. 10mm 아래 여백에서 둘째 쪽의 첫 행이 27번에서 한컴과 같은 26번으로 바뀌었다. |

최초 제출 당시 최소 글꼴 6개만 노출한 환경에서 전체 회귀는 14,151개가 통과하고 실행 목록에서 45개가 제외됐다. 집중 회귀 33개도 모두 통과했다. 보정 전 실패를 확인한 일곱 회귀를 최종 코드에서도 실행했다. Native Skia 공식 검사 4,227개와 Rust 정렬, Native·WASM·워크스페이스 Clippy, 전체 workspace build를 마쳤다. [검사별 결과](../assets/hyunmyungjanelee-20261125/maintainer/checks.tsv) · [수정 전후의 일곱 회귀](../assets/hyunmyungjanelee-20261125/maintainer/regression-controls.tsv)

추가한 HWP·HWPX 58개 모두 같은 입력으로 인쇄한 한컴 PDF와 쪽수가 같다. 58개를 한꺼번에 검사하고 각 파일을 따로 전달해서도 숨은 글자·프롬프트 주입 신호·유니코드 기만 검사를 통과했다. 짧은 용지 대조 문서 두 개는 한컴이 인쇄한 사진 수와 끝에 걸린 행을 기준으로 고정했다. [입력·인쇄본·쪽수](../assets/hyunmyungjanelee-20261125/maintainer/new-sample-checks.tsv) · [용지 끝의 세 출력 비교](../assets/hyunmyungjanelee-20261125/maintainer/photo-page-edge.png)

Native·브라우저 각각 71개 조합·110쪽을 검증했다. 두 backend에서 보호 문서 23종의 공식값 4,046행을 모두 충족했고, 편람 HWPX Native 383쪽도 통과했다. 화학물질 표시 기준 WASM 26쪽은 같은 환경의 devel·후보가 모두 100.00%여서 기존 하한 99.99%를 100.00%로 상향하는 변경을 포함했다. [목표 문서 실측](../assets/hyunmyungjanelee-20261125/maintainer/README.md) · [보호 문서 실측](../assets/hyunmyungjanelee-20261125/maintainer/protected/README.md)

두 소스에 공통인 99쪽 중 Native·브라우저 각각 86쪽에서 보정본의 일치율이 올랐고 13쪽은 같았다. 당시 devel에서 한 쪽이 더 나오던 12개 조합과 한 쪽이 모자라던 11개 조합도 보정본에서는 인쇄본과 쪽수가 같다. [최초 제출 기준과 후보의 쪽별 수치·이미지 해시](../assets/hyunmyungjanelee-20261125/maintainer/devel-target-pages.tsv)

## 최신 devel 병합 후 검증

진행 중에 병합된 #9411의 `0521a1df673e7f64d8bf7ef785a6a60001d3b9c3`를 반영한 제품 소스는 `8d1403eb14bbefb35690a50b6734b0c2ef9ba5cb`다. 두 소스에서 다시 빌드한 Native·브라우저로 71개 조합을 확인했다. 직전 제출 때 판독한 후보의 220개 PNG는 이번 병합 뒤에도 모두 같았다. 세 PR의 보완과 #9411의 수당표·식량안보 보고서·제주 교육계획·KAF 문서 조판을 함께 검증했다. 전체 회귀 14,164건, Native Skia 공식 검사 4,227건과 집중 회귀 62건이 통과했다. [최신 기준에서 검증한 쪽별 수치·이미지 해시와 검사 결과](../assets/hyunmyungjanelee-20261125/maintainer/current-develop/README.md)

작업지시자가 이번 PR에 한해 허용한 직전 소형 OTF는 개요 번호 33개와 종합부동산세법 12개다. 두 소스의 Native와 새 WASM에서 개요 번호 9쪽은 모두 100.00%였고, 종합부동산세법 6쪽은 각 쪽의 공식 측정값을 재현했다. 두 문서의 PNG는 각각 18쌍과 12쌍 모두 바이트 단위로 일치했다. [공급 순서와 두 SHA-256](../assets/hyunmyungjanelee-20261125/maintainer/current-develop/pr-font-programs.tsv)

## 조판 원칙과 검증 근거

| 검토 항목 | 판정 | 확인한 근거 |
|---|---|---|
| 제출 전 로컬 검증과 원 PR 상태 | 충족 | 접수 당시 세 원 PR은 모두 Open이었다. 위의 빌드·집중/전체 회귀·최적화 fresh WASM의 실제 브라우저 출력을 사용했다. |
| 기존 이슈와 해결 PR | 충족 | #9352·#9384의 이전 셀 그림 배치와 #9407의 행 분할 계보를 각 원 PR 검토에 기록하고 기존 셀 그림 세 문서·음수 오프셋 두 문서에서 보존을 확인했다. |
| 구현 근거·일반성 | 충족 | 같은 위치에서 정렬·글 뒤 배치·여백 한 속성씩 바꿔 다시 인쇄했다. [그리기·물리 좌표·행 소속](../assets/hyunmyungjanelee-20261125/maintainer/placement.md) |
| 측정·배치·최종 출력 | 충족 | 그림은 공통 위치 계산, 표는 동일한 행 컷을 사용한다. Native PNG와 새 WASM Canvas2D 각각의 출력과 manifest를 확보했다. [제품별 측정값](../assets/hyunmyungjanelee-20261125/maintainer/page-comparison.tsv) |
| 최신 기준의 기능 보존 | 충족 | 최신 devel의 보호 문서와 기존 셀 내용 그림·나란한 그림을 같은 입력·글꼴로 대조했다. 새 devel 병합 시 재검증 대상은 원 PR별 계획을 따른다. |
| 분할·이어받기·줄의 점유 | 충족 | 저장 줄이 있는 실제 재저장본, 저장 줄이 없는 원본, 마지막 중첩 행 소비 뒤 바깥 행만 남는 경우의 소속과 외곽선·후속 본문을 검증했다. [쪽별 행 소속](../assets/hyunmyungjanelee-20261125/maintainer/row-ownership.tsv) |
| 독립성·회귀 선행 조건 | 충족 | 한컴 직접 인쇄와 Native·WASM을 쪽별로 비교하고 관련 직접 인쇄 하한을 충족한 뒤 관계형·쪽 소속 회귀를 등록했다. 제출 HWP와 형제 HWPX 인쇄 대조는 따로 분류했다. [입력과 인쇄본](../assets/hyunmyungjanelee-20261125/maintainer/inputs.tsv) |
| 기준값 변경 | 충족 | 시각 보호 하한 중 한 행을 0.01%p 상향했다. 새로 추가한 두 짧은 용지 입력에는 의도한 잘림에 대응하는 진단 다섯 행을 최초 등록했다. [보호 하한 변경](../assets/visual-protected-baseline/updates/pr9352-9384-9407-20261125.json) · [추가 입력의 세 소스 측정](../assets/hyunmyungjanelee-20261125/maintainer/photo-page-edge.tsv) |
| 확대 판독·구현 주장 | 충족 | 겹치는 괘선·사진 경계·중첩 표의 다음 쪽과 편람 직인 경계를 한컴과 직접 비교했다. 작성자의 원본 TSV 46행과 통합 후보의 같은 입력·인쇄본 결과를 대조했다. [검토 그림과 측정](../assets/hyunmyungjanelee-20261125/maintainer/full-review.tsv) · [원 head 대조](../assets/hyunmyungjanelee-20261125/maintainer/original-pr-comparison.tsv) |

## 글꼴과 인쇄의 적용 범위

작업지시자 결정에 따라, HWP 5 영문 글꼴 표가 비었을 때의 한컴바탕 선택은 별도 검토로 남긴다. 원본 분할 HWP 두 쪽은 이 차이로 Native 95.98%·96.21%, 브라우저 96.56%·96.62%다. 영문 슬롯을 명시한 대조본과 한컴 재저장본은 전부 100.00%다. 기여자 보고의 Native 둘째 쪽 96.22%도 동일 호스트의 원 PR로 다시 측정했다. 두 소스 출력은 96.21%이며 PNG가 같다. [영문 슬롯 대조](../assets/hyunmyungjanelee-20261125/maintainer/hwp5-latin-slot.tsv) · [같은 호스트의 원 PR 재검증](../assets/hyunmyungjanelee-20261125/maintainer/pr9352-same-host.tsv)

보호 문서 `nested`의 21개 글꼴 중 다섯 파일은 원본과 다른 SHA-256이다. 같은 프로그램 공급으로 두 소스·두 backend에서 각 17쪽이 모두 100.00%였다. 작업지시자 승인에 따라 이 다섯 쌍을 중첩 표 한 문서의 Native·WASM 허용 목록에 등록했다. 1,033pt 추가 용지 대조군의 첫 쪽은 한컴 2020·2024 프린터가 1,031pt에 인쇄해 Native 96.84%·WASM 97.12%다. 원본 용지 좌표로 환산한 진단값은 둘 다 100.00%다. 작업지시자는 두 렌더러의 원래 수치로 이 추가 인쇄 대조군의 한 쪽을 수용했다. [중첩 표와 보호값](../assets/hyunmyungjanelee-20261125/maintainer/protected/README.md) · [인쇄 좌표](../assets/hyunmyungjanelee-20261125/maintainer/paper-coordinates-native.tsv)

접수 원 head의 검사에서 #9352와 #9407은 CodeQL 요약이 NEUTRAL이었고, #9384는 성공·정상 생략이었다. 통합 PR에서는 네 언어의 CodeQL과 보안 검사, 정책 및 필수 CI가 성공했다.

각 기여 PR의 fork는 `kidsnote/rhwp`이다. 세 원 head가 통합 커밋의 조상임을 확인했다.

## 통합 결과

문서별로 승인된 글꼴 및 인쇄 조건으로 [통합 PR #9412](https://github.com/edwardkim/rhwp/pull/9412)를 `a5f01c69392dd4cf6919c8291023ddc84466e181`에서 검토했다. 필수 CI와 [셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9412#pullrequestreview-6063886947)를 마치고 [커밋 `5fa8cede504e`](https://github.com/edwardkim/rhwp/commit/5fa8cede504eca188d2e003931a50e9a8434d6da)으로 `devel`에 병합했다. [CI·보안·정책과 병합 후 실행](../assets/hyunmyungjanelee-20261125/maintainer/github-ci.tsv)

병합 후 nextest 실행 시간 갱신은 [success](https://github.com/edwardkim/rhwp/actions/runs/45687733465)로 끝났다. 이 PR에서 수집한 회귀 검사 실행 시간을 [정책 데이터 커밋](https://github.com/edwardkim/rhwp/commit/4ceac08511d258e6a533c48b2c6db72e75f4c328)에 반영했다.

운영 기록 작성 시점의 원 PR·이슈 상태다. 열린 이슈에는 최종 동기화 뒤 해당 이슈에서 검증 결과와 처리를 기록한다.

| 원 PR | 상태 | 연결 이슈 | 상태 |
|---|---|---|---|
| [#9352](https://github.com/edwardkim/rhwp/pull/9352) | MERGED | [#9349](https://github.com/edwardkim/rhwp/issues/9349) | OPEN |
| [#9384](https://github.com/edwardkim/rhwp/pull/9384) | MERGED | [#9383](https://github.com/edwardkim/rhwp/issues/9383) | OPEN |
| [#9407](https://github.com/edwardkim/rhwp/pull/9407) | MERGED | [#9406](https://github.com/edwardkim/rhwp/issues/9406) | OPEN |
