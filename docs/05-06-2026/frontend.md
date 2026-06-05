# Poster Frontend Documentation

**Date:** 05-06-2026
**Version:** 0.1.0 (Prototype)
**Stack:** Elm 0.19.1

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Elm Application                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Browser Element                         │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │  │
│  │  │    init     │──│    update    │──│      view       │  │  │
│  │  │  (Model)    │  │  (Msg →      │  │  (Model →       │  │  │
│  │  │             │  │   Model,Cmd) │  │   Html Msg)     │  │  │
│  │  └─────────────┘  └──────────────┘  └─────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │                      Api.elm                              │  │
│  │  HTTP requests → http://localhost:3000/api/*              │  │
│  │  JSON decoders ← responses                                │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                    ┌─────────▼─────────┐                        │
│                    │   Rust Backend    │                        │
│                    │   (localhost:3000) │                        │
│                    └───────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## File Structure

```
frontend/
├── elm.json                    # Elm package config
├── public/
│   └── index.html              # SPA host + CSS (149 lines)
└── src/
    ├── Types.elm               # Type definitions (123 lines)
    ├── Api.elm                 # HTTP client + JSON decoders (220 lines)
    └── Main.elm                # App logic + views (945 lines)
```

**Total:** ~1,288 lines of Elm code

---

## Module Reference

### `Types.elm` (123 lines)
**Purpose:** All type definitions for the application.

**Model (application state):**
```elm
type alias Model =
    { page : Page                    -- Current active page
    , token : Maybe String           -- JWT auth token
    , userId : Maybe String          -- Current user ID
    , posts : List Post              -- Loaded posts
    , accounts : List Account        -- Connected accounts
    , calendar : List CalendarDay    -- Calendar data
    , compose : ComposeModel         -- Composer form state
    , error : Maybe String           -- Error message banner
    , success : Maybe String         -- Success message banner
    , loginEmail : String            -- Login form field
    , loginPassword : String         -- Login form field
    , registerEmail : String         -- Register form field
    , registerPassword : String      -- Register form field
    , loading : Bool                 -- Global loading state
    , publishing : Maybe String      -- Post ID being published
    , dashboardFilter : PostFilter   -- Active filter tab
    , deleteConfirm : Maybe String   -- Post ID pending deletion
    }
```

**Pages:**
```elm
type Page
    = Login | Dashboard | Accounts | Composer | Calendar | Analytics | Settings
```

**Post Filter:**
```elm
type PostFilter
    = All | Drafts | Scheduled | Published | Failed
```

**Domain Types:**
```elm
type alias Post =
    { id, content, status, platform, accountId : String
    , mediaUrl, scheduledAt, publishedAt : Maybe String
    }

type alias Account =
    { id, provider, providerUserId, username, createdAt : String
    , tokenExpiresAt : Maybe String
    }

type alias CalendarDay =
    { date : String, posts : List Post }

type alias ComposeModel =
    { content, platform, mediaType : String
    , selectedAccount, scheduledAt, mediaUrl, aiPrompt : Maybe String
    , aiGenerating : Bool
    }
```

**Msg (38 variants):**
- Navigation: `Navigate Page`
- Auth: `LoginEmail`, `LoginPassword`, `DoLogin`, `LoginResult`, `RegisterEmail`, `RegisterPassword`, `DoRegister`, `RegisterResult`, `Logout`
- Posts: `GotPosts`, `CreatePost`, `PostCreated`, `PublishPost`, `PostPublished`, `DeletePost`, `PostDeleted`
- Accounts: `GotAccounts`, `ConnectAccount`, `GotOAuthUrl`, `DeleteAccount`, `AccountDeleted`
- Compose: `UpdateComposeContent`, `UpdateComposePlatform`, `UpdateComposeAccount`, `UpdateComposeMediaType`, `UpdateComposeMediaUrl`, `UpdateComposeSchedule`, `UpdateAiPrompt`, `GenerateContent`, `GotGeneratedContent`, `ClearCompose`
- UI: `SetDashboardFilter`, `ShowDeleteConfirm`, `HideDeleteConfirm`, `DismissError`, `DismissSuccess`
- Calendar: `GotCalendar`

---

### `Api.elm` (220 lines)
**Purpose:** HTTP client functions and JSON decoders.

**Base URL:** `http://localhost:3000/api` (hardcoded)

**Auth:** Adds `Authorization: Bearer <token>` header when token is present.

**API Functions:**

