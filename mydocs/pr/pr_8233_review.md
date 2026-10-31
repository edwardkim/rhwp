# PR #8233 리뷰 — 그룹 자식 도형 CTRL_DATA 저장 보존

## 최종 판정

**메인터너 보정 후 수용 가능 — #8225의 그룹 자식 CTRL_DATA 보존 범위의 로컬 검증 완료.** 제품 기능은 원 기여를 유지하고 현행 회귀 증거 주석을 보완했다. 기존 최상위 중복 표현 차이까지 해결했다고 주장하지 않으며 원격 CI·push·merge는 아직 실행하지 않았다.

## 접수 정보

- 작성자: `chrisryugj`, CONTRIBUTOR, Open, non-Draft. Reviewer: `edwardkim`.
- 관련 이슈: #8225, 참조 범위. 원 head: `d9c8a71b642fad1ee8221f4036817a23dfe15f10`.
- 비교 기준: `upstream/devel` `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.
- 검토 브랜치: `review/integration-20261031-chrisryugj-storage`.
- #8224 다음 충돌 없이 통합한 후보: `86f4879934d1fc903b8497d011fdd7db9dede38f`.
- 이번 기여자의 원격 push·새 PR·댓글·머지는 아직 승인받지 않았다.

## 변경과 검토 범위

`SHAPE_COMPONENT` 자식의 `CTRL_DATA`를 `ShapeComponentAttr.ctrl_data`로 읽고, 저장할 때 해당 도형의 하위 레벨에 쓴다. 최상위 컨트롤 수집은 자식 데이터까지 잘못 가져오지 않도록 깊이를 제한한다. 저장 위치와 payload 순서·바이트를 대조하며 전체 레코드 수만으로 소유 관계를 판정하지 않는다.

이 필드는 serde에서 제외되는 HWP 원본 보존 데이터이며 레이아웃·paint의 소비 지점은 발견되지 않았다. 조판 자체를 수정하는 PR과 구분하고 실제 편집 저장을 통해 도형과 본문 보존을 확인한다. 동일 통합 후보의 #8224는 별도의 시각 검증 대상이다.

## 검증 입력과 진행 기록

- `samples/basic/sungeo.hwp`: 그룹 자식 CTRL_DATA payload 24개 보존 대상.
- `samples/issue6542/156678235_mid_para_vpos_rewind.hwp`: 자식 데이터가 최상위에 중복 저장되지 않아야 하는 대조군.
- `samples/hwp-multi-001.hwp`: 여러 구역 보존 대조군.
- 원 PR의 3개 저장 계약 검사를 고정 devel과 누적 후보에서 순차 실행했다.
- base WASM에서 실제 글자 삽입·삭제 → HWP 저장 → 재열기를 실행했다. 텍스트는 보존되며 쪽수는 각각 94·7·9쪽이었다. 이 결과는 CTRL_DATA 보존 판정과 구분한다.
- 기록과 진단 산출물: `output/pr-review/chrisryugj-20261031/`.

원 PR은 `sungeo.hwp`의 별도 최상위 CTRL_DATA 중복 표현까지 모두 복구한다고 주장하지 않는다. 남은 데이터 차이는 자식 보존과 구분해 확인하고 #8225 전체 해결로 과장하지 않는다. 고정 devel에서 3개 검사 실패, 실제 재컴파일한 후보에서 3개 통과를 확인했다. base WASM의 편집 저장본에서도 자식 payload 수가 24→0, 1→0, 1→0으로 줄었다. 원 PR CI와 별도로 아래 실제 후보 검증을 수행했다.

## 사양 근거

HWP 5.0 revision 1.3 §4.3.8·표 66은 CTRL_DATA를 컨트롤의 필드 이름·하이퍼링크 등을 담는 Parameter Set으로 정의한다. 실제 원본의 레코드 level과 부모 SHAPE_COMPONENT를 함께 추적한다. 이 불투명 payload를 실행하거나 조판 속성으로 재해석하지 않고 원 소유 도형 아래에 보존한다.

## 실제 WASM 편집 저장 결과

새 최적화 WASM에서 글자 삽입·삭제 → 저장 → 재열기를 실행했다.
그룹 자식 payload 24개·1개·1개는 원본과 순서·바이트가 모두 같고 텍스트도 보존됐다.
동일한 기본 글꼴 조건에서 세 문서의 94·7·9쪽 렌더 트리는 base와 후보가 동일했다.
이 검사는 데이터 저장 계약을 확인하며 해당 문서들의 한컴 시각 일치율로 승격하지 않는다.

`sungeo.hwp`의 전체 CTRL_DATA 60→48 차이도 별도로 조사했다. 원본의 고유 payload 25종은 모두
남고, 동일 payload 한 종류의 중복 수만 24→12로 감소한다. 자식 보존과 기존 최상위 중복 표현의
차이를 구분한다. [감사 결과와 정식 증거](assets/chrisryugj_20261031/README.md)를 연결했다.
lint·공통 전체 회귀·새 sample 보안 검사까지 아래와 같이 통과했다.

## 최종 공통 검증과 원격 처리 조건

- 전체 release-test: **12,207개 통과 / 실패 0 / 기존 skip 45**, 496.765초. 탭 6개·그룹 3개 포함.
- Native Skia: lib 4,040개, 그림 placeholder 2개, direct PDF 22개 통과. lib 기존 ignored 12개.
- fmt, Native/WASM Clippy, workspace build 및 all-target Clippy, suite manifest·unit-tier 모두 통과.
- 새 편집 HWP 4개를 명시한 보안·injection 검사 20개 통과. 전체 실행에서 빈 입력으로 반환된 결과와 구분한다.
- 독립 시각 증거·변경 테스트 3개 rustfmt·문서 링크·diff whitespace 확인 완료.
- [정식 게이트와 로그](assets/chrisryugj_20261031/README.md#최종-로컬-게이트).

제품 보정 `f85410c616`, 테스트·기록 시작점 `b8c0ef2e4b`와 이 문서의 최종 커밋을 구분한다.
검증 worktree와 기본 작업공간의 제품·변경 테스트 파일은 SHA-256으로 동일하다. 실제 사용한 기존
입력은 Git blob과 대조했고 새 편집 입력·PDF도 이 검토 기록과 같은 커밋에 포함한다.

2026-10-31 확인한 devel은 `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`이며 두 원 head는 변경되지 않았다.
본인 계정의 열린 PR은 0개였다. 원격 처리 승인을 받으면 기여 커밋을 조상으로 유지한 통합 branch를
push하고 Open PR의 정확한 head CI·self-review·최신 base와의 충돌 여부를 확인한다.
최종 merge SHA·CI·검토 증거를 원 PR별 존댓말 댓글로 안내하며 시각 판정에 사용한 #8224에는
SHA 고정 비교 이미지와 Print 링크를 포함한다. 아직 원격 push·PR·댓글·merge를 수행하지 않았다.
