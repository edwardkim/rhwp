---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-12
---

# #7657 일괄 보정과 #7807 진단의 현재 증거

이 단계는 **부분 보정·미충족 기록**이다. 정상 회귀 fixture/golden이나 PR 수용 완료가 아니다.
초기18개 통합 범위와 사용자가 추가 승인한 #7807 전체 해결을 유지한다.
사용자의 후속 지시에 따라 검증된 HWPX 1쪽 OLE 제목의 관계형 검사만 정식 추가한다.
미달인 HWP3 1쪽과 나머지 쪽의 검사·기대값·baseline·허용치는 바꾸지 않는다.

## 구현과 실제 실행

- 셀 서식 변경은 기존 저장 조각 경계를 먼저 보존하고 첫 줄의 실제 앞 간격 차이만
  LineSeg 원점에 갱신한다. 직접 문단 모양 변경·셀 스타일 변경·글자 모양만 바뀐 경우도
  같은 셀 재배치 표시를 사용한다. `cell_format_origin.rs`와 formatting 호출이 생산 경로다.
  아래 반례는 **원점 갱신 후에도 측정 소비 경로 보정이 남음**을 입증한다.
- 미주 접두 번호는 ENDNOTE AutoNumber가 소유한 원시 UTF-16 슬롯의 글자 모양을 읽는다.
  지운 첫 구분 공백의 글자 모양도 따로 보존하며, 뒤 본문 글자 모양을 번호에 대신 적용하지 않는다.
- WMF OLE 미리보기는 원본 개체의 두 축을 채운다. OlePres·직접 CONTENTS WMF 경로의
  preserveAspectRatio를 `none`으로 바꿨으며 EMF/일반 그림 경로는 바꾸지 않았다.
  독립 Print의 1쪽 제목 높이가 원본 OLE 영역을 채우는 관측을 따른다.

v406 fmt/fmt-check/Native build는 exit0, fresh WASM wrapper는 exit0(1142.069초)이다.
실제 빌드 소스1112개 해시가 검증 동안 유지됐고 pkg/Studio public JS·WASM 해시도 같다.
브라우저 API 전체 검증·최종 lint·전체 회귀 통과를 뜻하지 않는다.
최초 실행의 untracked PDF guard 실패는 source mutation 전이며, 다음 실행의 executable/출력
디렉터리 이름 충돌은 진단 실행 실패다. `result-retry2.json`은 이미 성공한 build·원본6개 생성과
실패 기록을 보존하고 수정한 출력 디렉터리로 반례 생성만 다시 실행했다.

## 첫 셀 반례와 초기 추론의 정정

한컴 저장 대조군은 첫 LineSeg 원점0→1200 **외에 표 Common 높이1282→3082HWPUNIT도
바뀌었다.** cellSz 높이282와 안여백141이 같다는 사실만으로 전체 선언 높이가 같다고
판정했던 초기 분석은 부족했다. 한컴 대조군의100%가 renderer의 올바른 앞 간격 측정을
입증하지는 않는다.

v406 저장본의 첫 줄 원점은1200이 됐고 빈 TextLine 위도175.3px로 옮겨졌다.
그러나 셀 높이는25.1px, 뒤 문단은186.2px로 남아 내부 줄이 셀 아래를 넘는다.
독립 Print의 셀 높이는41.1px다. 첫 물리 앞 간격의 생산→줄 높이 합산→표 예약 높이→
셀 배치→뒤 문단 흐름을 모두 연결해야 하며 Common 높이를 강제로 늘리거나 clamp하지 않는다.

현재 `cell_first_para_stored_lead`는 table_layout의 배치/정렬/원점 경로에는 있으나
height_measurer의 세 줄 합산 경로에는 없다. 저장 첫 문단의 앞 간격을0으로 읽는
`cell_paragraph_spacing_before` 이후 물리 lead가 빠지는 소비 위치를 후속 보정 대상으로 추적한다.