| Function | Method | Endpoint | Returns |
|----------|--------|----------|---------|
| `loginRequest` | POST | /auth/login | `Cmd msg` (token) |
| `registerRequest` | POST | /auth/register | `Cmd msg` (token) |
| `getPosts` | GET | /posts | `Cmd msg` (List Post) |
| `getAccounts` | GET | /accounts | `Cmd msg` (List Account) |
| `getCalendar` | GET | /calendar | `Cmd msg` (List CalendarDay) |
| `createPost` | POST | /posts | `Cmd msg` (Post) |
| `publishPost` | POST | /posts/{id}/publish | `Cmd msg` (Post) |
| `deletePost` | DELETE | /posts/{id} | `Cmd msg` () |
| `generateContent` | POST | /ai/generate | `Cmd msg` (String) |
| `connectAccount` | POST | /accounts/connect | `Cmd msg` (String URL) |
| `deleteAccount` | DELETE | /accounts/{id} | `Cmd msg` () |

**JSON Decoders:**
- `postDecoder` — Decodes 8-field Post from snake_case JSON
- `accountDecoder` — Decodes 6-field Account
- `calendarDayDecoder` — Decodes date + nested posts array

**Pattern:** All functions use `Http.request` (not `Http.get`/`Http.post`) for full control over headers and method.

---

### `Main.elm` (945 lines)
**Purpose:** Application entry point, state management, all views.

**Entry Point:**
```elm
main : Program () Model Msg
main = Browser.element { init, update, subscriptions, view }
```

**Init:** Creates empty model with `page = Login`, no token, empty lists.

---

## Pages & Views

### Login Page
- Combined login + register form on single card
- Email + password inputs
- Enter key submits forms
- Loading state disables buttons
- Error banner for failed attempts

### Dashboard
- Filter tabs: All, Drafts, Scheduled, Published, Failed (with counts)
- Post cards with:
  - Status badge (color-coded)
  - Platform badge
  - Content preview
  - Media preview (if image)
  - Scheduled/published timestamps
  - Publish button (for draft/scheduled)
  - Delete button with confirmation
- Empty state message
- "+ New Post" button in header

### Composer
- **Left side (main):**
  - Account selector dropdown
  - Platform selector (Threads/Instagram)
  - Content textarea with character count (500 max)
  - Media type selector (Text/Image/Video)
  - Media URL input (conditional on type)
  - Schedule datetime picker
  - Save Draft button
- **Right side (sidebar):**
  - AI Assistant section with prompt input
  - Generate with AI button
  - Live post preview

### Accounts
- Grid of account cards
- Each card shows:
  - Provider icon (📸 Instagram / 💬 Threads)
  - Username
  - Provider name
  - Token expiry status (green dot)
  - Disconnect button
- "+ Connect" button in header
- Empty state with connect CTA

### Calendar
- Posts grouped by date
- Each day shows:
  - Date header
  - List of scheduled posts with time, content preview, platform badge
- Empty state message

### Analytics (Stub)
- Placeholder text: "Analytics coming soon"

### Settings (Stub)
- Placeholder text: "Settings coming soon"

---

## State Management Flow

```
User Action → Msg → update() → New Model + Cmd → Api.elm → HTTP Request
                                                              ↓
Model ← newModel ← update(Got*) ← Response ← Backend
```

**Example: Login Flow**
```
1. User types email → UpdateComposeEmail "user@example.com"
2. update() sets model.loginEmail
3. User clicks Login → DoLogin
4. update() sets loading=True, calls Api.loginRequest
5. Api.elm sends POST /api/auth/login
6. Backend returns { token, user_id }
7. LoginResult (Ok token) received
8. update() sets token, navigates to Dashboard
9. Navigate Dashboard triggers Api.getPosts
10. GotPosts (Ok posts) updates model.posts
```

**Key Patterns:**
- All API results flow through `Got*` messages
- Navigation triggers data fetching
- Error/success messages auto-dismiss (via DismissError/DismissSuccess)
- Loading states for async operations

---

## CSS & Styling

**Location:** `frontend/public/index.html` (inline `<style>` block, 149 lines)

**Design System:**
```css
:root {
    --primary: #1a1a2e;
    --primary-hover: #16213e;
    --secondary: #6c757d;
    --danger: #dc3545;
    --success: #28a745;
    --warning: #ffc107;
    --bg: #f5f5f5;
    --card-bg: white;
    --border: #e9ecef;
    --border-radius: 8px;
}
```

**Components:**
- `.navbar` — Dark top navigation
- `.page` — Centered content container (max 1200px)
- `.post-card` — White card with left border color by status
- `.account-card` — White card for connected accounts
- `.composer` — Two-column layout (main + sidebar)
- `.filter-tabs` — Pill-style filter buttons
- `.form-group` — Label + input pairs
- `.btn-primary`, `.btn-secondary`, `.btn-danger`, `.btn-ghost` — Button variants
- `.error-banner`, `.success-banner` — Toast notifications

**Responsive:** Media query at 768px collapses composer to single column.

---

## Critical Bottlenecks & Issues

