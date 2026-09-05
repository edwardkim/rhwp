# 봉투 필드 카탈로그

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 공통

성공 시 stdout 순수 JSON 하나. 실패 시 stdout 비움.

`schemaVersion: "1.0"`. 쪽·문단·구역은 0 기준. 사람 답은 page+1.

## hidden-text

| 필드 | 형 | 분기 |

|---|---|---|

| clean | bool | 예 |

| hiddenCharCount | number | 합계 |

| hiddenText[] | object | kind/section/paragraph/page?/charCount/excerpt |

| thresholdPt | number | 사용한 임계 |

| includeOffPage | bool | 검사 범위 |

| source | string | 입력 경로 |

## injection

| 필드 | 형 | 분기 |

|---|---|---|

| clean | bool | 예 |

| highestConfidence | low/medium/high/null | 예 |

| signalCount | number | 건수 |

| injectionSignals[] | object | kind/confidence/scope/matched/excerpt/why/주소 |

| scanScopes[] | string | 훑은 영역 |

| minConfidence | string | 필터 |

| includeFields | bool | 필드 축 |

## unicode

| 필드 | 형 | 분기 |

|---|---|---|

| clean | bool | 예 |

| findingCount | number | 건수 |

| findings[] | object | kind/codepoint/severity/rendered/raw/why/주소 |

| severityCounts | {high,medium,low} | 우선순위 |

| kindCounts | object | 축별 |

| kindFilter | string | 사용한 필터 |

| scannedChars | number | 1패스 규모 |

## redact

| 필드 | 형 | 분기 |

|---|---|---|

| findingCount | number | dry-run 게이트 |

| findings[] | object | kind/raw?/masked/주소 |

| noRaw | bool | 자동화 로그 |

| dryRun | bool | 파일 무변경 |

| redactedCount | number | 적용 횟수 |

| output? | string | 저장했을 때만 |

| verify? | object | identical/diffCount |

| changedPages | number[]/null | 렌더 확인 범위 |

| kinds | string[] | 사용한 종류 |

| mask | string | 한 글자 |

## sanitize

| 필드 | 형 | 분기 |

|---|---|---|

| removedCount | number | 첫 실행 >0, 둘째 0 |

| removed[] | {field,before} | 거짓 보고 없음 |

| keepPreview | bool | 이미지 보존 |

| output | string | 산출 경로 |

| outputFormat | hwp5/hwpx | info.format 과 같은 어휘 |

## 픽스처 위치

`fixtures/envelopes/` 에 축별 음성·양성·게이트 봉투가 있다.

테스트는 키 존재와 분기 필드 형만 고정한다. 라이브 바이너리를 요구하지 않는다.