후속 경로 대조에서 단순 합산 네 곳 외에도 실제 소비 지점을 확인했다.
`height_measurer.rs`의 단일 행·rowspan·줄 범위 측정과 `calc_para_lines_height`가 예약 높이를
생산한다. 반면 `cell_units_uncached`는 첫 저장 원점을 분리한 뒤 첫 유닛의 물리 lead를
이미 계상한다. `stored_cell_initial_frame_origin_hu`와 실제 Top 배치도 같은 저장 원점을
소비한다. `calc_visible_content_height_from_ranges_with_offset`의 부분 범위 합산과
Center/Bottom의 `align_lead` 선택은 그 뒤의 별도 소비 지점이므로 함께 검증해야 한다.

따라서 저장 프레임의 물리 lead와 배치 흐름에 추가할 앞 간격을 하나의 결과에서 구분하고,
예약·컷·가시 범위·최종 정렬이 각각 그 값을 한 번만 소비하는 보정을 준비한다.
저장 프레임이 없는 재조판의 선언 앞 간격, 둘째 문단, 앞 간격 복구 대조군과
이어받기 뒤의 첫 줄을 함께 확인한다. 이 문단은 코드 대조 결과이며 구현·검증 완료가 아니다.

| v406 실제 Native 반례 | 2px 실루엣 |
| --- | --- |
| unchanged-empty / second-of-two / restored-para-shape | 각각100% |
| visible-style | 44.90212% |
| first-of-two | 49.78778% |
| direct-format-preserved | 42.15131% |
| idempotent-style | 41.80704% |
| first-paragraph-spacing | 41.08527% |
| 원래 셀 스타일 삭제 HWP/HWPX | 각각41.80704% |

이 낮은 출력의 기대값을 정식 회귀로 고정하지 않는다.

## SO-SUEOP 1쪽과 #7807 전체 범위

원본 HWP3/HWPX는 각각46쪽이고 입력·독립 PDF 해시가 #7807의 원문과 같다.
HWP 기준 PDF의 파일명은2020이지만 Creator는Hwp2022다. 형식 연도만으로 배제하거나
다른 형식 PDF로 대체하지 않았다.

| 원본 1쪽 | Native | fresh WASM | 직접 판독 |
| --- | --- | --- | --- |
| SO-SUEOP.hwp | 84.44393% | 아직 미실행 | OLE 높이 회복. 학교명/교사명/학년용의 폭·시작 차이 남음 |
| SO-SUEOP.hwpx | 96.97415% | 96.97415% | Native/fresh RGB 동일. OLE 높이 회복. 본문 글자 미세 간격 차이 남음 |

[HWP Native review](so-sueop-hwp3-native-review_001.png)와
[HWPX fresh review](so-sueop-hwpx-fresh-review_001.png)를 실제 표시해 읽었고,
같은 좌표·배율의2배 확대도 확인했다. 점수 통과만으로 알려진 글자 차이를 면제하지 않는다.

[1쪽 제목의 진단 검사](so-sueop-p1-diagnostic.rs.txt)는 같은 소스를 실제 기존
release-test library와 v406 debug library에 각각 컴파일해 실행했다.
수정 전2FAIL/exit101, 수정 후2PASS/exit0이며 모두 컴파일은 성공했다.
원본 OLE 소유 영역과 독립 usvg 파서가 해석한 최종 벡터 투영 영역의 관계를 검사한다.
수정 전 높이 차이−32.69691%·위 원점 차이16.349211%로 의도한 결함에서 실패했다.
절대 실물 px 위치나 전체 SVG 해시를 고정하지 않았다.

