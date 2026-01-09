import asyncio
import sys
from playwright.async_api import async_playwright

async def run():
    print("🎭 Starting Headless Smoke Test...")
    async with async_playwright() as p:
        # Launch browser (try chromium first)
        try:
            browser = await p.chromium.launch(headless=True)
        except Exception as e:
            print(f"⚠️ Failed to launch chromium, trying webkit... Error: {e}")
            browser = await p.webkit.launch(headless=True)
            
        context = await browser.new_context()
        page = await context.new_page()

        # Capture console logs
        page.on("console", lambda msg: print(f"🖥️ [CONSOLE] {msg.type}: {msg.text}"))
        page.on("pageerror", lambda exc: print(f"❌ [PAGE ERROR] {exc}"))

        url = "http://localhost:1420"
        print(f"🔗 Navigating to {url} ...")
        
        try:
            await page.goto(url, timeout=30000)
        except Exception as e:
            print(f"❌ Navigation failed: {e}")
            await browser.close()
            sys.exit(1)

        print("⏳ Waiting 10s for hydration and websocket connection...")
        await asyncio.sleep(10)

        title = await page.title()
        print(f"📄 Page Title: {title}")
        
        # Get simplified body text to verify content loaded
        body_text = await page.evaluate("document.body.innerText")
        preview = body_text.replace('\n', ' ')[:200]
        print(f"📝 Content Preview: {preview}...")

        await browser.close()
        print("✅ Test Completed.")

if __name__ == "__main__":
    asyncio.run(run())
