# Task #2951 최종 결과보고서

## 이슈 요약

**원 보고 제목**: hp:dutmal 의 mainText/subText 가 CDATA 로 인코딩된 경우 파서가 덧말(Ruby) 텍스트를 소실함 #2951

**증상**: `<hp:dutmal>`(덧말/Ruby) 하위 `<hp:mainText>`(기준 텍스트) / `<hp:subText>`(덧말 텍스트)가
`<![CDATA[...]]>`로 인코딩된 경우, 파서가 이를 인식하지 못해 빈 문자열로 소실됨.

## 원인

`src/parser/hwpx/section.rs`의 `read_dutmal_text` 함수가 quick-xml 이벤트 중
`Event::Text`, `Event::GeneralRef`만 매치하고 `Event::CData`를 처리하는 분기가 없었음.
CDATA 섹션은 `_ => {}` 폴백에 걸려 텍스트가 그대로 버려짐.

#2916(`hp:equation`의 `hp:script`, PR #2927)와 #2935(`hp:parameters`의
`stringParam(Command)`, PR #2943)에서 동일한 클래스의 결함이 이미 확인·수정된 바 있음.
이번 건은 세 번째 발생 지점인 `hp:dutmal`(덧말)에서 재현됨.

## 수정 내용

**파일**: `src/parser/hwpx/section.rs` (`read_dutmal_text`)

기존 `Event::Text` / `Event::GeneralRef` 분기 사이에 `Event::CData` 분기를 추가하여,
CDATA 내용을 `String::from_utf8_lossy`로 디코딩해 `text`에 이어붙이도록 함
(기존 header.rs `read_numbering_para_head_text`, section.rs `parse_field_parameters`의
CDATA 처리와 동일한 패턴).

```diff
             Ok(Event::GeneralRef(ref r)) => {
                 text.push_str(&decode_xml_general_ref(r));
             }
+            // [CDATA] dutmal(덧말)의 mainText/subText가 CDATA로 인코딩된 경우 처리하지
+            // 않으면 덧말 텍스트가 소실된다. #2916/#2935의 hp:script/stringParam CDATA
+            // 누락과 동일한 패턴.
+            Ok(Event::CData(ref cdata)) => {
+                text.push_str(&String::from_utf8_lossy(cdata.as_ref()));
+            }
             Ok(Event::End(ref ee)) => {
```

## 검증 결과

### 테스트 (red → green)

`dutmal_maintext_subtext_preserve_cdata` 단위 테스트 추가:
`mainText`에 `a<b`, `subText`에 `c>d`를 CDATA로 감싼 `<hp:dutmal>`을 파싱해
`Control::Ruby`의 `main_text`/`ruby_text`가 원문 그대로 보존되는지 확인.

- **수정 전 (red)**: `assertion left == right failed: mainText CDATA 가 소실되면 안 됨` — `left: "", right: "a<b"`
- **수정 후 (green)**: `test parser::hwpx::section::tests::dutmal_maintext_subtext_preserve_cdata ... ok`

### 빌드

- `cargo check --lib`: 통과 (경고 없음)

## 영향 범위

**영향 받음**: `<hp:dutmal>`의 `mainText`/`subText`가 CDATA로 인코딩된 HWPX 문서 (덧말에
`<`, `>`, `&` 등을 포함하는 경우 한/글이 CDATA로 저장할 수 있음).

**영향 없음**: CDATA를 쓰지 않는 일반 텍스트 덧말, 다른 컨트롤/요소의 파싱 경로.

## 산출물

- `mydocs/report/task_m100_2951_report.md` (본 문서)
- 소스: `src/parser/hwpx/section.rs` (`read_dutmal_text` 6줄 추가 + 테스트 1개 24줄 추가)

## 이슈 종료 조건

작업지시자 승인 후 `gh issue close 2951` 실행.