후속 v413은 HWPX 1쪽만 검사하는
[`so_sueop_hwpx_first_page_ole_title_regression.rs`](../../../../../tests/cases/so_sueop_hwpx_first_page_ole_title_regression.rs)를
정식 경로에 먼저 추가했다. **최신 사용자 지시는 HWP/HWPX 양쪽 포함**이므로
HWP3 1쪽 보정·Native/fresh90% 이상·확대 판독 뒤 같은 관계 검사를 정식 추가해야 한다.
HWPX-only 상태는 요청의 완료가 아니다. 추가 범위는 1쪽 제목 OLE의 두 축·원점·단일 소유/출력 관계다.
같은 쪽의 변경된 제목 전체·앞뒤 글자를 Native/fresh/PDF 동일 배율로 직접 대조했다.
남은 제목 바깥 본문 글꼴 미세 차이와 HWP3/다른 쪽의 미달은 전체 시각 보류로 유지한다.
정식 HWPX-only 소스의 수정 전 FAIL/수정 후 PASS와 파생 integration suite의 실제 실행은
`hwpx-formal-p1-before-after-result-v413-retry1.json`에 최종 포맷 소스로
수정 전1FAIL/수정 후1PASS를 확인했다. 첫 집중 실행은 fmt 단계에서 멈춰 테스트0건이었고,
같은 의미의 포맷 수정 뒤 `original-pr7657-p1-focused-v413-retry1-retry1-result.json`으로
파생 suite의 실제 집중 검증을 진행한다.
이 1쪽 회귀의 통과로 #7807 전체 종료를 주장하지 않는다.

HWP3 Native 전체46쪽은 최저2쪽42.64803%, 미달34쪽, 누락0·46/46이다.
HWPX 전쪽 Native는 현재 동일 frozen binary의 v409 작업에서 실행 중이며,
양 형식 전체 fresh 검증과 확대 판독도 남아 있다. #7807 전체 해결로 보고하지 않는다.
사용자의 최신 지시에 따라 1쪽 회귀 생성 뒤에는 원래 체리픽18개PR의 검증을 우선하며,
진행 중인 v409 진단은 종료 결과를 보존한다. 전체 #7807 fresh sweep을 추가 시작하지 않는다.

### HWP3 원문·실제 글꼴 프로그램 추적

