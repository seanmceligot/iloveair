from typing import List

from icecream import ic
from lxml import etree, html
from util import parse_float, parse_int


def xpath_get_str(zz) -> str:
    ic(zz)
    if isinstance(zz, List):
        ic(("got.list", len(zz)))
        for item in zz:
            if len(zz) == 0:
                return ""
            elif len(zz) == 1:
                return xpath_get_str(item)
            else:
                types = set(map(type, zz))
                if types == {etree._ElementUnicodeResult}:
                    return " ".join(map(lambda el: el.strip(), zz))
            raise ValueError(f"Unexpected type {type(zz)} {html.tostring(zz)}")
    elif isinstance(zz, html.HtmlElement):
        raise ValueError(f"Unexpected type {type(zz)} {html.tostring(zz)}")
    elif isinstance(zz, etree._ElementUnicodeResult):
        return zz.strip()
    else:
        ic((type(zz), zz))
        raise ValueError(f"Unexpected type {type(zz)}")
    raise ValueError("Unexpected return")


def xpath_get_int(zz) -> int:
    return parse_int(xpath_get_str(zz))


def xpath_get_float(zz) -> float:
    return parse_float(xpath_get_str(zz))
