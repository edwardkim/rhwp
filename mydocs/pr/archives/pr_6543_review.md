# PR #6543 검토 - 한컴 font face chain

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR head: `a3e1f514b9b7d52902b62f95a21f9b492745f674`
- 적용 commit: `3955515d3`, `7e903460d`
- 상태: 통합 병합 완료
- 통합 PR / merge: #6541 / `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`

## 계보와 구현

- source `1749531a9336a8fb9babf6400d9a590452eaca43` → integration `3955515d3`
- source `a3e1f514b9b7d52902b62f95a21f9b492745f674` → integration `7e903460d`

`HY신명조`, `HY헤드라인M`, `HY그래픽M`의 SVG/PDF local font chain 앞쪽에 Windows가 실제로
해석하는 접미사 제거 영문 family를 추가한다. 두 번째 commit은 SVG context를 자를 때 UTF-8 char
경계를 지키도록 기존 test를 보강한다.

## Chrome 151 host face 재계측

Windows 호스트 Chrome CDP에서 없는 글꼴 + `Times New Roman` 기준선과 같은 문자열을 Canvas2D로
그린 뒤 폭과 서로 다른 픽셀 수를 비교했다.

| 문서명 | name table 영문 | 접미사 제거 영문 |
|---|---|---|
| `HY신명조`: 0 px, Δ0 | `HYSinMyeongJo-Medium`: 0 px, Δ0 | `HYSinMyeongJo`: 35,193 px, Δwidth +284.663 |
| `HY헤드라인M`: 0 px, Δ0 | `HYHeadLine-Medium`: 0 px, Δ0 | `HYHeadLine`: 47,418 px, Δwidth +182.569 |
| `HY그래픽M`: 0 px, Δ0 | `HYGraphic-Medium`: 0 px, Δ0 | `HYGraphic`: 33,874 px, Δwidth +200.429 |

현재 호스트에서도 앞의 두 이름은 fallback 기준선과 같고 접미사 제거명만 설치 face를 선택한다.

## 결정성과 범위

- `issue-267`, `issue-617` SVG snapshot 통과
- 새 chain 문자열만 제거한 golden은 이전 golden과 SHA-256이 각각 동일
- font alias unit, UTF-8 boundary, #6514 자간 focused test 통과

이 PR은 core SVG/PDF 축을 닫는다. Studio가 core chain의 첫 이름을 버리고 자체 표를 만드는 별도 축은
#6263 후속 범위로 남기며, 이번 통합으로 해결됐다고 주장하지 않는다.

## Merge 후 contributor PR comment 계획

- source `1749531a9`, `a3e1f514b` → integration `3955515d3`, `7e903460d` →
  통합 merge `e9d2f8b25` 계보를 남긴다.
- Chrome 151에서 접미사 제거명만 실제 face를 선택한 재계측과 SVG 결정성 검증을 알린다.
- #6263은 이미 closed 상태이며, 이번 core SVG/PDF 통합이 Studio 잔여까지 해결한 것은 아님을
  issue 후속 comment에도 명시한다.
- 계보 comment를 게시한 뒤 원 PR #6543을 중복 병합하지 않고 close한다.
