# PR #7011 검토

## 판정: 승인

메인터너 보정과 재검증으로 본문 하단 기준선 실패를 해소했다. 1쪽 배너 이후 배치는 유지된다. 이 판정은 검토 대상 기능과 이번 보정의 승인이지, 아직 수행하지 않은 통합 PR의 원격 CI·병합 승인이 아니다.

## 대상과 검증 버전

- 원 PR: [#7011](https://github.com/edwardkim/rhwp/pull/7011), 본문 해결 대상 [#6928](https://github.com/edwardkim/rhwp/issues/6928).
- 작성자 `planet6897`, 검토자 `jangster77`, 검토일 2026-09-11.
- 원 head: `a0fa74b55a038ce316695234d89649e00128b5c5`, 로컬 체리픽 `5f554e802`.
- 검증 보정 커밋: `967b8d881d437d1254110b46e903a11edff29064`. 이후 최신 devel 병합 `2e6567618a2dbaa5c79b2dc86396ce84d38867e7`은 #6999 리뷰 문서만 추가했으며 검증한 제품·테스트 소스 변화는 없다.
- `src/diagnostics/layout_anomaly.rs` SHA-256: `5222cd7174ddc91b92924e844bf5a0bd7261f547b1ca1e913fd55a5855acafdd`.
- `tests/cases/issue_6928_empty_carrier_overflow.rs` SHA-256: `0edf14b7cdf4cca8f062703942d06015c8c1b8fc120fb62eed5a7cde78519641`.
- 검증 바이너리 `target/pr-review/release-test/rhwp` SHA-256: `da428ce2f9b3d98a3e245042b2cd4037d45329d9f9b5853829e489613c733988`.
- [통합 적용·초기 검증 및 보정 후 기록](pr_6996_review_impl.md).

## 원 PR 코드와 보류 원인

원 PR은 `src/renderer/layout/paragraph_layout.rs`에서 글자처럼 취급되는 표·도형 줄의 저장 진행량을 줄 높이와 함께 판단한다. 약 107.1px 높이의 개체 뒤에 약 2.7px 저장 진행량을 적용하여 후속 문단을 배너에 겹치게 하던 경로를 제한한다. 0.5px 수치 공차가 있으므로 모든 경우 줄 높이 미만 진행을 절대 금지한다고 설명하지 않는다.

초기 전체 회귀는 새 실물 sample의 baseline 없는 본문 하단 신호 1건 때문에 실패했다. JSON `page: 4`는 0부터 시작하는 번호이므로 **실제 대상은 5쪽**이다. 이전 문서가 4쪽으로 설명한 것은 검토자의 해석 오류이며 정정한다. 최초 `pr7011-p004.png`는 실패 대상 증거가 아니므로 최종 코멘트 자료에서 제외했다.

5쪽 `Page/Body/Column0/TextLine8`은 문단 69의 빈 줄이다. 유일한 TextRun은 `text=""`, 표시 텍스트도 빈 문자열, border_fill_id=0, 음영 없음 `0xFFFFFFFF`, 탭 리더 및 자식 없음이었다. 커서·줄 높이 상자의 y=1013.3867px, h=20px가 본문 하단 1028.04px를 5.3467px 초과한 것을 실제 출력 내용의 초과로 세던 **진단 오탐**이다. 이 신호만으로 실제 글자가 잘리거나 원 PR이 새 배치 회귀를 만들었다고 판단해서는 안 된다.

## 메인터너 보정

`layout_anomaly.rs`의 본문 overflow 검사에서 내용 없는 운반용 TextLine만 제외한다. 자식이 하나 이상이고 모두 원문·표시 문자열이 빈 TextRun이며, 테두리·실제 음영·탭 리더·하위 개체가 없는 경우로 좁혔다. 음영 여부는 렌더러와 동일한 `model::color::char_shade`로 판정한다.

- 실제 글자, 공백, 탭, 표시 문자열이 있는 줄은 계속 감지한다.
- 음영·테두리·개체 자식·TextRun 아래 개체 또는 자식 없는 불명확한 줄도 계속 감지한다.
- off-canvas, overlap, 본문 경계·공차, 레이아웃·저장 진행량은 이번 보정으로 바꾸지 않는다.
- 기준선 상향, sample 제외, 테스트 무시는 하지 않았다.
- 최초 보정은 흰색 sentinel만 고려해 실물에서 실패했다. 실제 `0xFFFFFFFF`와 미지정 값 0도 공통 판정으로 처리하도록 수정한 뒤 재검증했다.

## 테스트 결과

검토 전용 target과 테스트 스레드 8개를 사용했다. 전체 실행에는 이름을 바꾼 #7011 원본과 #6996 sample을 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 포함했다.

| 항목 | 최종 결과 |
| --- | --- |
| 집중 검증 | 114개 통과, 실패 0, 실행 30.738초, exit 0 |
| 기본 기능 전체 회귀 | 9,468개 통과, 실패 0, 기존 skip 46개, 실행 368.717초, exit 0 |
| 새 회귀 테스트 | 4개 통과: 빈 줄/음영 sentinel, 실제 문자열·공백·표시 문자열, 장식·개체 음성 대조, 실물 전체 페이지 |
| 본문 하단 기준선 | 16개 partition 모두 통과, 기준선 변경 없음 |
| 보안 코퍼스 | 새 sample 포함 검사 통과 |
| fmt·공백 검사 | 통과 |
| generated suite | 최종 재생성 및 check 통과: 1,257 sources / 5,332 static attrs / 48 integration targets |

실물 검사는 5쪽 유지, 전 페이지 본문 하단 초과 0건, 5쪽 표의 우측 초과 신호 유지까지 검증한다. 집중 실행에서 layout-anomaly CLI 계약 및 기존 #6996·#6999·#7007 관련 계약도 통과했다. 최초 실패한 2개 항목을 숨기지 않고 최종 재실행으로 해소했다.

최종 테스트 소스 길이 변경에 따른 generated harness 배정 drift는 전체 회귀 종료 후 `--prepare`로 정리했다. 제품·테스트 소스는 바꾸지 않고 suite 배정만 재생성했으며 전체 9,468개 실행 결과와 generator 정합성 검사를 구분한다.

## 원본과 기준 PDF

원본 수집 파일 `148769979_274 반도체포럼 개최(이귀남 최종).hwp`를 내용 변경 없이 `samples/issue6928/148769979_274 반도체포럼 개최(이귀남 최종).hwp`에 보관했다. 원본 SHA-256: `111eaf0265a19735feb904cea544235a2046bf891d9b16b886f24cb56798457f`.

한컴 2010 `8.5.8.1382` 저장 정보에 따라 기존 비동기 MCP `engine 2020` 작업 `5c3e5a4a-2482-4d91-a8fe-8d1957306df5`로 생성한 PDF를 재사용했다. 이름만 원본과 맞췄으며 다시 변환하지 않았다.

- 기준: `pdf/148769979_274 반도체포럼 개최(이귀남 최종).pdf`.
- Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.6, 5쪽, 247,919바이트.
- SHA-1: `b3ec34e42b1c11a07c0a85783df700baf2fcff95`.
- SHA-256: `24b1af0f6987709cf068ce028e5db00791b58db572895fc03b802d1da9d594f4`.
- 원 PR의 2024 기준 설명과 이번 engine 2020 결과는 구분한다.

## 보정 후 시각 대조

최종 바이너리로 `visual_sweep.py`를 실행해 1·5쪽을 새로 생성하고 직접 열어 대조했다. 왼쪽 rhwp, 가운데 PDF, 오른쪽 overlay다. Studio 직접 캡처가 아니다.

| 쪽 | 픽셀 일치율 | 내용 잉크 일치율 | 증적 |
| --- | --- | --- | --- |
| 1 | 88.343% | 30.052% | [배너·표·제목 흐름](../assets/pr_6996_7011_planet6897_20260911/pr7011-p001.png) |
| 5 | 89.872% | 41.463% | [실제 빈 문단 및 하단 대조 쪽](../assets/pr_6996_7011_planet6897_20260911/pr7011-p005.png) |

rhwp/PDF 모두 5쪽이며, 1쪽 배너 뒤 표·제목 분리가 유지된다. 5쪽에는 표 열 폭·행 높이·글꼴 차이가 남으나 이는 이번 빈 줄 오탐 해소와 별개다. `Table7`의 우측 초과 3.7733px는 여전히 보고되고 `overBottom`은 0이다. 전체 overflowCount=0이나 문서 전체 완전 일치를 주장하지 않는다. 잉크 일치율은 높을수록 유사한 보조값이며 사람 판정 정확도가 아니다.

## 향후 코멘트 계획

실제 통합 PR 및 devel CI 확인 후 merge SHA, 원 PR 출처, 집중 114개·전체 9,468개 통과, 빈 문단 진단 오탐 보정과 잔여 우측 경계를 명시한다. p1·p5 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6996_7011_planet6897_20260911/<파일명>` 형식으로 본문에 직접 표시하고 `<merge-commit-sha>`를 실제 값으로 치환한다. 같은 SHA의 `mydocs/manual/pr_review/visual_fixture_evidence.md` 링크와 수치의 한계도 포함한다.

원 PR 본문의 Closes 의도와 실제 API 종료 여부는 따로 확인한다. 추가 Native Skia lib 4,112개(기존 ignore 13개)·PNG 2개·직접 PDF 4개, Clippy 3종, workspace 빌드, Docker 없는 WASM 빌드까지 실제 통과했다. [공통 기록](pr_6996_review_impl.md)에 명령과 결과를 남겼으며 사용자 승인에 따라 통합 PR을 제출한다. 댓글·close·원격 merge는 아직 수행하지 않았다.
