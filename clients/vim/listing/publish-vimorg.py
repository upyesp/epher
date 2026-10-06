#!/usr/bin/env python3
"""Upload a new epher version to the vim.org scripts site (script_id 6195).

vim.org has no API and no tokens: the scripts site is plain PHP web forms.
This script drives those forms the way a browser would, so the per-train
upload can run in CI:

  1. POST /login.php with the account credentials (env, never hardcoded),
     which establishes the session cookie. A failed login sets no cookie
     and re-renders the login form.
  2. GET /scripts/add_script_version.php?script_id=<id> and parse the
     upload form: its inputs, selects, and textareas.
  3. Echo every existing field back (hidden inputs, defaults, selected
     options), overriding only what a release carries: the version string,
     the Vim version, the release notes, and the package file. Parsing at
     runtime keeps the script working if the site renames fields.
  4. POST the multipart form, then verify the script page really shows the
     new version in its versions table.

Credentials come from the environment:

  VIMORG_USERNAME, VIMORG_PASSWORD   vim.org account (in CI: GitHub
                                     environment secrets in `stores`)
  VIMORG_SCRIPT_ID                   default 6195 (the epher script)
  ZIP                                path of the lean package zip
  VERSION                            release version, e.g. 0.5.64
  VIM_VERSION                        default 9.0 (site's compatibility field)
  RELEASE_NOTES                      text for the release-notes field

Usage:

  python3 publish-vimorg.py            # login, upload, verify
  python3 publish-vimorg.py --dry-run  # login, show the parsed form and
                                       # planned fields, upload nothing

Exit codes: 0 success or already-published, 1 failure, 2 configuration
error. The password and session cookie are never printed.
"""

import html
import html.parser
import http.cookiejar
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

BASE = "https://www.vim.org"
SCRIPT_ID_DEFAULT = "6195"
VIM_VERSION_DEFAULT = "9.0"
TIMEOUT = 60
USER_AGENT = "epher-release/1.0 (https://github.com/upyesp/epher CI)"
ZIP_WARN_BYTES = 150 * 1024  # largest package hosted on the site is ~163K


class Form:
    def __init__(self):
        self.action = ""
        self.method = "GET"
        self.inputs = []      # dicts: name, type, value, checked
        self.selects = {}     # name -> list of (value, text, selected)
        self.textareas = {}   # name -> content


class FormGrabber(html.parser.HTMLParser):
    """Collect every <form> on a page with its fields."""

    def __init__(self):
        super().__init__()
        self.forms = []
        self._cur = None
        self._sel = None
        self._opt = None   # [value, text_parts, selected]
        self._ta = None
        self._ta_parts = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "form":
            if self._cur is None:
                self._cur = Form()
                self._cur.action = a.get("action", "")
                self._cur.method = (a.get("method") or "GET").upper()
            return
        if self._cur is None:
            return
        if tag == "input":
            self._cur.inputs.append({
                "name": a.get("name", ""),
                "type": (a.get("type") or "text").lower(),
                "value": a.get("value", ""),
                "checked": "checked" in a,
            })
        elif tag == "select":
            self._sel = a.get("name", "")
            self._cur.selects.setdefault(self._sel, [])
        elif tag == "option" and self._sel is not None:
            self._opt = [a.get("value", ""), [], "selected" in a]
        elif tag == "textarea":
            self._ta = a.get("name", "")
            self._ta_parts = []
            self._cur.textareas.setdefault(self._ta, "")

    def handle_data(self, data):
        if self._opt is not None:
            self._opt[1].append(data)
        elif self._ta is not None:
            self._ta_parts.append(data)

    def handle_endtag(self, tag):
        if tag == "form" and self._cur is not None:
            self.forms.append(self._cur)
            self._cur = None
        elif self._cur is None:
            return
        elif tag == "select":
            self._sel = None
        elif tag == "option" and self._opt is not None:
            value, parts, selected = self._opt
            text = "".join(parts).strip()
            self._cur.selects[self._sel].append((value or text, text, selected))
            self._opt = None
        elif tag == "textarea":
            self._cur.textareas[self._ta] = "".join(self._ta_parts)
            self._ta = None


