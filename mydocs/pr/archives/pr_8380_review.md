# [PR #8380](https://github.com/edwardkim/rhwp/pull/8380) 리뷰 — fix(pdf-import): 본문을 가리지 않는 그림 때문에 문단을 위치 고정 낱말로 남기지 않는다 ([이슈 #8362](https://github.com/edwardkim/rhwp/issues/8362), [PR #8375](https://github.com/edwardkim/rhwp/pull/8375) 위)

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `a19fbc6620` → `8a4c85b945`, `7f9ae2bebc` → `74452590eb`, `f05ecea19b` → `14a70ca103`, `735304f879` → `91a633b2a7`, `dca4739bcf` → `96c2e0ade8`, `5e42900f52` → `b1cc92d9ec`, `c463bbc892` → `599123e88e`, `908093912d` → `d0ec03704b`, `0d0e9e88de` → `3a22c6e2a9`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8362_pdf_editable_hancom_rows` 2건, `issue_8362_pdf_editable_paint_order` 2건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

- 원 PR: [PR #8380](https://github.com/edwardkim/rhwp/pull/8380), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `0d0e9e88de6fa9bf66f17f25963e6b9cf7f28ba9`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `0d0e9e88de6fa9bf66f17f25963e6b9cf7f28ba9`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8362](https://github.com/edwardkim/rhwp/issues/8362).
- 선행 PR: [PR #8365](https://github.com/edwardkim/rhwp/pull/8365)·[PR #8375](https://github.com/edwardkim/rhwp/pull/8375).

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

본문을 가리지 않는 PDF 그림을 이유로 문단을 위치 고정 낱말로 남기지 않는다.
선행 [PR #8365](https://github.com/edwardkim/rhwp/pull/8365)·[PR #8375](https://github.com/edwardkim/rhwp/pull/8375)와 그림 겹침·paint 순서 및 뒤 본문을 함께 확인한다.

## 체리픽과 보정

- 원본 `a19fbc662055adb6ee864141efa48db6b022e310` → 통합 `a63b516027e9f0e396b1f31bac1cd4c173a3ae90`.
- 원본 `7f9ae2bebc3ff66648512e42899702320b2e0cac` → 통합 `686fcc33f0ff9484235b44844521c0bf10b439bb`.
- 원본 `f05ecea19b1490c91179d9738204b896c6fe5b86` → 통합 `3d909c41953c40a3f5941c5f2103d889b2236f31`.
- 원본 `735304f879d269f9c86b3283a17aa73f3cd981c0` → 통합 `d8f611ce4d3929342a82475afbbf6e9795fa7a1f`.
- 원본 `dca4739bcf59b4a96e0633d485a878ffcb4283c0` → 통합 `31e627bfb96860cd68a7dccc059a436e3e01142a`.
- 원본 `5e42900f52786cf6d3b075499d660fe02ca2f7e5` → 통합 `64f6faa89c91496b00e0f352e25300fa373e29f6`.
- 원본 `c463bbc89201ec2b8541cec9665f1a2cc3601815` → 통합 `251d271f7fa15d7a25df958ccf7fa065454e1dfb`.
- 원본 `908093912d5dec9eb4b01ae1d7c268666a0b02ae` → 통합 `0f8751e97a2ac50be7f0f367fe30ac9b1fdc846e`.
- 원본 `0d0e9e88de6fa9bf66f17f25963e6b9cf7f28ba9` → 통합 `0efd5e97f5dbdbec16cdec3718f0761244dbdf60`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8362_pdf_editable_hancom_rows.rs`](../../../tests/cases/issue_8362_pdf_editable_hancom_rows.rs): 이전 실행 2건 통과, 0건 실패, 0건 건너뜀.
- [`issue_8362_pdf_editable_paint_order.rs`](../../../tests/cases/issue_8362_pdf_editable_paint_order.rs): 이전 실행 2건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`editable-import-2024-hancom-rows.pdf`](../../../pdf/issue8362/editable-import-2024-hancom-rows.pdf) — SHA-256 `139db8fbff158f31dc5a3ee6c4a36cf256268185a3fa1a7d0a2e24f48ab0b57c`.
- [`editable-import-2024.pdf`](../../../pdf/issue8362/editable-import-2024.pdf) — SHA-256 `ecfebf930da0c63e0f389f46405ee8053f6adcb82353189c766be5f1c1bd01a1`.
- [`21_언어_기출_편집가능본-2010.pdf`](../../../samples/21_언어_기출_편집가능본-2010.pdf) — SHA-256 `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## 2026-11-02 일괄 대조 추가 증거

현재 통합 코드로 `samples/21_언어_기출_편집가능본-2010.pdf`를 편집 우선
가져오기 해 저장한 [HWP](../assets/issue_8381/followup8438/editable-saved.hwp)와
그 파일의 [한컴 2024 Print PDF](../../../pdf/issue8361/batch-editable-saved-2024.pdf)를
96dpi에서 같은 16쪽으로 대조했다. MCP 작업과 파일 해시는 [기준 기록](../assets/issue_8373/followup8438/print-sources.md)에 있다.
Native 최저 98.79%, fresh WASM 최저 98.72%이며 양쪽 모두 90% 미만은 없다.
[Native 전체 TSV](../assets/issue_8381/followup8438/native-all.tsv) ·
[WASM 전체 TSV](../assets/issue_8381/followup8438/wasm-all.tsv).
9·15쪽의 글상자와 흐름 본문, 13쪽 제목 줄바꿈을 직접 대조했다.
[9쪽 review](../assets/issue_8381/followup8438/native-review-009.png) · [9쪽 overlay](../assets/issue_8381/followup8438/native-overlay-009.png) · [13쪽 review](../assets/issue_8381/followup8438/native-review-013.png) · [13쪽 overlay](../assets/issue_8381/followup8438/native-overlay-013.png) · [15쪽 review](../assets/issue_8381/followup8438/native-review-015.png) · [15쪽 overlay](../assets/issue_8381/followup8438/native-overlay-015.png).

### 새 base의 저장본 확인

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `target/pr-review` Native
실행 파일(SHA-256 `34dfddd062947e91c33cb3c2b2c16a10f9903d2d1184fcdb646683e6a0dc2f1f`)로
같은 PDF를 다시 편집 우선 가져오기 했다. 결과 HWP는 16쪽이며 SHA-256
`edaf8172427572450503f3265fca95b1c1d2c9673c1e5b25528f5b55294a043f`로 위 기존
Print 입력과 바이트 단위로 일치한다. 새 base와 이전 검증 코드 사이의 렌더러·
파서·HWPX 저장기는 변경되지 않았고 후속 브랜치에도 제품 코드 차이가 없어
위 기존 TSV와 Print 이미지를 재사용한다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8362_pdf_editable_paint_order` 2건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.
