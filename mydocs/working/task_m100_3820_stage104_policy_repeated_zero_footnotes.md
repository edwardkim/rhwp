---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 104 — 정책연구 반복-zero 각주 연속 쪽

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `9c3e33b84`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 수정 전 직접 비교: `/tmp/rhwp-stage104-policy-p176-p179/`
- 수정 후 직접 비교: `/tmp/rhwp-stage104-policy-p176-p179-after2/`

작업트리의 Stage 96·98 #4138 조사 변경은 다른 진행 변경이다. 이 stage에서는
편집·stage·되돌리기하지 않는다.

## 재현과 PDF 판정

현재 원장은 p176→p177에 75자, p178→p179에 94자의
`rhwp_earlier_than_reference`를 기록한다. PDF 직접 비교로 두 후보 모두 각주 continuation
owner 결함으로 확정했다.

- p176: PDF는 표 셀 각주 234의 첫 줄을 p176에 두고, `severely steatotic donor
  livers…` tail을 p177의 각주 lane에서 각주 235보다 먼저 이어 그린다. rhwp는 각주
  234 전체를 p176에 둔다.
- p178: PDF는 각주 240의 첫 줄만 p178에 두고, URL을 포함한 tail을 p179에서 각주
  241·242보다 먼저 이어 그린다. rhwp는 각주 240 전체를 p178에 두어 본문 두 줄과
  FootnoteArea가 겹친다.

source 각주 `LINE_SEG`는 다음 물리 경계를 직접 기록한다.

```text
note 234 (table cell): (textpos,vpos) = (0,0), (107,0)
note 240 (body):       (0,0), (67,0), (93,1172)
```

일반 같은 쪽 각주는 `0,1172,…`로 증가한다. 첫 두 줄이 모두 `vpos=0`인 경우는 첫 줄
뒤 새 physical page에서 다시 시작한 신호다. 전체 fixture에서 같은 반복-zero 형상은 기존
p30 각주 30, 이번 각주 234·240 세 건뿐이다. 각주 30은 기존 두 줄 특수 회귀가 이미
PDF의 p30→p31 분할을 고정한다.

## 수정 계약

1. native HWP5 각주에서 composer 줄과 저장 `LINE_SEG`가 일대일이어야 한다.
2. 동일 각주 문단의 line 0과 line 1이 모두 non-synthetic `vpos=0`이면 line 1을 명시적
   다음 page fragment 시작으로 인정한다.
3. 첫 fragment만 separator와 번호를 그리고 tail은 반복하지 않는다.
4. 표 셀 각주가 terminal RowBreak fragment에 있더라도 명시적 반복-zero이면 prefix를
   현재 page에 등록한 뒤 새 page를 열어 suffix를 먼저 등록한다.
5. 일반 body 각주도 marker가 current page에 있고 반복-zero이면 같은 방식으로 새 page에
   suffix를 등록한다. 저장 reset이 없거나 줄 수가 다르면 기존 원자 배치를 유지한다.

## 진행 상태

- [x] p176~p179 PDF/rhwp 직접 판정
- [x] 각주 234·240 source line owner 확정
- [x] 공통 repeated-zero fragment 신호 구현
- [x] 표 셀 terminal fragment 및 body current-page 경로 구현
- [x] p176~p179 실물 회귀와 기존 각주 회귀 수행
- [x] 직접 비교 원장·review PNG 재생성
- [x] stage 커밋
- [ ] 전체 release-test·clippy·Skia는 모든 잔여 stage 뒤 최종 PR gate에서 수행

## 결과

- 문서 페이지 수: 한컴 PDF, rhwp SVG, rhwp render tree 모두 215쪽
- 정책 fixture 회귀: `issue_3738_rowbreak_table_footnote_fragment` 30/30 통과
- p176→p177 및 p178→p179의 text owner-shift/sequence/page-boundary 후보: 0건
- p176~p179 `body_footnote_lines`: 모두 0건
- p176 각주 234:
  - p176은 번호와 `…using moderately and`까지 소유
  - p177은 번호를 반복하지 않고 `severely steatotic donor livers…`부터 이어진 뒤
    각주 235를 배치
- p178 각주 240:
  - p178은 번호와 `…이식대상자도`까지 소유
  - p179은 번호를 반복하지 않고 `HTLV-1 양성인 경우에는…` 및 URL을 이어 그린 뒤
    각주 241·242를 배치
- pixel diff:
  - p176: 16.27% → 15.77%
  - p177: 10.81% → 10.89%
  - p178: 14.05% → 13.27%
  - p179: 11.11% → 11.29%

p177·p179의 전체 pixel 수치는 글꼴·자간 차이를 포함해 소폭 증가했지만, 비교 PNG와
자동 원장에서 각주 continuation의 물리 page owner, 번호·separator 비반복, 본문↔각주
비중첩이 한컴 PDF와 일치한다.