class Browser:
    def __init__(self):
        self.jar = http.cookiejar.CookieJar()
        self.opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(self.jar))
        self.opener.addheaders = [("User-Agent", USER_AGENT)]

    def get(self, url):
        with self.opener.open(url, timeout=TIMEOUT) as r:
            return r.geturl(), r.read().decode("utf-8", "replace")

    def post(self, url, data, content_type, referer=None):
        req = urllib.request.Request(url, data=data, method="POST")
        req.add_header("Content-Type", content_type)
        if referer:
            req.add_header("Referer", referer)
        with self.opener.open(req, timeout=TIMEOUT) as r:
            return r.geturl(), r.read().decode("utf-8", "replace")


def fail(msg, hint=""):
    print(f"error: {msg}")
    if hint:
        print(hint)
    sys.exit(1)


def parse_forms(page):
    grabber = FormGrabber()
    grabber.feed(page)
    return grabber.forms


def find_login_form(forms):
    for form in forms:
        if any(i["name"] == "password" for i in form.inputs):
            return form
    return None


def find_upload_form(forms):
    for form in forms:
        if any(i["type"] == "file" for i in form.inputs):
            return form
    return None


def login(browser, username, password):
    _, page = browser.get(BASE + "/login.php")
    form = find_login_form(parse_forms(page))
    if form is None:
        fail("the vim.org login page has no login form; the site may have changed",
             "Re-check the page shape in docs/research/vim-org-publishing.md.")
    fields = []
    for i in form.inputs:
        if i["type"] in ("submit", "button") and i["name"]:
            fields.append((i["name"], i["value"]))
        elif i["type"] == "hidden":
            fields.append((i["name"], i["value"]))
        elif i["type"] == "text":
            fields.append((i["name"], username))
        # password-type inputs: the value goes in the POST body below only
    body = dict(fields)
    body["password"] = password
    data = urllib.parse.urlencode(body).encode()
    url = urllib.parse.urljoin(BASE + "/login.php", form.action or "login.php")
    browser.post(url, data, "application/x-www-form-urlencoded")
    if not any(c.domain.endswith("vim.org") for c in browser.jar):
        fail("login failed: vim.org set no session cookie",
             "Check VIMORG_USERNAME / VIMORG_PASSWORD. Failed logins "
             "re-render the form and set no cookie.")


def pick_vim_version(options, want):
    for value, text, _ in options:
        if value == want or text.lower().startswith(want.lower()):
            return value
    return options[0][0] if options else ""


def plan_upload(form, zip_path, version, vim_version, release_notes):
    """Echo the form back with release overrides. Returns (fields, file_field,
    overrides, missing)."""
    fields = []
    file_field = None
    overrides = []
    notes_done = False
    for i in form.inputs:
        name, typ = i["name"], i["type"]
        if typ == "file":
            with open(zip_path, "rb") as f:
                content = f.read()
            file_field = (name, os.path.basename(zip_path), content,
                          "application/zip")
            overrides.append((name, "(file)", f"{zip_path} ({len(content)} bytes)"))
        elif typ in ("submit", "button"):
            # handled below: browsers send only the clicked button, and
            # sites with several same-named submits (upload/cancel) take
            # the last value, so echoing them all would cancel the upload
            pass
        elif typ == "hidden":
            fields.append((name, i["value"]))
        elif typ in ("checkbox", "radio"):
            if i["checked"] and name:
                fields.append((name, i["value"]))
        elif typ == "text":
            lowered = name.lower()
            if "vim" in lowered and "version" in lowered:
                # a plain text Vim-version field, if the site uses one
                fields.append((name, vim_version))
                overrides.append((name, i["value"], vim_version))
            elif "version" in lowered:
                # the release version: "version", "script_version", ...
                fields.append((name, version))
                overrides.append((name, i["value"], version))
            else:
                fields.append((name, i["value"]))
        elif typ == "password":
            continue
        else:  # unknown types: pass the default through
            if name:
                fields.append((name, i["value"]))
    # send exactly one submit: the first that does not look like a cancel
    submits = [i for i in form.inputs
               if i["type"] in ("submit", "button") and i["name"]]
    primary = next((i for i in submits
                    if i["value"].lower() not in ("cancel", "back", "reset")),
                   None)
    if primary is not None:
        fields.append((primary["name"], primary["value"]))
        others = [i["value"] for i in submits if i is not primary]
        overrides.append((primary["name"],
                          f"(submit; not sent: {others})" if others else "(submit)",
                          primary["value"]))
    for name, options in form.selects.items():
        if "vim" in name.lower():
            chosen = pick_vim_version(options, vim_version)
        else:
            chosen = next((v for v, _, sel in options if sel),
                          options[0][0] if options else "")
        fields.append((name, chosen))
        overrides.append((name, f"(select: {'/'.join(t for _, t, _ in options)})",
                          chosen))
    textareas = list(form.textareas.items())
    notes_name = next((n for n, _ in textareas
                       if any(k in n.lower()
                              for k in ("release", "notes", "comment"))), None)
    if notes_name is None and len(textareas) == 1:
        notes_name = textareas[0][0]
    for name, content in textareas:
        if name == notes_name and not notes_done:
            fields.append((name, release_notes))
            overrides.append((name, "(textarea)", release_notes))
            notes_done = True
        else:
            fields.append((name, content))
    return fields, file_field, overrides, notes_done


