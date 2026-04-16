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
4. **Expected:** Install prompt dialog appears with:
   - "Install IVISS App" title
   - Benefits list (quick access, offline, faster loading, full-screen)
   - "Not Now" and "Install" buttons

5. Click **"Install"**
6. **Expected:** 
   - Browser shows native install confirmation
   - App installs and opens in standalone window
   - Install prompt disappears

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
2. Wait for install prompt (3 seconds)
3. Click **"Not Now"**
4. **Expected:** Prompt closes and doesn't reappear

5. Refresh the page
6. **Expected:** Prompt does NOT appear (dismissed for 2 hours)

### Verification:
- Check localStorage: `pwa-install-dismissed` should have a timestamp
- Prompt should not appear for 2 hours

### Reset Dismissal:
Open browser DevTools → Console:
```javascript
localStorage.removeItem('pwa-install-dismissed');
location.reload();
```

---

## Test 3: Android Installation (Chrome)

### Steps:
1. Open Chrome on Android device
2. Navigate to IVISS application URL
3. Wait 3 seconds after page load
4. **Expected:** Install prompt dialog appears

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
3. **Expected:** Install prompt does NOT appear
4. Navigate through the app
5. **Expected:** No install prompts at any time

### Verification:
- App detects `(display-mode: standalone)` media query
- Install prompt component returns `null`

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
2. Verify install prompt behavior
3. Check offline functionality
4. Test automatic updates
