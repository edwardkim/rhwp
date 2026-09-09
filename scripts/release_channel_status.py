#!/usr/bin/env python3
"""공개 registry에서 exact release channel version의 존재 여부를 판정한다."""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Mapping


CHANNELS = ("npm", "vscode-marketplace", "open-vsx")
SAFE_VALUE = re.compile(r"^[A-Za-z0-9@._+:/-]+$")
USER_AGENT = "rhwp-release-channel-status/1"
VSCODE_QUERY_URL = (
    "https://marketplace.visualstudio.com/"
    "_apis/public/gallery/extensionquery?api-version=7.2-preview.1"
)


class ChannelStatusError(RuntimeError):
    """상태 조회 결과가 부재와 구분되지 않거나 계약에 맞지 않는다."""


def _mapping(value: Any, code: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ChannelStatusError(code)
    return value


def _extension_identity(identifier: str) -> tuple[str, str]:
    if identifier.count(".") != 1:
        raise ChannelStatusError("extension-identifier-invalid")
    namespace, name = identifier.split(".", maxsplit=1)
    if not namespace or not name:
        raise ChannelStatusError("extension-identifier-invalid")
    return namespace, name


def interpret_response(
    channel: str,
    identifier: str,
    version: str,
    http_status: int,
    payload: Any,
) -> bool:
    """HTTP 응답을 exact version 존재 여부로 해석한다. 모호한 응답은 예외다."""

    if channel not in CHANNELS:
        raise ChannelStatusError("channel-not-supported")
    if http_status == 404 and channel in {"npm", "open-vsx"}:
        return False
    if http_status != 200:
        raise ChannelStatusError(f"unexpected-http-status:{http_status}")

    body = _mapping(payload, f"{channel}-response-invalid")
    if channel == "npm":
        if body.get("name") != identifier or body.get("version") != version:
            raise ChannelStatusError("npm-response-mismatch")
        return True

    namespace, name = _extension_identity(identifier)
    if channel == "open-vsx":
        if (
            body.get("namespace") != namespace
            or body.get("name") != name
            or body.get("version") != version
        ):
            raise ChannelStatusError("open-vsx-response-mismatch")
        return True

    results = body.get("results")
    if not isinstance(results, list):
        raise ChannelStatusError("vscode-marketplace-response-invalid")
    extensions: list[Mapping[str, Any]] = []
    for result in results:
        result_body = _mapping(result, "vscode-marketplace-result-invalid")
        result_extensions = result_body.get("extensions")
        if not isinstance(result_extensions, list):
            raise ChannelStatusError("vscode-marketplace-extensions-invalid")
        extensions.extend(
            _mapping(item, "vscode-marketplace-extension-invalid")
            for item in result_extensions
        )
    if not extensions:
        return False

    exact_extensions = []
    for extension in extensions:
        publisher = _mapping(
            extension.get("publisher"), "vscode-marketplace-publisher-invalid"
        )
        if (
            publisher.get("publisherName") == namespace
            and extension.get("extensionName") == name
        ):
            exact_extensions.append(extension)
    if len(exact_extensions) != 1:
        raise ChannelStatusError("vscode-marketplace-response-mismatch")
    versions = exact_extensions[0].get("versions")
    if not isinstance(versions, list):
        raise ChannelStatusError("vscode-marketplace-versions-invalid")
    available = []
    for item in versions:
        version_item = _mapping(item, "vscode-marketplace-version-invalid")
        value = version_item.get("version")
        if not isinstance(value, str):
            raise ChannelStatusError("vscode-marketplace-version-invalid")
        available.append(value)
    return version in available


def _decode_json(raw: bytes, channel: str) -> Mapping[str, Any]:
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ChannelStatusError(f"{channel}-response-invalid-json") from error
    return _mapping(value, f"{channel}-response-invalid")


def _request(
    request: urllib.request.Request, channel: str, timeout_seconds: int
) -> tuple[int, Mapping[str, Any]]:
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            return response.status, _decode_json(response.read(), channel)
    except urllib.error.HTTPError as error:
        raw = error.read()
        if error.code == 404 and channel in {"npm", "open-vsx"}:
            return error.code, {}
        try:
            payload = _decode_json(raw, channel)
        except ChannelStatusError:
            payload = {}
        return error.code, payload
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        raise ChannelStatusError(f"{channel}-request-failed:{type(error).__name__}") from error


def probe_channel(
    channel: str, identifier: str, version: str, timeout_seconds: int = 15
) -> bool:
    """인증 정보 없이 공개 registry의 exact version을 조회한다."""

    for label, value in (("identifier", identifier), ("version", version)):
        if not value or not SAFE_VALUE.fullmatch(value):
            raise ChannelStatusError(f"{label}-invalid")
    if not 1 <= timeout_seconds <= 60:
        raise ChannelStatusError("timeout-out-of-range")

    headers = {"Accept": "application/json", "User-Agent": USER_AGENT}
    if channel == "npm":
        quoted_identifier = urllib.parse.quote(identifier, safe="@")
        quoted_version = urllib.parse.quote(version, safe="")
        request = urllib.request.Request(
            f"https://registry.npmjs.org/{quoted_identifier}/{quoted_version}",
            headers=headers,
        )
    elif channel == "open-vsx":
        namespace, name = _extension_identity(identifier)
        request = urllib.request.Request(
            "https://open-vsx.org/api/"
            f"{urllib.parse.quote(namespace, safe='')}/"
            f"{urllib.parse.quote(name, safe='')}/"
            f"{urllib.parse.quote(version, safe='')}",
            headers=headers,
        )
    elif channel == "vscode-marketplace":
        body = json.dumps(
            {
                "filters": [
                    {
                        "criteria": [{"filterType": 7, "value": identifier}],
                        "pageNumber": 1,
                        "pageSize": 1,
                        "sortBy": 0,
                        "sortOrder": 0,
                    }
                ],
                "assetTypes": [],
                "flags": 1,
            },
            separators=(",", ":"),
        ).encode("utf-8")
        request = urllib.request.Request(
            VSCODE_QUERY_URL,
            data=body,
            headers={
                **headers,
                "Accept": "application/json;api-version=7.2-preview.1",
                "Content-Type": "application/json",
            },
            method="POST",
        )
    else:
        raise ChannelStatusError("channel-not-supported")

    status, payload = _request(request, channel, timeout_seconds)
    return interpret_response(channel, identifier, version, status, payload)


def _append_github_output(path: str, present: bool, state: str) -> None:
    with Path(path).open("a", encoding="utf-8") as output:
        output.write(f"present={'true' if present else 'false'}\n")
        output.write(f"probe_state={state}\n")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--channel", required=True, choices=CHANNELS)
    parser.add_argument("--identifier", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--timeout-seconds", type=int, default=15)
    parser.add_argument("--github-output")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        present = probe_channel(
            args.channel, args.identifier, args.version, args.timeout_seconds
        )
    except ChannelStatusError as error:
        print(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "accepted": False,
                    "channel": args.channel,
                    "identifier": args.identifier,
                    "version": args.version,
                    "error": str(error),
                },
                ensure_ascii=False,
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        return 1

    state = "already-present" if present else "not-present"
    result = {
        "schemaVersion": 1,
        "accepted": True,
        "channel": args.channel,
        "identifier": args.identifier,
        "version": args.version,
        "present": present,
        "state": state,
    }
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    if args.github_output:
        _append_github_output(args.github_output, present, state)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