def page_summary(page, limit=400):
    """Title and a stripped-text excerpt, for failure diagnostics."""
    m = re.search(r"<title>(.*?)</title>", page, re.S)
    title = html.unescape(m.group(1)).strip() if m else "(no title)"
    text = re.sub(r"<script.*?</script>|<style.*?</style>", " ", page, flags=re.S)
    text = re.sub(r"<[^>]+>", " ", text)
    text = re.sub(r"\s+", " ", text).strip()
    return title, text[:limit]


def multipart_body(fields, file_field, boundary):
    out = bytearray()
    for name, value in fields:
        out += (f"--{boundary}\r\n"
                f'Content-Disposition: form-data; name="{name}"\r\n\r\n'
                f"{value}\r\n").encode("utf-8")
    name, filename, content, ctype = file_field
    out += (f"--{boundary}\r\n"
            f'Content-Disposition: form-data; name="{name}"; '
            f'filename="{filename}"\r\n'
            f"Content-Type: {ctype}\r\n\r\n").encode("utf-8")
    out += content + b"\r\n"
    out += f"--{boundary}--\r\n".encode("utf-8")
    return bytes(out)


def main():
    args = sys.argv[1:]
    dry_run = "--dry-run" in args
    cfg = {
        "username": os.environ.get("VIMORG_USERNAME", ""),
        "password": os.environ.get("VIMORG_PASSWORD", ""),
        "script_id": os.environ.get("VIMORG_SCRIPT_ID", SCRIPT_ID_DEFAULT),
        "zip": os.environ.get("ZIP", ""),
        "version": os.environ.get("VERSION", ""),
        "vim_version": os.environ.get("VIM_VERSION", VIM_VERSION_DEFAULT),
        "notes": os.environ.get("RELEASE_NOTES", ""),
    }
    missing = [k for k in ("username", "password", "zip", "version")
               if not cfg[k]]
    if missing:
        print("error: missing environment: " + ", ".join(missing))
        print("set VIMORG_USERNAME, VIMORG_PASSWORD, ZIP, VERSION "
              "(VIM_VERSION, VIMORG_SCRIPT_ID, RELEASE_NOTES optional)")
        sys.exit(2)
    if not os.path.isfile(cfg["zip"]):
        print(f"error: ZIP not found: {cfg['zip']}")
        sys.exit(2)
    size = os.path.getsize(cfg["zip"])
    if size > ZIP_WARN_BYTES:
        print(f"warning: the zip is {size} bytes; vim.org silently rejects "
              "packages over its upload cap (largest hosted today ~163K). "
              "Ship the lean zip, not the full release artifact.")

    script_url = f"{BASE}/scripts/script.php?script_id={cfg['script_id']}"
    browser = Browser()

    print(f"logging in to vim.org as {cfg['username']}")
    login(browser, cfg["username"], cfg["password"])
    print("login ok (session cookie received)")

    _, script_page = browser.get(script_url)
    if cfg["version"] in script_page:
        print(f"version {cfg['version']} is already on the script page; "
              "nothing to do")
        return

    version_url = (f"{BASE}/scripts/add_script_version.php"
                   f"?script_id={cfg['script_id']}")
    final_url, page = browser.get(version_url)
    if "login.php" in final_url or find_upload_form(parse_forms(page)) is None:
        fail("the version-upload form is not reachable with this session",
             f"landed on {final_url}. If this persists with valid "
             "credentials, the site may have changed; see "
             "docs/research/vim-org-publishing.md.")
    if dry_run:
        # dump the raw upload form (anchored on the file input, since the
        # page header carries its own search form)
        idx = page.find("script_file")
        if idx >= 0:
            start = page.rfind("<form", 0, idx)
            end = page.find("</form>", idx)
            print("--- raw upload form ---")
            print(page[start:end + 7] if start >= 0 and end >= 0 else "(bounds not found)")
            print("--- end raw form ---")
        else:
            print("--- no script_file anywhere on the page ---")
        # what does the logged-in script page expose per version?
        _, own_page = browser.get(script_url)
        print("--- logged-in versions table lines ---")
        seen = set()
        for line in own_page.splitlines():
            if re.search(r"download_script\.php|delete|add_script_version|edit_", line, re.I):
                stripped = line.strip()[:250]
                if stripped and stripped not in seen:
                    seen.add(stripped)
                    print(stripped)
        print("--- end versions table lines ---")
    form = find_upload_form(parse_forms(page))
    fields, file_field, overrides, notes_done = plan_upload(
        form, cfg["zip"], cfg["version"], cfg["vim_version"], cfg["notes"])

    print(f"upload form: action={form.action or '(self)'} method={form.method}")
    echoed = [f"{i['name']}={i['value']!r}" for i in form.inputs
              if i["type"] in ("hidden", "submit", "button") and i["name"]]
    print(f"echoing fields: {', '.join(echoed) if echoed else '(none)'}")
    print("planned fields:")
    for name, old, new in overrides:
        shown = new if len(new) <= 60 else new[:57] + "..."
        print(f"  {name}: {old!r} -> {shown!r}")
    if not notes_done and cfg["notes"]:
        print("warning: no release-notes field found in the form; "
              "release notes will not be set")
    if dry_run:
        print("dry run: form parsed and fields planned; nothing uploaded")
        return
    if file_field is None:
        fail("the upload form has no file field")

    boundary = "----epher" + uuid.uuid4().hex
    body = multipart_body(fields, file_field, boundary)
    action = urllib.parse.urljoin(version_url, form.action or version_url)
    print(f"posting {len(body)} bytes to {action}")
    resp_url, response = browser.post(action, body,
                                      f"multipart/form-data; boundary={boundary}",
                                      referer=version_url)
    print(f"response: {resp_url}")
    print("response page:", " | ".join(page_summary(response)))
    if find_upload_form(parse_forms(response)) is not None:
        fail("vim.org re-rendered the upload form; the upload was not saved",
             "The site rejects oversized packages silently, exactly this "
             "way. Check the zip size, or inspect the response above.")

    # the versions table is db-driven; give it a short window before
    # declaring failure
    for attempt in range(4):
        _, script_page = browser.get(script_url)
        if cfg["version"] in script_page:
            print(f"verified: version {cfg['version']} now appears on "
                  f"{script_url}")
            return
        if attempt < 3:
            print(f"version not visible yet (attempt {attempt + 1}/4); "
                  "waiting 15s")
            time.sleep(15)
    fail("the POST did not error, but the script page does not show "
         f"version {cfg['version']}",
         "The response page above shows what vim.org sent back; check it "
         "before re-running, since a second upload of the same version "
         "could duplicate a row.")


if __name__ == "__main__":
    main()
