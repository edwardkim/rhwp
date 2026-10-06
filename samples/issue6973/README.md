# #6973 정식 회귀 입력

[이슈 #6973](https://github.com/edwardkim/rhwp/issues/6973)에서 식별한 공개 문서
**감정평가 및 감정평가사에 관한 법률 시행규칙 일부개정령(안)**(국토교통부 입법예고)의
원본 바이트를 등록했다. 비공개 PC 경로나 환경 변수가 없어 검사를 건너뛰는 방식은 쓰지 않는다.

- 파일: [83818-appraisal-rules-amendment.hwpx](83818-appraisal-rules-amendment.hwpx)
- SHA-256: `79b24b98874723f0d7952a772ec9b16b5cacc7d0c91438c700679addde2ba9b5`
- 크기: 64,492바이트
- 저장 제품: `hancom-office-2020`, `11.0.0.8808`
- 기준 엔진: **`2020`** (저장 제품이 `hancom-office-2020` 이므로
  [시각·fixture 증적](../../mydocs/manual/pr_review/visual_fixture_evidence.md) §3.5.1)
- **기준 PDF: `pdf/83818-appraisal-rules-amendment-2020.pdf`** — `engine: 2020` 으로 직접
  산출해 저장했다. **9쪽**, 260,345 bytes,
  SHA-256 `5e51c3a98b18959cea44be10b87328c492b43ab2dfdf6679e859f212add0c7ba`.

## 이 fixture 가 고정하는 것

`section3` 의 15행 × 2열 신·구조문대비표(`pageBreak="CELL"`)에서 **행 13** 은 두 셀 모두
저장 `lineseg` 12줄을 가지며 vpos 가 `0 → 2520 → … → 20160` 으로 오르다 **`li=9` 에서 0 으로
되감긴다**. 한/글이 앞 9줄을 8쪽에, 뒤 3줄을 9쪽에 둔 기록이다.

```text
  한/글 2020 (저장 버전)   9쪽
  rhwp (수정 전)          8쪽   ← 마지막 조각이 본문 바닥을 108.9px 넘어 이어진다
  rhwp (수정 후)          9쪽
```

수정 전에는 저장 증거가 두 관문에 걸려 버려졌다 — ① `row_has_single_visible_source_cell`
(가시 셀 1개)이 두 열 표에서 거짓이고, ② `direct_hwpx_cell_has_declared_stored_frame`
(프레임 span ≤ 선언 `cellSz height`)은 이 문서의 선언 높이가 **전 행 2416HU(한 줄 규모)** 로
유지되지 않아 span 28000HU 가 통과할 수 없다. 그래서 상한 없는 파생 꼬리가 문단 끝(12줄)까지
당겨 잔여 299.8px 에 388.3px 를 실었다.

⚠ 남는 17.7px 은 **다른 축**이다 — 한 쪽 앞 행 7 도 같은 형상(li=5 되감김)인데 rhwp 의 7쪽
예산이 5줄(168.0px)에 5.6px 부족해 4/4 로 끊고, 그 한 줄(33.6px)이 8쪽 잔여를 잡아먹는다.
예산 대 실측 어긋남(`#6923` · `#6976` 단계 3)의 몫이라 이 fixture 는 쪽수와 초과 상한만
계약한다.
