#!/usr/bin/env python3
"""RHWP Agent Toolkit - Common Library
플레이북 패턴 자동화를 위한 공통 유틸리티
"""

import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Any
from enum import Enum


class ExitCode(Enum):
    """RHWP CLI 종료 코드"""
    SUCCESS = 0
    GENERAL_ERROR = 1
    FATAL_ERROR = 2
    VERIFY_GATE_FAIL = 3
    PAGE_COUNT_MISMATCH = 4


class RhwpToolkit:
    """RHWP CLI 자동화 도구킷"""

    def __init__(self, rhwp_binary: str = "rhwp", verbose: bool = False):
        self.rhwp_binary = rhwp_binary
        self.verbose = verbose

    def log(self, message: str, level: str = "INFO"):
        if self.verbose or level in ["ERROR", "WARNING"]:
            print(f"[{level}] {message}", file=sys.stderr)

    def run_command(self, args: List[str], stdin_data: Optional[str] = None):
        cmd = [self.rhwp_binary] + args
        self.log(f"Running: {' '.join(cmd)}")
        
        result = subprocess.run(
            cmd,
            input=stdin_data,
            capture_output=True,
            text=True,
            check=False
        )
        
        return result

    def parse_ndjson(self, ndjson_str: str) -> List[Dict]:
        """NDJSON 파싱"""
        results = []
        for line in ndjson_str.strip().split('\n'):
            if line:
                try:
                    results.append(json.loads(line))
                except json.JSONDecodeError:
                    pass
        return results
