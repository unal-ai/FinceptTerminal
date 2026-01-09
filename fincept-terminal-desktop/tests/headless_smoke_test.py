import asyncio
import sys
from playwright.async_api import async_playwright, TimeoutError as PlaywrightTimeout

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

        # Wait for the app to finish initializing
        # Look for content that indicates the app has loaded beyond "Initializing..."
        print("⏳ Waiting for app to fully load (max 60s)...")
        
        load_success = False
        try:
            # Wait for the Dashboard tab or any main content to appear
            # The app shows "Dashboard" tab when fully loaded
            await page.wait_for_selector(
                'text=Dashboard, text=Markets, text=Portfolio, [data-testid="main-content"]',
                timeout=60000
            )
            load_success = True
            print("✅ Main app content detected!")
        except PlaywrightTimeout:
            print("⚠️ Timeout waiting for main content, checking current state...")
        
        # Also wait for "Initializing" text to disappear
        if not load_success:
            try:
                await page.wait_for_function(
                    '''() => {
                        const text = document.body.innerText;
                        return !text.includes("Initializing") || text.length > 500;
                    }''',
                    timeout=30000
                )
                load_success = True
                print("✅ App finished initializing!")
            except PlaywrightTimeout:
                print("⚠️ App may still be initializing...")

        # Additional wait for any async data loading
        print("⏳ Extra 5s wait for async data...")
        await asyncio.sleep(5)

        title = await page.title()
        print(f"📄 Page Title: {title}")
        
        # Get simplified body text to verify content loaded
        body_text = await page.evaluate("document.body.innerText")
        preview = body_text.replace('\n', ' ')[:500]
        print(f"📝 Content Preview: {preview}...")
        
        # Check for common errors in content
        if "Initializing" in body_text and len(body_text) < 100:
            print("⚠️ WARNING: App appears stuck on initialization screen!")
            await browser.close()
            sys.exit(1)
        
        # Take a screenshot for debugging
        try:
            await page.screenshot(path="tests/headless_screenshot.png")
            print("📸 Screenshot saved to tests/headless_screenshot.png")
        except Exception as e:
            print(f"⚠️ Could not save screenshot: {e}")

        await browser.close()
        
        if load_success:
            print("✅ Test Completed Successfully!")
        else:
            print("⚠️ Test Completed with warnings - app may not have fully loaded")
            sys.exit(1)

if __name__ == "__main__":
    asyncio.run(run())
