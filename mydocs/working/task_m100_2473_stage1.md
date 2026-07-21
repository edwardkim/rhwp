# 작업 2473 단계 1 - Safari 확장 소스 동등성 통합

## 범위

- 현재 메인터너 브랜치에 기여자 PR #2473, #2477, #2491을 통합한다.
- 지원 중인 Chrome/Firefox 확장과 Safari 확장의 manifest, content script capability, 아이콘,
  locale 구성을 동등하게 복원한다.

## 검토 근거

- PR 본문: Safari manifest 버전은 Chrome/Firefox의 `0.2.8`과 같아야 하고, 확장 준비 상태는
  이미 지원하는 `edit`, `print` capability를 알려야 하며, 누락된 Safari 패키지 asset과
  Dependabot ecosystem을 복원해야 한다.
- PR 코멘트: 검토 당시 세 PR 모두 없음.
- Chrome/Firefox manifest는 모두 `0.2.8`이며, Chrome locale JSON과 아이콘 blob 네 개는
  #2491 추가분과 정확히 일치한다.
- `npm/editor/package.json`과 `rhwp-vscode/package.json`은 모두 존재하고 JSON으로 파싱되므로,
  새 Dependabot directory는 유효하다.

## 검증 계획

1. 변경된 manifest·locale JSON 세 파일과 Safari JavaScript 문법을 검증한다.
2. 로컬 서명·프로젝트 상태가 허용하면 Chrome 확장 선행 빌드와 macOS converter 빌드를 포함해
   Safari 패키지를 빌드한다.
3. Safari 중심 그룹이 Chrome, Firefox, VS Code, npm 패키지 source tree의 동작을 바꾸지 않는지
   확인한다.
4. 최종 통합 PR 전 통합 전체 회귀에 이 그룹을 포함한다.
