# PR #2642: SECURITY.md 추가 — 보안 취약점 제보 정책 명시

## 이슈
- **Issue**: #2641 — 저장소에 SECURITY.md 없음

## 분석

보안 취약점(예: #2550 BinData 무제한 압축 해제)은 공개 이슈로
등록되기 전에 담당자에게 비공개로 전달되어야 한다.

GitHub는 SECURITY.md가 있으면 저장소 페이지에 Security 탭을
표시하고, 리포터가 취약점을 비공개로 제보할 수 있는 경로를 제공한다.

## 변경

저장소 루트에 `SECURITY.md` 신규 생성:
- 제보 방법 (이메일 및 Security Advisory)
- 예상 응답 시간 (7일)
- 책임 있는 공개 정책

## 결과
- 저장소에 Security 탭 활성화
- Closes #2641
