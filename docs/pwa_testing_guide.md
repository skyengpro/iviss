# PWA Installation Testing Guide

This guide provides step-by-step instructions for testing the Progressive Web App (PWA) installation feature in IVISS.

## Prerequisites

- IVISS application running (development or production)
- Modern browser (Chrome, Edge, Safari, or Firefox)
- HTTPS connection (required for PWA features)

## Testing Environments

### Development Mode
```bash
cd frontend
npm run dev
```
Access at: `http://localhost:8080`

### Production Build
```bash
cd frontend
npm run build
npm run preview
```
Access at: `http://localhost:4173`

**Note:** PWA features work in both development and production modes.

---

## Test 1: Desktop Installation (Chrome/Edge)

### Steps:
1. Open Chrome or Edge browser
2. Navigate to the IVISS application URL
3. Wait 3 seconds after page load
4. **Expected:** Install banner appears at the top of the screen with:
   - X button on the left (to dismiss)
   - IVISS shield logo
   - "Get the app" heading
   - Description: "Fast, secure vehicle inspection and identification"
   - Blue "Install" button on the right

5. Click **"Install"**
6. **Expected:** 
   - Browser shows native install confirmation
   - App installs and opens in standalone window
   - Install banner disappears

### Verification:
- Check browser's address bar - should show app icon
- App should open in its own window (no browser UI)
- Check Chrome menu → "Apps" → IVISS should be listed

### Uninstall:
- Click three dots in app window → "Uninstall IVISS"
- Or: Chrome Settings → Apps → IVISS → Uninstall

---

## Test 2: Desktop Installation Dismissal

### Steps:
1. Open browser and navigate to IVISS
2. Wait for install banner (3 seconds)
3. Click **X button** (on the left side)
4. **Expected:** Banner closes immediately

5. Refresh the page (F5 or Ctrl+R)
6. **Expected:** Banner appears again after 3 seconds

### Verification:
- Banner reappears on every page refresh
- No localStorage persistence for dismissal
- Only permanently hidden after app is installed

### Reset Test:
Simply refresh the page to see the banner again
```javascript
location.reload();
```

---

## Test 3: Android Installation (Chrome)

### Steps:
1. Open Chrome on Android device
2. Navigate to IVISS application URL
3. Wait 3 seconds after page load
4. **Expected:** Install banner appears at the top of the screen

5. Click **"Install"**
6. **Expected:**
   - Android shows "Add to Home screen" confirmation
   - App icon appears on home screen
   - App opens in full-screen mode

### Verification:
- Find IVISS icon on home screen
- Tap icon - app opens without browser UI
- Check Android Settings → Apps → IVISS should be listed

### Uninstall:
- Long-press app icon → "Uninstall" or "App info" → Uninstall

---

## Test 4: iOS Installation (Safari)

### Steps:
1. Open Safari on iPhone/iPad
2. Navigate to IVISS application URL
3. **Note:** iOS doesn't support automatic install prompts
4. Tap the **Share** button (square with arrow)
5. Scroll down and tap **"Add to Home Screen"**
6. Tap **"Add"** in the top-right corner

### Verification:
- IVISS icon appears on home screen
- Tap icon - app opens in full-screen mode
- No Safari UI visible when app is open

### Uninstall:
- Long-press app icon → "Remove App" → "Delete App"

---

## Test 5: Automatic Updates

### Steps:
1. Install the PWA using any method above
2. Make a code change in the application
3. Rebuild the application:
   ```bash
   npm run build
   ```
4. Open the installed PWA
5. **Expected:** 
   - App automatically detects new version
   - Updates silently in the background
   - No user prompt or notification

### Verification:
- Check browser DevTools → Application → Service Workers
- Should show "activated and is running" status
- New version should be active after refresh

---

## Test 6: Offline Functionality

### Steps:
1. Install the PWA
2. Open the installed app
3. Navigate through a few pages
4. Open DevTools → Network tab
5. Check **"Offline"** checkbox
6. Try navigating to previously visited pages

### Expected:
- Previously visited pages load from cache
- Static assets (CSS, JS, images) load from cache
- API calls fail gracefully
- Offline fallback page appears for unvisited routes

### Verification:
- Check DevTools → Application → Cache Storage
- Should see cached files listed
- Service Worker should intercept requests

---

## Test 7: Already Installed Detection

### Steps:
1. Install the PWA
2. Open the installed app
3. **Expected:** Install banner does NOT appear
4. Navigate through the app
5. **Expected:** No install banner at any time

### Verification:
- App detects `(display-mode: standalone)` media query
- Install banner component returns `null`
- Banner never shows in installed app mode

---

## Test 8: Browser Compatibility

### Browsers to Test:

| Browser | Desktop | Mobile | Install Support |
|---------|---------|--------|-----------------|
| Chrome  | ✅ | ✅ | Full |
| Edge    | ✅ | ✅ | Full |
| Safari  | ✅ | ✅ | Manual only (iOS) |
| Firefox | ✅ | ✅ | Limited |
| Opera   | ✅ | ✅ | Full |

### Steps:
1. Test installation on each browser
2. Verify install banner behavior
3. Check offline functionality
4. Test automatic updates
