---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 103 — 정책연구 p129·p131 본문 각주 연속 fragment

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `bf2e59c73`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 직접 비교: `/tmp/rhwp-stage103-policy-p129-p132/`

작업트리의 Stage 96·98 #4138 조사 변경과 `table_layout.rs` 진단 출력은 다른 진행
변경이다. 이 stage에서는 편집·stage·되돌리기하지 않는다.

## 재현과 PDF 판정

직접 비교 원장은 p129→p130에 72자, p131→p132에 96자의
`rhwp_earlier_than_reference`를 보고한다. 본문 paragraph owner를 대조하면 두 후보의
본문 경계는 이미 저장 `LINE_SEG` reset 및 PDF와 일치한다.

- `pi=1372`: rhwp p129 `lines=0..6`, p130 `lines=6..9`
- `pi=1382`: rhwp p131 `lines=0..2`, p132 `lines=2..3`

차이는 본문이 아니라 각주 본문이다. PDF는 긴 각주 176을 p129에서 시작해 p130의
bottom footnote lane으로 이어 그린다. 반면 p131→p132는 각주 180의 분할
continuation이 아니다. PDF와 현재 rhwp 모두 p131에는 각주 179만 두고, p132에 각주
180 전체와 각주 181을 배치한다.

- p129: 각주 176의 `…일상생`까지 소유
- p130: 번호·separator를 반복하지 않고 `활이나 직업적 활동…`부터 이어진 뒤 각주
  177·178 배치
- p131: 각주 179만 소유하고 각주 180 fragment는 소유하지 않음
- p132: 각주 180 전체와 각주 181을 소유

## 수정 계약

native HWP5 본문 각주가 다음 조건을 모두 만족할 때 저장 각주 `LINE_SEG` reset을
물리 page fragment 경계로 사용한다.

1. 각주 문단별 composer 줄 수와 저장 `LINE_SEG` 수가 일대일이다.
2. synthetic 줄이 아닌 연속 두 줄 사이에 `previous.vpos > 0 && next.vpos == 0`인
   내부 reset이 정확히 하나다.
3. marker 본문 문단도 저장 reset에 의해 두 physical page로 나뉘었고, 기존 inline
   control owner 라우터가 marker page를 찾는다.
4. prefix는 marker page에 separator·번호와 함께 소급 등록한다.
5. suffix는 다음 page에 separator·번호 없이 먼저 등록하고, 뒤 각주가 있으면 그
   페이지의 separator는 한 번만 예약한다.

저장 줄 수가 composer와 다르거나 reset이 없거나 여러 개인 각주는 기존 원자 배치를
유지한다. 표 셀 각주에서 이미 쓰는 동일한 보수적 reset 판정을 공통화하고, 일반
capacity 추정으로 임의 분할하지 않는다.

source 각주 줄을 직접 감사한 결과는 다음과 같다.

- 각주 176: 4줄, `(textpos, vpos) = (0,0), (71,1172), (134,0), (197,1172)`.
  내부 reset line 2를 기준으로 p129 `0..2`, p130 `2..4`가 정답이다.
- 각주 180: 2줄, `(0,0), (72,1172)`로 내부 reset은 없다. marker는 `pi=1382`
  line 1, 다음 body line 2가 `vpos=0`이다. marker 줄 하단은 body 높이의 약 93%
  지점이고 p131에는 이미 각주 179가 있어, 180 전체를 p132가 소유한다.
- 대조군 p74의 각주 100도 기존 각주 뒤 marker 직후 reset 형상이지만 marker 줄 하단은
  body 높이의 약 81% 지점이라 같은 쪽에 들어간다. 전체 이동은 native HWP5 단일단,
  기존 각주, marker 직후 reset, body 하단 10% 구간을 모두 만족할 때로 한정한다.

## 진행 상태

- [x] p129~p132 PDF/rhwp 직접 판정
- [x] 본문 owner 정상 및 각주 176·180 fragment 결함 확정
- [x] 각주 내부 stored reset 및 composed line 대응 확정
- [x] native HWP5 본문 각주 reset fragment 구현
- [x] p129~p132 실물 회귀 추가 및 focused gate 29/29 통과
- [x] 직접 비교 원장·review PNG 재생성
- [x] stage 커밋
- [ ] 전체 release-test·clippy·Skia는 모든 잔여 stage 뒤 최종 PR gate에서 수행

## 결과

- 문서 페이지 수: 한컴 PDF/rhwp render tree 모두 215쪽
- 직접 비교: `/tmp/rhwp-stage103-policy-p129-p132-after/`
- p129→p130 및 p131→p132 page-boundary/text-owner/sequence 후보: 0건
- p129~p132 `body_footnote_lines`: 모두 0건
- pixel diff:
  - p129: 17.38% → 16.77%
  - p130: 9.96% → 10.16%
  - p131: 19.34% → 18.63%
  - p132: 15.10% → 15.50%

픽셀 수치는 글꼴·자간 차이를 포함해 p130·p132가 소폭 상승했지만, 비교 PNG에서 실제
page owner는 PDF와 일치한다. p129 각주 176은 reset 전 두 줄에서 끝나고 p130은 번호와
separator를 반복하지 않은 tail로 시작한다. p131은 각주 179만, p132는 각주 180·181을
소유한다. 자동 owner 원장과 body↔footnote 충돌 원장이 모두 비었다.
