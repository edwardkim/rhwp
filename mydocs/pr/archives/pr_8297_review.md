# PR #8297 리뷰 — fix(parser): HWP5 개체 번호 범주는 bit 26-27 의 2비트 — bit 28 이 켜진 표·그림·수식의 캡션이 1로 고정되던 문제 (#8294)

## 최종 판정

**메인터너 검토 완료 — HWP3 변환본의 그림 캡션 번호 확인.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐고, 작업지시자 지시에 따라 `hwp3-sample-hwp5.hwp`만으로 판단한다. 이 판정은 다른 원본이나 이슈 전체의 해결을 뜻하지 않는다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `8724d9f60e` → `a36ab08494`, `1dddcb4020` → `169e04e61e`, `1953a4be92` → `bd72faf384`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8294_hwp5_numbering_category_bit28` 2건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 접수와 적용

### 통합 후 HWP3 변환본 캡션 대조

제품 코드 `d536455b4e`에서 원 PR의 HWP3 변환본을 독립 한컴 2020 Print와 전쪽 비교했다. 16/16쪽이며 Native 최저 97.33%, fresh WASM 최저 99.90%다. 원 PR이 지목한 그림 4·5는 각각 8·9쪽의 원 PDF와 같다. [Native TSV](../assets/issue_8294/followup8438/native-h3s5.tsv) · [WASM TSV](../assets/issue_8294/followup8438/wasm-h3s5.tsv) · 8쪽 [review](../assets/issue_8294/followup8438/native-p008-review.png)·[overlay](../assets/issue_8294/followup8438/native-p008-overlay.png) · 9쪽 [review](../assets/issue_8294/followup8438/wasm-p009-review.png)·[overlay](../assets/issue_8294/followup8438/wasm-p009-overlay.png). 5쪽 제목의 글꼴 차이는 [#8316 검토](pr_8316_review.md)에 구분했다.

- 원 PR: [#8297](https://github.com/edwardkim/rhwp/pull/8297), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `1953a4be92b1c18a42a723721c45c5275d8fd0a2`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `1953a4be92b1c18a42a723721c45c5275d8fd0a2`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8294](https://github.com/edwardkim/rhwp/issues/8294).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP5 공통 속성의 번호 범주는 bit 26~27이며 bit 28과 분리해 캡션 번호를 읽는다.
표·그림·수식과 bit 28의 조합 및 저장 바이트 보존을 확인한다.

## 체리픽과 보정

- 원본 `8724d9f60e540c912060a86ff12293c422608d5c` → 통합 `788d3b11ba627a2de76971c4c0e37cece25993a0`.
- 원본 `1dddcb40207e9ee5e97a8a3c6b9a2b93973fd9fb` → 통합 `090d4313c461667ce14b0317a36f8279bf7c8197`.
- 원본 `1953a4be92b1c18a42a723721c45c5275d8fd0a2` → 통합 `80e7080f40e27dc8af08110179d9e459bbc9d68c`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8294_hwp5_numbering_category_bit28.rs`](../../../tests/cases/issue_8294_hwp5_numbering_category_bit28.rs): 이전 실행 2건 통과, 0건 실패, 0건 건너뜀.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.
