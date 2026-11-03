# [PR #8386](https://github.com/edwardkim/rhwp/pull/8386) 리뷰 — fix(hwpx): 개체 공통 attr bit 28 을 변경 추적 설정과 위치로 정한다 ([이슈 #8339](https://github.com/edwardkim/rhwp/issues/8339))

## 최종 판정

**승인** — HWPX 출처 개체의 bit 28 저장 계약을 확인했다.
[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)의 머지 SHA
`b8338ba38be288b6e9385f5372d184649323aa9f`에 원 PR의 적용 내용이 반영됐고
원 PR head가 검토한 head와 일치한다. 후속 제품 코드 `5aa6ee08dd`의 최소 글꼴
전체 Rust 회귀 12,387건 통과·45건 skipped 및 아래의 원시 OLE bit·저장 바이트
검사를 확인했다. 원 PR은 중복 병합하지 않고 후속 보정 PR의 최신 head CI·병합
확인 뒤 종료한다. 원 PR이 범위 밖으로 밝힌 양식과 HWP3·PDF 출처 변환은 이
결과로 해결했다고 주장하지 않는다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `26d70e72be` → `11f21bbc7e`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8339_hwpx_object_attr_bit28` 5건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

### 리베이스 차이 대조

원본 `26d70e72be`와 통합 `11f21bbc7e`의 range-diff에서 OLE 초기값의 이전 비트 설정이 새 base 문맥에서 제외된 차이를 확인했다. 변경 추적에 따른 bit 28 결정은 보존되며, 차이가 있는 OLE 경로는 아래에서 별도로 실행했다.

제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀에서 flags 56·0·16·48·8·24, 본문·바탕쪽, 수식과 원래 flags 0 문서를 검증한 해당 모듈 5건이 통과했다. `materialize_object_attr_bit28`은 도형 공통 속성에 접근해 OLE도 같은 규칙으로 처리하며, 렌더러는 이 개체 bit를 소비하지 않는다. 저장 속성 검증으로서 Visual Sweep 대상이 아니다.

리베이스로 달라진 OLE 경로는 별도로 실행했다. 기존 [OLE HWPX](../../../tests/fixtures/inline-control-move/chart-before.hwpx)의 본문 OLE를 HWP로 저장하고, 같은 입력에서 `trackchageConfig`만 56→0으로 바꾼 [합성 대조군](../assets/issue_8339/followup8438/ole-flags-zero.hwpx)도 저장했다. CFB의 `BodyText/Section0` 원시 CTRL_HEADER를 직접 읽은 결과 flags 56 [저장본](../assets/issue_8339/followup8438/ole-flags-56.hwp)은 `0x140a2211`로 bit 28이 켜졌고, flags 0 [저장본](../assets/issue_8339/followup8438/ole-flags-zero.hwp)은 `0x040a2211`로 꺼졌다. [원시 레코드 확인 스크립트](../assets/issue_8339/followup8438/read-ole-attr.py.txt)로 재현할 수 있다. 두 입력의 기대 방향은 원 PR에 기록된 한컴 2024 flags·문맥 규칙이며, 합성 대조군을 한컴이 저장한 원본으로 취급하지 않는다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8386](https://github.com/edwardkim/rhwp/pull/8386), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `26d70e72be952f00544cb50363ed0a8d860cd6f3`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `26d70e72be952f00544cb50363ed0a8d860cd6f3`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8339](https://github.com/edwardkim/rhwp/issues/8339).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWPX에서 HWP로 변환할 때 개체 공통 속성 bit 28을 변경 추적 설정과 개체 위치에 맞춰 저장한다.
번호 범주 bit와 변경 추적 bit가 독립적으로 보존되는지 확인한다.

## 체리픽과 보정

- 원본 `26d70e72be952f00544cb50363ed0a8d860cd6f3` → 통합 `2447ecc89181010c85f87de159771eac221b33f5`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8339_hwpx_object_attr_bit28.rs`](../../../tests/cases/issue_8339_hwpx_object_attr_bit28.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`math-001.hwp`](../../../samples/math-001.hwp) — SHA-256 `26b9e9a450e2191c529348f933ba674106394c9a9f1d009559b5af50cd57e826`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8339_hwpx_object_attr_bit28` 5건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 새 base OLE 저장 경로 재실행

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 실행 파일로 두 HWPX를 HWP로 다시
저장하고 `--verify`로 재파싱을 확인했다. 설정 56 저장본의 SHA-256은
`c1f22903b149a137fb9e85a8c6f383c6e601c326dc189762579fbf71c10a9a46`, 설정 0 저장본은
`e132a72ae5d275ccce356479c64ea3e28a66567f6cac7b979b3571ac2fcfb0e9`로 위 기존
저장본과 각각 바이트 단위로 같다. 원시 OLE 속성도 각각 `0x140a2211`, `0x040a2211`이다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.

2026-11-03 제품 코드 `d56930dda7`에서도 같은 두 HWPX를 `convert --verify`로
각각 HWP에 저장했다. 설정 56과 0의 출력 SHA-256은 위 기록과 각각 동일한
`c1f22903b149a137fb9e85a8c6f383c6e601c326dc189762579fbf71c10a9a46`,
`e132a72ae5d275ccce356479c64ea3e28a66567f6cac7b979b3571ac2fcfb0e9`다.
원시 OLE 속성도 각각 `0x140a2211`과 `0x040a2211`로 재확인했다.
이 저장 계약 확인은 최신 코드의 전체 최소 글꼴 회귀 실행 결과와 구분한다.
