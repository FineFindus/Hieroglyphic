#!/usr/bin/env python3

import requests
from bs4 import BeautifulSoup
import yaml
import os


scripts_folder = os.path.dirname(os.path.abspath(__file__))
symbols_file = os.path.join(scripts_folder, "../symbols.yaml")

resp = requests.get("https://typst.app/docs/reference/symbols/sym/")
if not resp.ok:
    resp.raise_for_status()

parsed_html = BeautifulSoup(resp.text)

commands = {}
for li in parsed_html.select(".symbol-grid li[id^=symbol-]"):
    latex_cmd = li.attrs.get("data-latex-name")
    typst_cmd = li.attrs.get("id")
    if not typst_cmd or not latex_cmd:
        continue

    typst_cmd = str(typst_cmd).removeprefix("symbol-")
    commands[latex_cmd] = typst_cmd

with open('../typst-aliases.yaml', 'w') as outfile:
    yaml.dump(commands, outfile, default_flow_style=False)
