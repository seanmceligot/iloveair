import os
import re
from datetime import datetime

import requests
from icecream import ic


def parse_int(s: str) -> int:
    ic(s)
    match = re.search(r"\d+", s)
    if match:
        number = int(match.group())
        ic(number)
        return number
    else:
        ic(f"No int {s}")
        raise ValueError(f"No int {s}")


def parse_float(s: str) -> float:
    ic(s)
    match = re.search(r"\d+\.\d+", s)
    if match:
        number = float(match.group())
        ic(number)
        return number
    else:
        ic(f"No float {s}")
        raise ValueError(f"No float {s}")


def is_recent_file(filename: str, within_minutes: int = 10) -> bool:
    """Return True if file exists and is less than minutes old"""
    if os.path.exists(filename):
        file_modify_date = datetime.fromtimestamp(os.stat(filename).st_mtime)
        if datetime.now() - file_modify_date < timedelta(minutes=within_minutes):
            return True
    return False


def save_request_response(response: requests.Response, filename: str):
    with open(filename, "w") as json_file:
        json_file.write(response.text)
