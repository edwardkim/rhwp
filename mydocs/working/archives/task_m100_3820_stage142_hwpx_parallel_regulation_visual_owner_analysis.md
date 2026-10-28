# Stage 142: HWPX 병렬 규정 표 visual owner 분석

## 목적

HWPX `2025 행정업무운영 편람(최종).hwpx`의 103×2 병렬 규정 표를 PDF p311~p367과
대조해, 단순 383쪽/57 fragment 수가 아니라 각 physical page가 소유하는 행과 실제
글자폭·행높이를 맞출 근거를 만든다.

## 확정된 입력과 제약

- 기준 PDF는 `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`이며 383쪽이다.
- HWPX와 native HWP도 현재 383쪽이지만, PDF만 최종 visual oracle이다.
- Stage 141이 복원한 p310 blank와 p374→p375 전환은 후퇴시키지 않는다.
- 새 릴리스 준비 중이므로 merge, push, PR 생성, 원격 변경은 금지한다.

## 현재 차이

PDF 말미 계약은 다음과 같다.

- p364: 제63조의4·제63조의5, 제64조, 제5장 및 제65조 시작
- p365: 제65조 tail, 제66조~제68조
- p366: 제69조~제70조와 시행규칙 제47조·제48조 시작
- p367: 시행규칙 부칙과 양쪽 부칙 표

HWPX의 전역 160px reserve는 표를 57 fragment로 만들지만, r5/r12에서 조기 분할해
후반 owner를 밀어낸다. 특히 HWPX p366은 PDF p366보다 이른 r94(제66조)부터 시작해
PDF가 요구하는 r97(제69조)과 다르다. reserve 56px은 초반 owner는 가깝지만 표가
53 fragment에서 끝난다.

따라서 전역 reserve 숫자를 조절해 쪽수만 맞추는 방식은 금지한다.

## 1차 분석 결과: 첫 visual divergence

최종 383쪽 HWPX render-tree와 PDF를 p311~p318에서 대조했다.

| 물리쪽 | HWPX 규정 표 행 | PDF 왼쪽 열의 새 조항 |
| --- | --- | --- |
| p311 | r0, r1~r5 | 제1조~제3조 |
| p312 | r0, r5 | 없음 (제3조 continuation) |
| p313 | r0, r5 | 없음 (제3조 continuation) |
| p314 | r0, r5 | 제4조, 제5조 |
| p315 | r0, r5~r8 | 제6조, 제7조 |

PDF p314와 HWPX p314 SVG를 직접 비교하면, PDF는 제2장 및 제4조·제5조로
진행하지만 HWPX는 제3조 tail을 한 쪽 더 그린다. 따라서 최초 owner divergence는
p314이며, 원인은 160px 전역 reserve가 r5를 추가 fragment로 조기 분할한 것이다.

이 결과를 근거로 r5에는 일반 56px 경계를 유지하고, PDF 후반에서 실제로 긴 r71에만
필요한 fragment를 보충하는 row-level 계약을 구현한다.

## 구현

`src/renderer/typeset.rs`의 HWPX stored-layout 103×2, 206-cell `RowBreak` 표에만
다음 row-level reserve를 적용했다.

- r5: 56px. PDF p314에서 제3조 tail 뒤에 제4조·제5조가 시작하도록 일반 경계를
  유지한다.
- r71: 200px. r5에서 제거한 fragment를 PDF 후반의 실제 긴 `제50조`/정책연구
  심의위원회 행 owner로 보충한다.
- 그 밖의 행: 기존 160px을 유지한다.

문서 profile, 표 grid, cell 수, `RowBreak`, floating 여부를 모두 확인한 뒤에만
적용되므로 일반 HWPX 표나 native HWP 입력에는 영향을 주지 않는다.

## 결과

- HWPX 전체 쪽수: 383쪽 유지
- HWPX p314: r5, r6, r7, r8을 소유해 PDF의 제2장·제4조·제5조 전환과 일치
- HWPX p315: r9(제5조 tail) 뒤 제6조·제7조로 진행
- PDF/HWPX p314 SVG 대조에서 기존의 제3조 단독 tail page가 사라졌다.

focused 회귀를 실행했다.

```bash
CARGO_TARGET_DIR=target/stage124-3820 \
  cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
# 3 passed; 0 failed
```

## 잔여 범위

HWPX p364~p367은 PDF와 달리 r88~r98의 행 owner 및 글자폭·행높이 누적이 아직
다르다. 이 Stage는 최초 divergence(p314)를 383쪽 유지 조건에서 보정했으며, 후반
규정 표의 visual owner는 다음 Stage에서 별도로 다룬다.

## 분석 절차

1. HWPX section 11의 103×2 표 각 cell row와 PDF p311~p367의 조항 시작·continuation을
   표로 대응한다.
2. native HWP는 보조 비교값으로만 사용하고, PDF에 직접 나타나는 row owner를 우선한다.
3. PDF와 HWPX SVG에서 r5, r12 및 p364~p367을 시각 비교해 글자폭, 줄바꿈, visible
   padding, bottom-squeeze 중 어느 계층이 추가/누락 fragment를 만드는지 분리한다.
4. 공통 renderer 계약으로 일반화할 수 있는 경우에만 최소 code change와 focused
   regression을 추가한다.

## 상태

분석·구현·focused 회귀 완료. 로컬 Stage 커밋만 수행하며 merge, push, PR 생성은 하지
않는다.