### 🔴 High Priority

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| Token lost on page refresh | Model (in-memory) | User must re-login every refresh | Add localStorage persistence |
| Hardcoded API base URL | `Api.elm:12` | Can't deploy to different backend | Make configurable via flags |
| OAuth flow uses Http.get | `Main.elm` GotOAuthUrl handler | Won't work for redirects | Use proper redirect or deep link |

### 🟡 Medium Priority

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| No pagination | `Api.elm` getPosts | Performance degrades with many posts | Add pagination params |
| No token refresh | `Main.elm` | Token expires after 24h | Add refresh flow |
| Analytics/Settings stubs | `Main.elm` | Incomplete features | Implement pages |
| No media upload | Composer | URL-only media | Add file upload |
| 500 char limit not enforced | Composer | Meta API will reject | Add client-side validation |

### 🟢 Low Priority

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| No error details shown | Error banner | Generic messages | Parse API error responses |
| No loading skeleton | Dashboard | Flash of empty content | Add skeleton loaders |
| No keyboard shortcuts | All pages | Power user friction | Add shortcuts |
| No dark mode | All pages | User preference | Add theme toggle |

---

## Scenarios & Future Work

### Scenario 1: Token Persistence
**Problem:** User refreshes page, loses session.

**Solution:**
```elm
-- In Main.elm init
port module Main exposing (..)

-- Ports for localStorage
port saveToken : String -> Cmd msg
port loadToken : () -> Cmd msg

-- In init
init flags =
    ( { token = flags.token  -- Pass from JS
      , ...
      }
    , loadToken ()
    )
```

**JS side:**
```javascript
// In index.html
const app = Elm.Main.init({
    node: document.getElementById("elm-root"),
    flags: { token: localStorage.getItem("token") }
});
app.ports.saveToken.subscribe(token => localStorage.setItem("token", token));
```

### Scenario 2: Adding Pagination
**Problem:** Posts list grows unbounded.

**Backend changes needed:**
- Add `page` and `limit` query params to GET /api/posts
- Return `{ posts: [...], total: N, page: N, pages: N }`

**Frontend changes:**
```elm
type alias PaginatedPosts =
    { posts : List Post
    , total : Int
    , page : Int
    , pages : Int
    }

-- Add to Model
, currentPage : Int
, totalPages : Int

-- Add messages
| NextPage | PrevPage | SetPage Int
```

### Scenario 3: Media Upload
**Problem:** Users must host images externally.

**Solution:**
1. Add file input to composer
2. Upload to backend (or S3/Cloudflare R2)
3. Return URL for post creation

**Backend changes:**
- Add `POST /api/upload` endpoint
- Accept multipart form data
- Store in object storage
- Return public URL

### Scenario 4: Multi-Language Support
**Problem:** Hardcoded English strings.

**Solution:**
```elm
-- Types.elm
type alias translations =
    { login : String
    , register : String
    , dashboard : String
    , ...
    }

-- Load translations based on locale
```

### Scenario 5: Real-time Updates
**Problem:** Posts only update on page navigation.

**Solution:**
- Add WebSocket connection to backend
- Push status changes (published, failed)
- Update model in real-time

---

## Quick Reference

### Pages & Navigation

| Page | Route Trigger | Data Loaded |
|------|---------------|-------------|
| Login | Initial / Logout | None |
| Dashboard | Navigate Dashboard | GET /posts |
| Accounts | Navigate Accounts | GET /accounts |
| Composer | Navigate Composer | Accounts (for selector) |
| Calendar | Navigate Calendar | GET /calendar |
| Analytics | Navigate Analytics | None (stub) |
| Settings | Navigate Settings | None (stub) |

### Key User Flows

**Login:**
1. Enter email + password → Click Login
2. Token stored in model → Redirect to Dashboard

**Create Post:**
1. Navigate to Composer
2. Select account, platform, enter content
3. Optionally set schedule time
4. Click "Save Draft"
5. Redirect to Dashboard with success message

**Publish Post:**
1. Dashboard → Find draft/scheduled post
2. Click "Publish"
3. Loading state → Success/Error message
4. Post status updates to "published"

**Connect Account:**
1. Navigate to Accounts
2. Click "+ Connect"
3. Redirect to Meta OAuth (currently broken)
4. Callback stores account

### Building

```bash
cd frontend
elm make src/Main.elm --output=public/elm.js
# Opens public/index.html in browser
```

### Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| elm/browser | 1.0.2 | Browser application |
| elm/core | 1.0.5 | Core library |
| elm/html | 1.0.1 | HTML rendering |
| elm/http | 2.0.0 | HTTP requests |
| elm/json | 1.1.4 | JSON encode/decode |

---

*Generated by codebase analysis on 05-06-2026*
