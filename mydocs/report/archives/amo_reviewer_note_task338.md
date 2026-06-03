# Task M100 #338 AMO 검토자 설명 문서

- **대상**: rhwp Firefox 확장
- **이슈**: [#338](https://github.com/edwardkim/rhwp/issues/338)
- **작성일**: 2026-04-26
- **영문 제출본**: `mydocs/report/amo_reviewer_note_task338_en.md`

## 1. 목적

Firefox AMO 정적 분석에서 보고된 manifest 버전 경고와 보안 API 문자열 경고에 대해, 실제 수정 내용과 재검증 방법을 정리한다.

이 문서는 저장소 규칙에 따라 한국어로 작성한다. AMO 제출 화면의 `Notes for Reviewers`에 그대로 붙여 넣기 위한 영문본은 별도 파일 `amo_reviewer_note_task338_en.md`로 작성한다.

## 2. 제출 언어 판단

Firefox Extension Workshop 공식 문서는 AMO 제출 과정에서 `Notes for Reviewers` 입력란에 검토자를 돕는 정보를 넣을 수 있다고 설명한다. 예시는 dummy account, source code information 등이다.

또한 서드파티 라이브러리 정보는 AMO 제출 과정에서 `Notes for Reviewers`에 추가할 수 있다고 안내한다. 번들/빌드 산출물이 있는 확장은 reviewer가 빌드를 재현할 수 있도록 source code package와 README의 빌드 지침을 제공해야 한다.

확인한 공식 문서에서 `Notes for Reviewers`를 반드시 영어로 작성해야 한다는 명시 규정은 찾지 못했다. 다만 AMO reviewer를 대상으로 하는 제출 정보이고 공식 문서와 제출 UI가 영어 기준이므로, 실제 제출용 문구는 영어로 제공하는 것이 리뷰 지연 가능성을 줄이는 방식이다.

빌드 명령은 `Notes for Reviewers` 본문에 반드시 넣어야 하는 정보가 아니다. AMO에 source code package를 함께 제출하는 경우에는 공식 문서 안내에 따라 source package 안의 README 또는 빌드 스크립트에 재현 빌드 절차를 제공하는 것이 맞다. 따라서 영문 제출본은 reviewer note에 필요한 경고 처리 내용과 검증 결과 중심으로 작성한다.

## 3. Manifest 버전 경고 대응

`browser_specific_settings.gecko.data_collection_permissions`는 AMO 제출 요구에 따라 유지했다.

대신 `data_collection_permissions` 지원 버전과 manifest 최소 버전이 충돌하지 않도록 `strict_min_version`을 `112.0`에서 `142.0`으로 상향했다.

확인 대상:

- `rhwp-firefox/manifest.json`
- `rhwp-firefox/dist/manifest.json`

두 파일 모두 다음 값을 가진다.

```json
"strict_min_version": "142.0",
"data_collection_permissions": {
  "required": ["none"]
}
```

## 4. 빌드 산출물 정리

`rhwp-firefox/build.mjs`는 빌드 시작 전에 기존 `dist/` 디렉터리를 삭제하도록 변경했다.

효과:

- 과거 hashed viewer bundle 누적 제거
- 오래된 test artifact 제거
- AMO 제출 패키지가 최신 빌드 산출물만 포함

재빌드 후 `rhwp-firefox/dist/assets`에는 현재 viewer bundle 1개만 남는다.

```text
rhwp-firefox/dist/assets/rhwp_bg-DcCngJ7I.wasm
rhwp-firefox/dist/assets/viewer-Dk56CfXJ.js
rhwp-firefox/dist/assets/viewer-Di8-R0fz.css
```

`*.map`, `*test*`, `test/` 파일은 `dist/`에 남지 않는다.

## 5. `document.write` 경고 제거

인쇄 기능의 기존 구현은 `rhwp-studio/src/command/commands/file.ts`에서 `printWin.document.write(...)`로 인쇄 문서 전체를 작성했다.

변경 후:

- 인쇄 문서를 DOM API로 구성
- 파일명과 라벨은 `textContent`로 삽입
- SVG 페이지는 `DOMParser`로 SVG XML 파싱 후 print window 문서로 import
- 인라인 스크립트는 사용하지 않음

검증:

```bash
grep -R -l --exclude='*.wasm' 'document\.write' .
```

결과: 매치 없음.

## 6. 동적 `Function` 경고 확인

현재 재빌드된 제출 패키지에서는 이슈에 기록된 `Function` 생성자 경고가 재현되지 않는다.

검증:

```bash
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' .
```

결과: 매치 없음.

## 7. `innerHTML` 경고 제거

최종 재빌드된 제출 패키지에는 `innerHTML` 문자열 매치가 없다.

검증:

```bash
grep -R -l --exclude='*.wasm' 'innerHTML' .
```

결과: 매치 없음.

주요 수정 경로:

- 외부 커맨드 메뉴 label/shortcut 삽입을 `textContent` 기반 DOM API로 전환
- canvas container 초기화를 `replaceChildren()`으로 전환
- table/object 선택 overlay와 shape placement overlay를 DOM/SVG API로 전환
- content script 주석에서 정적 스캔 노이즈를 만들던 문자열 제거
- editor dialog, dropdown, preview widget을 DOM API, `replaceChildren()`, `textContent`, `option` 요소, SVG XML 파싱/import 기반으로 전환

## 8. 제출 패키지 기준 검증 명령

AMO `Notes for Reviewers`에 포함할 검증 명령은 빌드 전 저장소가 아니라 제출된 확장 패키지 루트를 기준으로 작성한다.

따라서 로컬 빌드 명령은 영문 제출본에서 제외한다. source code package를 별도로 제출하는 경우에만 해당 패키지의 README 또는 빌드 스크립트에 재현 빌드 절차를 작성한다.

```bash
grep -R -n -E '"strict_min_version"|"data_collection_permissions"' manifest.json
find . -path '*/test/*' -o -name '*test*' -o -name '*.map'
grep -R -l --exclude='*.wasm' 'document\.write' .
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' .
grep -R -l --exclude='*.wasm' 'innerHTML' .
```

패턴 검색 명령은 매치가 없을 때 종료 코드 `1`을 반환할 수 있다. 이 경우 출력이 없으면 기대한 결과다.

## 9. 브라우저 수동 검증

Firefox 확장을 로드한 상태에서 변경 전/후 산출물을 수동 비교했다.

최종 `innerHTML` 제거 후에는 인앱 브라우저가 아니라 로컬 Firefox 창을 직접 조작했다. 변경 전 산출물과 변경 후 산출물은 서로 다른 로컬 빌드로 제공해 비교했으며, 구체적인 로컬 포트는 AMO 검토자가 재현해야 하는 정보가 아니므로 생략한다.

확인 항목:

- content script badge 표시 및 viewer 열기
- HWP 문서 로드
- `document.write` 제거 후 인쇄 팝업 렌더링
- 새 문서 및 문서 교체 시 이전 페이지 제거
- 표 객체, 그림, 선, 사각형, 타원, 호, 다각형 선택 overlay
- 도형 배치 프리뷰 및 크기 라벨
- 도형 선택 팝업, 문자표 다이얼로그, 표 선택 팝업, 문단 모양 다이얼로그 변경 전/후 표시 비교

특이사항:

- 표 객체 핸들은 표시되고 hover cursor도 바뀐다.
- 다만 핸들 drag로 표 전체 크기를 변경하는 기능은 현재 `upstream/devel`에도 구현되어 있지 않다.
- 소스 확인 결과 표 객체 핸들 hit testing은 cursor 변경에만 사용되고, 표 내부 셀/행/열 resize는 `resizeTableCells` 경로로 별도 구현되어 있다.

## 10. 개인정보 처리 선언

확장은 다음과 같이 데이터 수집 없음을 선언한다.

```json
"data_collection_permissions": {
  "required": ["none"]
}
```

HWP/HWPX 문서는 브라우저 안에서 로컬로 처리된다. 확장은 사용자 문서 내용을 원격 서비스로 수집하거나 전송하지 않는다.
