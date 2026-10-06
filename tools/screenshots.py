#!/usr/bin/env python3
"""Take the screenshots of the documentation (`doc/screenshots/`) from a running development server.

Usage:
    python tools/screenshots.py [--url http://localhost:8300] [--out doc/screenshots] [--learner camille]

The server must run with `--dev-login` on a tenant whose database can receive a demonstration learner: the
tool signs in as that learner and records quiz scores so that the pages have something to show (a completed
course, a course in progress, XP, a badge). Running it twice only refreshes the pictures.

It drives a headless Chrome through the DevTools protocol.

Requires: Google Chrome or Chromium, and the `websockets` package (pip install websockets).
"""

import argparse
import asyncio
import base64
import json
import shutil
import subprocess
import tempfile
import urllib.request
from pathlib import Path

import websockets

DESKTOP = (1280, 800)
PHONE = (400, 820)

# (file name, path, size, dark theme, signed in, full page)
SHOTS = [
    ("home", "/", DESKTOP, False, False, False),
    ("catalogue", "/catalogue/", DESKTOP, False, True, False),
    ("course", "/courses/docker-hello/", DESKTOP, False, True, False),
    ("lesson", "/courses/docker-hello/premier-conteneur/", DESKTOP, False, True, False),
    ("quiz", "/courses/docker-hello/premier-conteneur/#quiz-root", DESKTOP, False, True, False),
    ("dashboard", "/dashboard/", DESKTOP, False, True, False),
    ("badges", "/badges/", DESKTOP, True, True, False),
    ("exam", "/courses/docker-hello/exam/", DESKTOP, False, True, False),
    ("paths", "/paths/", DESKTOP, False, True, False),
    ("path-map", "/paths/devops-infrastructure/#map-title", DESKTOP, False, True, False),
    ("path-map-phone", "/paths/devops-infrastructure/", PHONE, True, True, True),
    ("lesson-phone", "/courses/git-basics/premier-commit/", PHONE, True, True, False),
]

# What the demonstration learner has done: every lesson of the first course, and part of the second.
PROGRESS = {"git-basics": None, "linux-shell": 3, "docker-hello": 1}


class Browser:
    """A page of a headless Chrome, driven through the DevTools protocol."""

    def __init__(self, socket):
        self.socket, self.next_id = socket, 0

    async def call(self, method, **params):
        self.next_id += 1
        await self.socket.send(json.dumps({"id": self.next_id, "method": method, "params": params}))
        while True:
            message = json.loads(await self.socket.recv())
            if message.get("id") == self.next_id:
                if "error" in message:
                    raise RuntimeError(f"{method}: {message['error']}")
                return message.get("result", {})

    async def evaluate(self, expression):
        result = await self.call("Runtime.evaluate", expression=expression, awaitPromise=True, returnByValue=True)
        if "exceptionDetails" in result:
            raise RuntimeError(result["exceptionDetails"].get("text", "script error") + ": " + expression[:80])
        return result["result"].get("value")

    async def open(self, url):
        # The very first connection to `localhost` may be refused while Chrome tries IPv6: try again.
        for _ in range(5):
            await self.call("Page.navigate", url=url)
            for _ in range(100):
                if await self.evaluate("document.readyState") == "complete":
                    break
                await asyncio.sleep(0.1)
            if await self.evaluate("!!document.querySelector('main')"):
                break
            await asyncio.sleep(0.5)
        else:
            raise RuntimeError(f"{url} does not answer with a page of the site")
        # Web fonts, images and the scroll to an anchor.
        await asyncio.sleep(0.6)


async def record_progress(page, base):
    """Signs in is done; answers the quizzes of the lessons listed in PROGRESS with a perfect score."""
    await page.open(base + "/catalogue/")
    script = """
    (async () => {
      const done = [];
      for (const [course, limit] of Object.entries(%s)) {
        const html = await (await fetch(`/courses/${course}/`)).text();
        const lessons = [...new DOMParser().parseFromString(html, 'text/html').querySelectorAll('.timeline__card a')]
          .map((a) => a.getAttribute('href').split('/').filter(Boolean).pop());
        for (const lesson of lessons.slice(0, limit ?? lessons.length)) {
          const page = await (await fetch(`/courses/${course}/${lesson}/`)).text();
          const data = new DOMParser().parseFromString(page, 'text/html').getElementById('lesson-data');
          if (!data) continue;
          const score = JSON.parse(data.textContent).quiz.length;
          const answer = await fetch('/api/progress', { method: 'POST', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ course, lesson, type: 'quiz', score }) });
          done.push(`${course}/${lesson}: ${answer.status}`);
        }
      }
      return done;
    })()
    """ % json.dumps(PROGRESS)
    return await page.evaluate(script)


async def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--url", default="http://localhost:8300")
    parser.add_argument("--out", default="doc/screenshots", type=Path)
    parser.add_argument("--learner", default="camille")
    options = parser.parse_args()
    base = options.url.rstrip("/")
    options.out.mkdir(parents=True, exist_ok=True)

    chrome = shutil.which("google-chrome") or shutil.which("chromium") or shutil.which("chromium-browser")
    if not chrome:
        raise SystemExit("Google Chrome or Chromium is required")
    profile = tempfile.mkdtemp(prefix="mentor-screenshots-")
    process = subprocess.Popen(
        [chrome, "--headless=new", "--remote-debugging-port=9333", f"--user-data-dir={profile}", "--hide-scrollbars", "--no-first-run", "about:blank"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        for _ in range(50):
            try:
                targets = json.load(urllib.request.urlopen("http://127.0.0.1:9333/json"))
                break
            except OSError:
                await asyncio.sleep(0.2)
        else:
            raise SystemExit("Chrome did not start")
        target = next(target for target in targets if target["type"] == "page")
        async with websockets.connect(target["webSocketDebuggerUrl"], max_size=None) as socket:
            page = Browser(socket)
            await page.call("Page.enable")
            await page.call("Runtime.enable")

            signed_in = False
            for name, path, (width, height), dark, needs_learner, full_page in SHOTS:
                if needs_learner and not signed_in:
                    await page.open(base + "/dev/login")
                    await page.evaluate(
                        f"fetch('/dev/login', {{method: 'POST', headers: {{'Content-Type': 'application/x-www-form-urlencoded'}}, body: 'username={options.learner}'}}).then(r => r.status)"
                    )
                    for line in await record_progress(page, base):
                        print("  ", line)
                    signed_in = True
                await page.call("Emulation.setDeviceMetricsOverride", width=width, height=height, deviceScaleFactor=1, mobile=width < 600)
                await page.call("Emulation.setEmulatedMedia", features=[{"name": "prefers-color-scheme", "value": "dark" if dark else "light"}])
                await page.open(base + path)
                # Notifications of a previous action would hide the page.
                await page.evaluate("document.querySelectorAll('.toast').forEach((toast) => toast.remove())")
                shot = await page.call("Page.captureScreenshot", format="png", captureBeyondViewport=full_page)
                if full_page:
                    size = await page.evaluate("[document.documentElement.scrollWidth, document.documentElement.scrollHeight]")
                    shot = await page.call(
                        "Page.captureScreenshot", format="png", captureBeyondViewport=True, clip={"x": 0, "y": 0, "width": size[0], "height": size[1], "scale": 1}
                    )
                (options.out / f"{name}.png").write_bytes(base64.b64decode(shot["data"]))
                print(f"{name}.png  {width}x{height}{' dark' if dark else ''}")
    finally:
        process.terminate()
        process.wait()
        shutil.rmtree(profile, ignore_errors=True)


if __name__ == "__main__":
    asyncio.run(main())
