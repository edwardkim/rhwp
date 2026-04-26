[v0.2.1 변경 사항 / Changes — 2026-04-19]


━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
한국어
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

▣ v0.2.1 (2026-04-19) 주요 변경

[버그 수정]
• 일반 파일 다운로드의 "마지막 저장 위치 기억" 동작 복원 — 확장 활성 시
  바탕화면으로 떨어지던 문제 (chrome-fd-001 사용자 보고, #198)
• 옵션 페이지의 스크립트가 동작하지 않던 CSP 문제 수정 (#166)
• 일부 관공서 사이트 (DEXT5 류) 다운로드 시 빈 뷰어 탭이 뜨던 문제 차단 (#198)
• Windows 환경의 한글 파일 경로 처리 오류 수정
  (외부 기여 by @dreamworker0, PR #152)
• 모바일 드롭다운 메뉴 아이콘/라벨 겹침 수정
  (외부 기여 by @seunghan91, PR #161)
• 썸네일 로딩 스피너 정리 + options CSP 호환
  (외부 기여 by @postmelee, PR #168)

[기능 개선]
• HWP 파일 Ctrl+S 시 같은 파일에 직접 덮어쓰기 — 저장 다이얼로그가
  매번 뜨지 않음 (외부 기여 by @ahnbu, PR #189)
• 회전된 도형의 리사이즈 커서 + Flip(반전) 처리 개선
  (외부 기여 by @bapdodi, PR #192)
• HWPX 파일 열람 시 베타 안내 표시 + 직접 저장 비활성화
  (데이터 손상 방지, #196)
• HWPX Serializer — Document IR → HWPX 저장
  (외부 기여 by @seunghan91, PR #170)
• HWP 그림 효과(그레이스케일/흑백) SVG 반영 정확도 개선
  (외부 기여 by @marsimon, PR #149)
• HWPX ZIP 압축 한도 + strikeout shape 화이트리스트 + 도형 리사이즈 클램프
  (외부 기여 by @seunghan91, PR #153, #154, #163)
• 제품 정보 다이얼로그의 버전 표시 정상화

[알려진 한계]
• HWPX 직접 저장은 현재 베타 단계로 비활성화
  (HWPX→HWP 완전 변환기 #197 완성 시까지)
• 인쇄 미리보기 창 크기가 비정상적으로 크면 Ctrl+0 으로 리셋 가능 (#199)

[기여해주신 분들 — 감사합니다]
@ahnbu (PR #189)
@bapdodi (PR #192)
@dreamworker0 (PR #152)
@marsimon (PR #149)
@postmelee (PR #168)
@seunghan91 (PR #149, #153, #154, #161, #163, #170)


▣ v0.1.1 (이전)

• 초기 공개 베타


[전체 변경 이력]
https://github.com/edwardkim/rhwp/releases


━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
English
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

■ v0.2.1 (2026-04-19) Highlights

[Bug Fixes]
• Restored "remember last save location" for general file downloads
  (chrome-fd-001 user report, #198)
• Options page now works correctly (CSP fix, #166)
• Blocked empty viewer tabs on certain Korean government download
  handlers (DEXT5-style, #198)
• Fixed Korean file path handling on Windows
  (external contribution by @dreamworker0, PR #152)
• Mobile dropdown menu icon/label overlap fix
  (external contribution by @seunghan91, PR #161)
• Thumbnail loading spinner cleanup + options CSP compatibility
  (external contribution by @postmelee, PR #168)

[Improvements]
• HWP files: Ctrl+S now overwrites the same file directly — no save
  dialog every time (external contribution by @ahnbu, PR #189)
• Rotated shape resize cursor + Flip handling improved
  (external contribution by @bapdodi, PR #192)
• HWPX files now show a beta notice with direct save disabled
  (prevents data loss, #196)
• HWPX Serializer — Document IR → HWPX save
  (external contribution by @seunghan91, PR #170)
• HWP image effects (grayscale / black-and-white) reflected more
  accurately in SVG (external contribution by @marsimon, PR #149)
• HWPX ZIP entry decompression cap + strikeout shape whitelist
  + shape resize clamp
  (external contribution by @seunghan91, PR #153, #154, #163)
• About dialog version display fix

[Known Limitations]
• Direct HWPX save is in beta and disabled for now
  (until the HWPX→HWP full converter #197 lands)
• If the print preview window appears too large, press Ctrl+0 to
  reset zoom (#199)

[Thanks to Contributors]
@ahnbu (PR #189)
@bapdodi (PR #192)
@dreamworker0 (PR #152)
@marsimon (PR #149)
@postmelee (PR #168)
@seunghan91 (PR #149, #153, #154, #161, #163, #170)


■ v0.1.1 (previous)

• Initial public beta


[Full Changelog]
https://github.com/edwardkim/rhwp/releases
