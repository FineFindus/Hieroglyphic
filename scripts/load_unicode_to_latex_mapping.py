#!/usr/bin/env python3

import requests
from xml.etree import ElementTree
import yaml
import os


scripts_folder = os.path.dirname(os.path.abspath(__file__))
unicode_to_latex = os.path.join(scripts_folder, "../unicode_to_latex.yaml")

resp = requests.get("https://www.w3.org/Math/characters/unicode.xml")
if not resp.ok:
    resp.raise_for_status()

parsed_xml = ElementTree.fromstring(resp.text)


symbols = {}
for character_el in parsed_xml.iter("character"):
    unicode_symbol = character_el.attrib.get("id")
    latex_cmd = character_el.find("latex")
    if not unicode_symbol or latex_cmd is None:
        continue

    # only take the first 6 characters of the symbol's hex representation (e.g. "U1D7F7"),
    # as sometimes they're followed by additional variant info like "-0FE00" that we ignore
    symbols[unicode_symbol[:6]] = (latex_cmd.text or "").strip()

with open(unicode_to_latex, 'w') as outfile:
    yaml.dump(symbols, outfile, default_flow_style=False, encoding="utf-8")
