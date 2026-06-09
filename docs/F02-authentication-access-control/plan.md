# Implementation Plan: Authentication & Access Control

**Prerequisites:**
- F01 Platform Foundation implemented (Axum REST/SSE base, `AppState`, error envelope, React shell with the reserved Clerk provider slot and unguarded router mount points).
- A Clerk instance (development) providing an issuer, JWKS endpoint, authorized parties, and a publishable key.
- New backend dependencies: `jsonwebtoken`; `reqwest` promoted to a runtime dependency for JWKS fetch.
- New frontend dependency: `@clerk/clerk-react`.
- Environment variables: backend `CLERK_ISSUER`, optional `CLERK_JWKS_URL`, `CLERK_AUTHORIZED_PARTIES`; frontend `VITE_CLERK_PUBLISHABLE_KEY` (see `frontend/.env.example`).

### Stage 1: Backend Auth Foundation

**1. Auth Dependencies & Configuration** - Add the JWT verification and HTTP-fetch dependencies to the backend manifest and extend the typed configuration layer with the Clerk issuer, JWKS URL, and authorized-parties settings, deriving the JWKS URL from the issuer when it is not supplied. Reference the spec's Technical Decisions and `config.rs` responsibilities.

**2. Auth Error Envelopes** - Extend the shared application error type with the authentication failure cases so handlers and middleware render the standard JSON envelope for missing/invalid tokens, an unreachable auth service, and cross-user access. Reference the spec's Error Handling section.

**3. JWKS Cache & Auth State** - Implement the in-memory JWKS cache that fetches and maps signing keys with a TTL and a refetch-on-unknown-key path, package it together with the Clerk configuration as a shared auth state, and thread that state into the application state. Reference the spec's `auth/jwks.rs`, `auth/mod.rs`, and `state.rs` responsibilities.

### Stage 2: Token Verification & Identity

**4. Token Verification** - Implement verification that selects the signing key by the token header, checks the signature, and validates the issuer, authorized party, and expiry claims, returning the decoded identity claims on success. Reference the spec's `auth/verify.rs` responsibilities.

**5. Users Table & Repository** - Author the migration that creates the user identity table keyed by the Clerk user id and implement the repository operations that just-in-time provision and read a user. Reference the spec's Data Model and `auth/user.rs` responsibilities.

**6. Auth Middleware & Ownership Guard** - Implement the middleware that sources the token from the request (header for REST, query parameter for SSE), verifies it, provisions the caller into the user table, and exposes the authenticated identity to handlers, plus the extractor and the ownership guard that later features will use to reject cross-user access. Reference the spec's `auth/middleware.rs` and `auth/extractor.rs` responsibilities.

### Stage 3: Protected Routing & Boot

**7. Protected Routing Split** - Split the API router so the health probe stays public while the current-user endpoint and the SSE heartbeat move behind the auth layer, and implement the current-user handler that returns the authenticated identity. Reference the spec's API Contracts and `routes/me.rs`, `routes/mod.rs` responsibilities.

**8. Router Assembly & Startup Warm-up** - Assemble the public and protected sub-routers, apply the auth layer to the protected group, and warm the JWKS cache during the boot sequence so the first authenticated request does not pay the fetch cost. Reference the spec's `app.rs` and `main.rs` responsibilities.

### Stage 4: Frontend Auth Integration

**9. Clerk Provider & Token Bridge** - Wrap the application providers in the Clerk provider using the publishable key and add the module-level token registry plus the bridge component that registers Clerk's token getter for use by the non-hook API client. Reference the spec's `main.tsx`, `lib/authToken.ts`, and `auth/AuthTokenBridge.tsx` responsibilities.

**10. API Client & SSE Token Attachment** - Update the REST client to attach the session token as a bearer credential and surface an unauthorized response as a typed error, and attach the token to SSE connection URLs so streaming requests carry authentication. Reference the spec's `lib/apiClient.ts` responsibilities and SSE token-sourcing decision.

**11. Route Guard & Auth Pages** - Implement the route guard that redirects unauthenticated visitors to sign-in and renders protected content when signed in, and add the sign-in and sign-up pages. Reference the spec's `auth/RequireAuth.tsx`, `pages/SignInPage.tsx`, and `pages/SignUpPage.tsx` responsibilities.

**12. Routing & Navigation Integration** - Register the public sign-in and sign-up routes, wrap the protected layout in the route guard, add the session/sign-out control to the navigation, and document the new publishable-key variable in the environment template. Reference the spec's `routes/router.tsx`, `components/NavBar.tsx`, and `.env.example` responsibilities.