HWP3 원문 1쪽의3개 문단을 압축 해제한 원시 레코드에서 독립 읽었다.
각 문단은 header/줄 정보/글자 모양 플래그/Johab 내용/CR 끝이 유일하게 대응한다.
저장 pgy/sx/psx·공백 보정과 직접 PDF의 실제 텍스트 원점을 구분해 진단한다.
공식 [HWP3 공개 사양 표12](https://cdn.hancom.com/link/docs/한글문서파일형식3.0_HWPML_revision1.2.pdf)는
줄 정보 offset6의6바이트를 예약 영역으로 정의한다. 현재 parser가 pgy/sx/psx로 읽는 영역의
관측 대응을 공식적인 좌표 정의로 승격하지 않으며 추가 정상 원본·편집 대조가 필요하다.
1쪽은 공백 보정과 낱말 간격이0이고 char 장평95%·자간−1%·bold다.

HWP PDF의 제목 본문은 Type3 프로그램이고 HWPX PDF는 일부 같은 Type3와 실제 Human TTF를
혼용한다. HWPX PDF의 Human MyeongJo767개/Human Gothic502개 고유 글리프 윤곽선과
전진폭은 각각 실제 HMKMM.TTF/HMKMG.TTF의 프로그램과 전부 대응했다.
진단 프로필로 그 실제 TTF를 공급한 HWPX43쪽은85.73708%로 여전히 미달이다.
따라서 같은 파일명이나 입력의 HFT 선언만으로 프로그램·폭·paint가 같다고 판정하지 않는다.
실제 HFT 설치의 내부 이름도 읽어 HMKSM.HFT=휴먼명조, HMKMG.HFT=휴먼고딕,
HGPG.HFT/ENPG.HFT=필기임을 확인했다. 이 상용 글꼴의 바이트는 공개 자산에 포함하지 않는다.

## 입력과 독립 Print의 연결

API 생성의 빈 문서 반례와 실물 원본의 편집 저장본을 구분한다.
[counter 생성 소스](cell-counters.rs.txt)는 기존 편집 API로 앞/뒤 문단·직접 서식·복구·반복을
실행하며 수동 줄 좌표를 만들지 않는다. before/after 입력을 독립 Print와 각각 연결한다.
한컴2020 runtime11.0.0.9136·PrintMethod0·1-up·input_preprocess none이다.
동일 whole-input SHA일 때만 기존 Print를 재사용했다. Print 생성 성공과 시각 통과는 다르다.

| 단계 | 입력 | 입력 SHA256 | 독립 Print | PDF SHA256 | job | 현재 Native |
| --- | --- | --- | --- | --- | --- | --- |
| before | [counter-01-unchanged-empty.hwpx](before/counter-01-unchanged-empty.hwpx) | `abfe32b6753b3cd686ab161046918718d39297139abee0c0508635b2fe688b62` | [counter-01-unchanged-empty-abfe32b6753b-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-01-unchanged-empty-abfe32b6753b-2020.pdf) | `8d30808ac58ee634aa0b3a1653bf4fb108dacf98f830e3049a2b12e7093eb05a` | `d5a7f167-483a-48dc-ac54-13d55ef87c1e` | 이 행은 미측정 |
| before | [counter-02-visible-style.hwpx](before/counter-02-visible-style.hwpx) | `bdd5c5f7c8bdb3fb008ecbe590fd95c0ac3f305ce2a292d5ae8af2239cf323c7` | [counter-02-visible-style-bdd5c5f7c8bd-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-02-visible-style-bdd5c5f7c8bd-2020.pdf) | `99569b94ac32f73da92bcec75473c4f22cd7a97fdca9a7c9bc8871bfea7896a6` | `19cf28d0-ffcd-4975-b8f2-68bf560ccab6` | 이 행은 미측정 |
| before | [counter-03-first-of-two.hwpx](before/counter-03-first-of-two.hwpx) | `df5219f154849f61eadc08bed07a63080ce712dd93097044d7c7f61b7874738b` | [counter-03-first-of-two-df5219f15484-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-03-first-of-two-df5219f15484-2020.pdf) | `56476953dd287a29affdb6ac38bbce0bb92ebe36e57215e96c0ac9945793e880` | `a9283e8b-9bf4-42a1-b26c-d0ed17a73dd2` | 이 행은 미측정 |
| before | [counter-04-second-of-two.hwpx](before/counter-04-second-of-two.hwpx) | `0a8bef3556eabc95a05a7de885fc7fa50935714eccfd2834934f53d00661c80f` | [counter-04-second-of-two-0a8bef3556ea-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-04-second-of-two-0a8bef3556ea-2020.pdf) | `5a79e13492dd4427f73b3a1cf5fa6bbfad838547814dbebea1003ead5dd81d23` | `26281813-959a-4f15-8188-c7de01797964` | 이 행은 미측정 |
| before | [counter-05-direct-format-preserved.hwpx](before/counter-05-direct-format-preserved.hwpx) | `cda5dd968544f21473175460417d62335f9a501a7aaf6dbc4d824605c4ff8dc2` | [counter-05-direct-format-preserved-cda5dd968544-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-05-direct-format-preserved-cda5dd968544-2020.pdf) | `33a4b3c5378708305e370021dd0973f691a625a0f3d0544c811ebe0e8327acaf` | `4a245275-a52d-459a-8c54-bb64ce69b425` | 이 행은 미측정 |
| before | [counter-06-restored-para-shape.hwpx](before/counter-06-restored-para-shape.hwpx) | `3fd34172d6c9934e05951a5e52035e5d68a63f959c048ccae530ae726f9d10a9` | [counter-06-restored-para-shape-3fd34172d6c9-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-06-restored-para-shape-3fd34172d6c9-2020.pdf) | `0ecb759b0adee459b1717ae7d1e69f7978780214022e92dc8e6d5c1fa39996fa` | `8f7de6a9-927a-4020-a03f-4a66490bd111` | 이 행은 미측정 |
| before | [counter-07-idempotent-style.hwpx](before/counter-07-idempotent-style.hwpx) | `0e6ed5fb335d3bdfef8f214fc87a3b6e77813dd9b2640efbbf58c9fb7437e403` | [counter-07-idempotent-style-0e6ed5fb335d-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-07-idempotent-style-0e6ed5fb335d-2020.pdf) | `bf66db9a9a1ab05575299270072407bf9230959943bf376d9b6d33011bb60f34` | `bf709386-9fae-4b57-a552-b34f1d674d9f` | 이 행은 미측정 |
| before | [counter-08-first-para-spacing.hwpx](before/counter-08-first-para-spacing.hwpx) | `de4d27be2ff69eebad34b5958f7574c1f89abc7eacae4bab3dfb97e134d80c1d` | [counter-08-first-para-spacing-de4d27be2ff6-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-08-first-para-spacing-de4d27be2ff6-2020.pdf) | `b12a15ebf8b9238d8e2ca1bc88cf0cf592ea186dd5e26d6fa79fe5e246d865be` | `8b871190-6923-455d-8d8a-41afb3f5e420` | 이 행은 미측정 |
| after | [counter-01-unchanged-empty.hwpx](after/counter-01-unchanged-empty.hwpx) | `abfe32b6753b3cd686ab161046918718d39297139abee0c0508635b2fe688b62` | [counter-01-unchanged-empty-abfe32b6753b-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-01-unchanged-empty-abfe32b6753b-2020.pdf) | `8d30808ac58ee634aa0b3a1653bf4fb108dacf98f830e3049a2b12e7093eb05a` | `d5a7f167-483a-48dc-ac54-13d55ef87c1e` | 100.00000% |
| after | [counter-02-visible-style.hwpx](after/counter-02-visible-style.hwpx) | `8ebbea4fa2349165a036ab6a7d832d7b475ce001cccd4715458cd4ed8d4703e6` | [counter-02-visible-style-8ebbea4fa234-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-02-visible-style-8ebbea4fa234-2020.pdf) | `38a281f126f2fe383639d3b8f1c9932317bcf9d6153d9fffe9ce52c9eb61fb6c` | `aa9f6e21-22ad-4ee7-9c98-edf99429df6a` | 44.90212% |
| after | [counter-03-first-of-two.hwpx](after/counter-03-first-of-two.hwpx) | `c09b29828e2b0780faa8bb28c65b96e3c03bc73b2a5f29abb64540eec1b584d7` | [counter-03-first-of-two-c09b29828e2b-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-03-first-of-two-c09b29828e2b-2020.pdf) | `e7785732ad79741e5fafca8fb3e0e8537a16ab6ccf8cbffd4c18cbc5c7abdea3` | `a2ff41d0-5a0e-40ec-a5af-bdf1cdc2ebe5` | 49.78778% |
| after | [counter-04-second-of-two.hwpx](after/counter-04-second-of-two.hwpx) | `7291e9f076905a69e9ec7afd8b7da0247eadec011dac7c68e35ac83ca3928236` | [counter-04-second-of-two-7291e9f07690-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-04-second-of-two-7291e9f07690-2020.pdf) | `d68495d907bb76513ebee1570029774f6c54b1a62b230cefb19e6bb934d95f59` | `3fef6fb4-8768-4229-87a7-9b235b756808` | 100.00000% |
| after | [counter-05-direct-format-preserved.hwpx](after/counter-05-direct-format-preserved.hwpx) | `d4d84c1d50ed75e54150605d7e62b080481d646bf391f0b7fa4af9a9c8d8610e` | [counter-05-direct-format-preserved-d4d84c1d50ed-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-05-direct-format-preserved-d4d84c1d50ed-2020.pdf) | `fb2d5d93456f347973ef43838a10013506a981e12b02bd59ef2853b6877f1e5a` | `787193cc-c5e9-4156-a81a-36c05d9a94f3` | 42.15131% |
| after | [counter-06-restored-para-shape.hwpx](after/counter-06-restored-para-shape.hwpx) | `3fd34172d6c9934e05951a5e52035e5d68a63f959c048ccae530ae726f9d10a9` | [counter-06-restored-para-shape-3fd34172d6c9-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-cell-counters-v406/counter-06-restored-para-shape-3fd34172d6c9-2020.pdf) | `0ecb759b0adee459b1717ae7d1e69f7978780214022e92dc8e6d5c1fa39996fa` | `8f7de6a9-927a-4020-a03f-4a66490bd111` | 100.00000% |
| after | [counter-07-idempotent-style.hwpx](after/counter-07-idempotent-style.hwpx) | `c1e9bcef6a63c593061ee95b2e3fcb6f1e60bcea16c749dfc6c6bb9c45f5457c` | [counter-07-idempotent-style-c1e9bcef6a63-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-07-idempotent-style-c1e9bcef6a63-2020.pdf) | `026ec70b46623be06c6f8041248e6a5085df3063ff3e33a514557140ffe4bfc9` | `abcb00f1-dc5d-4156-a631-0a476bb8432d` | 41.80704% |
| after | [counter-08-first-para-spacing.hwpx](after/counter-08-first-para-spacing.hwpx) | `4ab2beea5fd61706834bf8f929347e0be4b9b4de3e117ecef59f7b62623ea379` | [counter-08-first-para-spacing-4ab2beea5fd6-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/counter-08-first-para-spacing-4ab2beea5fd6-2020.pdf) | `e131778b5e2b92d68717166b2d8e82a5500ed58d110b4996355a43ec2ebe8e51` | `8243beb5-549e-4584-8743-2f15882ae651` | 41.08527% |
| after | [original-143e-after-delete.hwp](../pr7657-cell-style-before-correction-v399/original-143e-after-delete.hwp) | `d4ce355e04edf942628a82d1722fad387f74afa70f855d025d470b9d295c90a6` | [original-143e-after-delete-d4ce355e04ed-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-143e-after-delete-d4ce355e04ed-2020.pdf) | `90a0b8fb03a4f2389c65cd5e03acb8c9b8ff9a4d3127200dd5dfb6c258c8c174` | `6ad90c0a-3908-4f40-8eea-ac8ebfa148a9` | 이 행은 미측정 |
| after | [original-cell-style-after-delete.hwp](after/original-cell-style-after-delete.hwp) | `e788c42c854c71b454563e554a0f95863bf5c56325d5bbb13e53920cb22d5fcc` | [original-cell-style-after-delete-e788c42c854c-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/original-cell-style-after-delete-e788c42c854c-2020.pdf) | `32344ee08712aac6d6e8e5cf6bd8999c9b2c835846fc0e445f943ba3be9c7bf5` | `d74088c2-3f8c-4566-afba-0d64d1cbc0fa` | 41.80704% |
| after | [original-cell-style-after-delete.hwpx](after/original-cell-style-after-delete.hwpx) | `a1805d6c940915094d5a97397159f29ea6f4310151d379136cc1e768353d9f00` | [original-cell-style-after-delete-a1805d6c9409-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-after-v406/original-cell-style-after-delete-a1805d6c9409-2020.pdf) | `121668ce8d33b5e9e3169844d883fa4fc36eaa2421c3492c4df8f92b7c23e05e` | `6c331e37-3520-4c5e-ba01-c3fd14fb727b` | 41.80704% |
| after | [original-ruby-style-after-delete.hwp](../pr7657-cell-style-before-correction-v399/original-ruby-style-after-delete.hwp) | `0b8e4afed62e11a71a67488e7ad8ad7e956a14d15078e26ba6d18c3e7778c95a` | [original-ruby-style-after-delete-0b8e4afed62e-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-ruby-style-after-delete-0b8e4afed62e-2020.pdf) | `5b59276665ad9c9db2450432cf7260be6c3a593ace9030c2c213ba5d3029c074` | `7eeb7394-5704-4776-8cd7-3cce4367cb28` | 이 행은 미측정 |
| after | [original-ruby-style-after-delete.hwpx](../pr7657-cell-style-before-correction-v399/original-ruby-style-after-delete.hwpx) | `1a2b454f56dc12c7fd3a4b2dc70b49f94d868fa3e29d932d8ca6733e793ab505` | [original-ruby-style-after-delete-1a2b454f56dc-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-ruby-style-after-delete-1a2b454f56dc-2020.pdf) | `91e2e7f2a121ed6ac3067013a9e971c8e8b56d29ce0c8b6b1402ec95768fdaff` | `60f5897b-bebe-4fdf-ae47-d60eb20b8171` | 이 행은 미측정 |
| after | [original-sosueop-after-delete.hwpx](../pr7657-cell-style-before-correction-v399/original-sosueop-after-delete.hwpx) | `c43961693982d75b1803c8c0c96b20f8ccd87ce02d65f7af8aa408b4a94cf85c` | [original-sosueop-after-delete-c43961693982-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-sosueop-after-delete-c43961693982-2020.pdf) | `4727e7a1c9aee21760b0e0b5548a27b8cea9b8a988d1736c368488e2178a5c25` | `3abc4929-9768-440e-aaaa-0c04c60e068f` | 이 행은 미측정 |

중간 JSON·전쪽 TSV·실행 로그·SVG는 ignored `output/pr-review/semanticist21-20261008/`에
보존한다. v406 `result-retry2.json`, `fresh-wasm-result-retry2.json`, `cell-boundaries-result.json`,
`print-after-result-retry2.json`, `regression-before-after-result.json`,
`hwpx-p1-qualification-result.json`, v409 `issue7807-native-whole-baseline-result.json`,
v410 `issue7807-font-program-proof-v410/result.json`, v411 직접 review,
v412 `issue7807-hwp3-p1-raw-line-proof-v412.json`을 구분한다.

전체 회귀 실패·기존 직접 보정 문서·초기18개 개별 review의 최종 수용은 여전히 미완료이며,
이번 단계에서 원격 게시/push/PR/승인/merge/이슈 종료는 하지 않았다.

## 실행 종료 확인 (2026-10-12)

v413 retry1 실제 nextest는 4개 실행·4개 통과, exit0이다. 원래 #7657 의미 검사3개와
HWPX 1쪽 OLE 영역 관계 검사1개를 포함한다. 최종 source hash guard는 변경 없음이다.
증적은 ignored `original-pr7657-batched-correction-v406/original-pr7657-p1-focused-v413-retry1-retry1-result.json`
및 `logs/original-pr7657-p1-focused-v413-retry1-focused-nextest.log`다. 최초 fmt 실패는0개 실행으로 유지한다.

v409 원본 전체 Native 비교도 종료됐다(exit1, source 변경 없음). HWP3는46/46쪽,
최저2쪽42.64803%, 미달34쪽이다. HWPX는46/46쪽, 최저43쪽85.59465%, 미달1쪽이다.
쪽수 불일치·누락은 없다. `issue7807-native-whole-baseline-result.json`의 전쪽 TSV를 보존한다.
전체 #7807 해결 및 PR 통과 근거로 사용하지 않는다.

HWP3 1쪽 전진폭 가설의 SVG 진단(v415)은97.44400%지만 생산 코드 반영·fresh WASM 검증 전이다.
HWP와 HWPX 양형식 회귀를 완료하려면 HWP 원본1쪽의 실제 보정과 두 출력 검증을 먼저 마친다.
