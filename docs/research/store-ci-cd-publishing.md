# Store and marketplace CI/CD publishing: seven routes, researched

Research of 2026-09-17, in preparation for unparking marketplace publication
(the ADR-0068 gate stands until the user explicitly asks; this document does
not unpark anything, it prices the parking space). Question: for each of the
seven marketplaces/stores epher could ship through, what does publishing
actually require of CI, which credential, minted where, living how long, and
is there an OIDC route so CI holds nothing long-lived?

## Method

Fetched the primary sources on 2026-09-17: the official publisher docs, the
REST API references, and where the docs were thin, the first-party CLI source
itself (vsce `src/auth.ts`, snapcraft `snapcraft/commands/account.py` and
`snapcraft/store/client.py`, craft-store `craft_store/login/_ubuntuone.py`).
Every load-bearing claim carries the URL. Where a claim could not be verified
from a primary source, the text says so instead of guessing.

---

## 1. VS Code Marketplace

### The classic route: publisher + personal access token

The Marketplace runs on Azure DevOps. Publishing needs (1) a publisher
identity and (2) a Personal Access Token with the *Marketplace → Manage*
scope. The current flow in the official docs
([publishing-extension](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)):

1. Create an Azure DevOps organization
   ([create-organization](https://learn.microsoft.com/azure/devops/organizations/accounts/create-organization)).
2. Create a PAT: Azure DevOps portal → *User settings → Personal access
   tokens → New Token*, *Scopes: Custom defined*, scroll to **Marketplace**
   and select **Manage** (same page).
3. Create the publisher at the
   [publisher management page](https://marketplace.visualstudio.com/manage/publishers/)
, ID (permanent) and Name; verify with `vsce login <publisher>` and the
   PAT.
4. Publish: `vsce publish -p $PAT` (auto-increment with `vsce publish minor`,
   etc.), or `vsce package` and upload the vsix manually through the
   publisher page (same doc).

**But: global PATs die on 2026-12-01.** The Azure DevOps blog
([Retirement of global Personal Access Tokens](https://devblogs.microsoft.com/devops/retirement-of-global-personal-access-tokens-in-azure-devops/),
2025-12-12) retires *global* PATs, tokens scoped "All accessible
organizations", completely on **December 1, 2026** (the planned March 15
creation block was withdrawn; you can create globals until then). Tokens
scoped to a *single organization* survive. Since epher starts fresh, the PAT
should be org-scoped from day one. Note the docs still walk you through
"Organization: All accessible organizations" when creating the token, that
example ages out with the retirement; pick the single org.

**Max lifetime:** the current docs no longer state one fixed global maximum.
Organization policy can cap token lifetime ("Token lifetime restrictions …
for example, no tokens lasting more than 90 days",
[use-personal-access-tokens-to-authenticate](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate)),
and in Entra-backed orgs a PAT goes inactive if its user hasn't completed a
full sign-in within 90 days (same doc, "sign in with your new PAT within 90
days or it becomes inactive"). The traditional "custom-defined PATs max one
year" UI cap could **not** be verified in the current docs.

**Can a service principal mint Marketplace PATs? No.** The PAT Lifecycle
Management API exists,
`GET/POST/DELETE https://dev.azure.com/<org>/_apis/tokens/pats?api-version=7.1-preview.1`
([use-personal-access-tokens-to-authenticate](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate),
section "PAT Lifecycle Management APIs"), and requires a Microsoft Entra
access token. But the docs are explicit: *"Only users or apps that use an
'on-behalf-of user' flow can generate PATs. … As such, service principals or
managed identities can't create or manage PATs."* The recommended app scope
for the API is `vso.pats` (previously `user_impersonation`). Rotation via the
API is GET old → POST new → DELETE old (same doc's FAQ). So the PAT-within-CI
pattern fundamentally needs a *delegated user token*, which GitHub OIDC
cannot provide. The API is still useful to us: it returns `validTo` for each
token, which is exactly what the canary monitor (below) needs.

### The new route: Entra identity + workload identity federation (the OIDC answer)

The publishing docs now lead with a PAT-free path
([publishing-extension, "Secure automated publishing"](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)):

- Create a user-assigned **managed identity** in Azure, add a **federated
  credential** for it (workload identity federation), and wire an Azure
  DevOps service connection of type *Workload Identity Federation (manual)*.
- Authorize the identity in the Marketplace: *"Add the managed identity
  (using its resource ID) as a member of your publisher. Assign the
  **Contributor** role."*
- In CI, mint an Entra token and run **`vsce publish --azure-credential`**
  (requires **vsce ≥ 2.26.1**).

What token does the flag accept, exactly? The vsce source
([src/auth.ts](https://github.com/microsoft/vsce/blob/main/src/auth.ts))
chains `EnvironmentCredential → AzureCliCredential →
ManagedIdentityCredential(AZURE_CLIENT_ID) → AzurePowerShellCredential →
AzureDeveloperCliCredential` and requests the scope
**`499b84ac-1321-427f-aa17-267ca6975798/.default`**: the Azure DevOps
service-principal app ID, with tenant from `AZURE_TENANT_ID`.

The doc demonstrates this on Azure Pipelines. For GitHub Actions, the
mechanism is the same token: `azure/login` with `client-id`, `tenant-id`,
`permissions: id-token: write` exchanges the Actions OIDC token for an Entra
token via the managed identity's federated credential; any Entra token for
`499b84ac-1321-427f-aa17-267ca6975798` works, and vsce's
`AzureCliCredential` picks up `az login`'s token. (The GitHub-Actions
equivalent is inferred from the credential chain in vsce's source plus the
Entra federation docs, not shown as a worked example in Microsoft's docs.)

The generic federation mechanism, GitHub as external IdP, client-assertion
exchange, is documented in
[Workload identity federation](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation):
configure a user-assigned managed identity or app registration to trust
tokens from GitHub, then exchange them in the client-credentials flow. The
issuer/subject/audience in the exchanged token must **case-sensitively
match** the federated credential.

**Unverified claim, flagged:** the brief for this research said publisher
signup "now requires an Entra ID (Azure AD) organization-backed account".
The primary docs fetched do not say that; the current publishing-extension
page says to log into publisher management "with the same Microsoft account
you used to create the Personal Access Token", no Entra-org requirement
stated. What *is* verified: the retirement of global PATs and the
Entra-identity publishing path above. If an Entra-backed org is required
somewhere in the signup UI, that lives behind the login wall; treat the
claim as unconfirmed.

### Open VSX (the second registry, short story)

Same vsix, different registry and credential
([openvsx wiki: Publishing Extensions](https://github.com/eclipse-openvsx/openvsx/wiki/Publishing-Extensions)):

1. Eclipse account → log in at open-vsx.org → accept the **Publisher
   Agreement** on the profile page.
2. Create an access token: avatar → *Settings → Access Tokens → Generate New
   Token*. "An access token can be used to publish as many extensions as you
   like, **until it is deleted**", no expiry.
3. One-time: `npx ovsx create-namespace <name> -p <token>` (namespace
   ownership can be claimed separately, for the *verified* badge).
4. Per release: `npx ovsx publish epher-vscode.vsix -p <token>`.

The registry scans uploads at publish time (secret detection, file-hash
blocklist, namespace-similarity/typosquat checks), same wiki page.

---

## 2. JetBrains Marketplace

### Upload without Gradle: one multipart POST

The plugin zip our CI produces (the `jetbrains` job of
`build-installers.yml`) can go up with a plain HTTP call, no Gradle
publishing plugin involved
([Plugin upload API](https://plugins.jetbrains.com/docs/marketplace/plugin-upload.html)):

```
curl -i --header "Authorization: Bearer <token>" \
  -F pluginId=<pluginId> \
  -F file=@epher-jetbrains.zip \
  -F channel=<channel> \
  https://plugins.jetbrains.com/api/updates/upload
```

`pluginId` is the numeric ID from the plugin URL; `pluginXmlId` (our
`io.epher.jetbrains`) is accepted instead; `channel` empty means the default
Stable channel; `isHidden=true` uploads a update that won't go public after
approval. Maximum plugin size: **400 MB**. The same page documents the .NET
variant (`nuget push` against plugins.jetbrains.com). Our build technically
uses `gradle buildPlugin` (Gradle pinned in CI, no wrapper committed) to
produce `build/distributions/*.zip`, irrelevant for the upload itself,
which never depends on Gradle tooling.

### Tokens

One documented token type: the permanent token, minted in the profile
dashboard, *"You should create a permanentToken in **My Tokens** tab within
your JetBrains Marketplace profile dashboard"*
([plugin upload API](https://plugins.jetbrains.com/docs/marketplace/plugin-upload.html)).
The SDK docs add the UI path: profile page → *My Tokens* → name → *Generate
Token* → "Copy it before closing this page … This is the only time the token
is visible" ([publishing-plugin](https://plugins.jetbrains.com/docs/intellij/publishing-plugin.html)).
The brief asked about "the newer distribution/scoped tokens": **no such
token type appears in the current Marketplace docs**, the full docs
sitemap (checked 2026-09-17) contains no token-expiry or scoped-token page,
and the upload API doc describes only the permanent token. No documented
expiry; no documented revocation REST call; revocation is whatever the My
Tokens UI offers.

### Signing

[Plugin Signing](https://plugins.jetbrains.com/docs/intellij/plugin-signing.html)
(since the 2021.2 cycle) has the author sign the distribution; JetBrains
verifies and re-signs. It is **not a publication gate**: "If the author does
not sign the plugin or has a revoked certificate, a warning dialog will
appear in the IDE during installation." The `signPlugin` Gradle task runs
automatically before `publishPlugin` when a key is configured, and a
standalone ZIP Signer CLI exists for non-Gradle builds. epher can ship the
Marketplace upload unsigned (warning dialog) or sign it later with a key
kept in Actions secrets.

### Review

The
[JetBrains Marketplace Approval Guidelines v1.3 (effective 2026-03-31)](https://plugins.jetbrains.com/docs/marketplace/jetbrains-marketplace-approval-guidelines.html)
leave no auto-publish lane: *"Every new Plugin and all new Plugin versions
(also known as updates), are subject to a verification and approval
process"*; automated checks plus a manual, one-by-one review before public
availability; *"If you haven't heard from us within the next 3–4 working
days after the Plugin upload, please reach out to us at
marketplace@jetbrains.com."* No editing after submission except the
compatibility range ([plugin-updates](https://plugins.jetbrains.com/docs/marketplace/plugin-updates.html));
compatibility must be validated with the Plugin Verifier. CI implication:
the upload lands in review, and release trains must expect multi-day latency
between "uploaded" and "visible".

---

## 3. Eclipse Marketplace

**Honest verdict first: there is no CI upload route, and that is fine.**
Listing is a manual web form with human moderation:

- Accounts: the Marketplace uses Eclipse Foundation accounts
  ([Quickstart](https://marketplace.eclipse.org/quickstart): "Marketplace
  uses the same accounts from Eclipse Bugzilla, so if you need an account,
  go here to create one").
- Submission: log in → *Add Content* →
  [Add a new Solutions Listing](https://marketplace.eclipse.org/content/add-content)
  → "it will be placed into the Moderation Queue and should appear on
  Marketplace in the next **24 business hours**" (Quickstart, same link).
- Updates: "In order to edit your listings, you can visit the **My
  Marketplace** link at the top of the page" (Quickstart), also manual. No
  publisher-facing write API and no token is documented anywhere in the
  Marketplace docs; the `api/mpc` endpoints serve the Marketplace Client
  catalog, not publishers.

The one technical requirement to know:
[Eclipse Marketplace Client (drag-to-install)](https://marketplace.eclipse.org/quickstart)
only works when the listing carries an **Eclipse p2 update site URL and
default Feature IDs**, "Your product needs to be downloadable from an
Eclipse p2 update site." `epher-eclipse.jar` is a dropins bundle with no p2
site (ADR-0068 amendment), so epher's listing can be a catalog entry but
cannot offer one-click MPC install.

The correct model for epher is exactly the one the brief proposes: **CI
attaches `epher-eclipse.jar` to the GitHub release (it already does), and
the listing points at that URL**, ideally the stable
`releases/latest/download/epher-eclipse.jar` pattern (ADR-0068), so the
listing needs no per-release edits at all. The only recurring human work is
editing the listing text when it changes.

---

## 4. Sublime Text (Package Control)

There is no official store; the de-facto one is **Package Control**, and
submission is a pull request to the channel repository.

**Current channel repo: `sublimehq/package_control_channel`**: 
`wbond/package_control_channel` redirects there
([github.com/sublimehq/package_control_channel](https://github.com/sublimehq/package_control_channel),
master branch; note the Libraries registry moved separately to
`packagecontrol/channel`, per the README). The README's process:

1. Fork the channel repository.
2. Add the package details, alphabetically, to the correct JSON file in the
   `repository/` directory.
3. Open a PR and tick every box in the PR template.

The community guide
([docs.sublimetext.io: Submitting a package](https://docs.sublimetext.io/guide/package-control/submitting.html))
adds the constraints that shape epher's packaging: one package per Git
repository, **the package root must be the repository root**, submissions
only from maintainers, and **a valid semver tag must exist**. Updates then
require no auth at all: the channel's sources are "processed by a crawler
and compiled into" `channel_v3.json` (channel README), and "packages are
kept up-to-date automatically" from the repository's tags/releases
([about](https://packagecontrol.io/about)). New semver tag → new release in
Package Control, no recurring credential.

Honest packaging note: `epher-sublime.zip`, a config zip attached to our
releases, is *not* the shape Package Control consumes. A channel submission
would point at a repository whose root holds the Sublime package contents
(syntaxes/settings/commands), i.e. the material now under
`clients/sublime/` promoted to a repo root, with semver tags. The zip keeps
serving the direct-download flow; the channel is an additional public face,
not a rename.

---

## 5. Snap Store

### export-login: the flags and the defaults

The current snapcraft docs
([Authenticate](https://documentation.ubuntu.com/snapcraft/stable/how-to/publishing/authenticate/)):

```
snapcraft export-login <credentials-file> \
  --snaps=epher --channels=stable --acls=package_upload \
  --expires=2027-09-17T00:00:00Z
```

- `--snaps`, `--channels`, `--acls`: comma-separated limits; **default is
  all snaps, channels, and ACLs of the account**, always set them for CI.
- `--expires`: ISO 8601; the accepted formats are `%Y-%m-%d` and
  `%Y-%m-%dT%H:%M:%SZ` ([snapcraft source](https://github.com/canonical/snapcraft/blob/main/snapcraft/commands/account.py)).
- **Default expiry: one year**: not stated in the docs page; it comes from
  the CLI source, where the login call's `ttl` defaults to
  `timedelta(days=365)` and `--expires` merely overrides it
  ([snapcraft/store/client.py](https://github.com/canonical/snapcraft/blob/main/snapcraft/store/client.py)).
- Consume: `export SNAPCRAFT_STORE_CREDENTIALS=$(cat <file>)`; verify with
  `snapcraft whoami`, whose output includes the ACLs and an **`expires:`
  line**, the hook the canary monitor (below) reads.
- The docs recommend one export per machine/purpose; exported credentials
  from snapcraft 7 only work on 7.2+.

ACL semantics
([store macaroon reference](https://dashboard.snapcraft.io/docs/reference/v1/macaroon.html)):
`package_upload` is the bundle of `package_register`, `package_push`,
`package_release`, `package_update`, `package_metrics`, the right
least-privilege grant for the release train (`--snaps=epher
--channels=stable --acls=package_upload`).

### Login flow, and a breaking change we already owe

`snapcraft login` authenticates via Ubuntu One SSO (login.ubuntu.com).
**snapcraft 9 (current stable release: 9.0.1) removed the Candid login
path**: both `--experimental-login` and `SNAPCRAFT_STORE_AUTH=candid` now
raise "no longer supported" errors
([snapcraft/commands/account.py](https://github.com/canonical/snapcraft/blob/main/snapcraft/commands/account.py)).
Our one-time provisioning workflow
(`.github/workflows/snap-store-login.yml`) selects exactly that path
(`SNAPCRAFT_STORE_AUTH: candid` device flow, printed URL + code). On the
current `snapcraft` stable it will fail. Fix at provisioning time: pin the
snap to the `8.x` track (`snap install snapcraft --classic
--channel=8.x/stable`; tracks verified via the store's snap info API), or
migrate the provisioning script to the 9.x Ubuntu One prompt flow, which on
a headless runner means typing email/password/2FA into a non-interactive
run, so the 8.x pin is the pragmatic route. The login workflow already
runs inside a D-Bus session with an unlocked gnome-keyring; that part stays
valid.

### CI publish, rotation, OIDC

CI uses `snapcraft upload "$SNAP_FILE" --release=stable`
([upload command reference](https://documentation.ubuntu.com/snapcraft/9/reference/commands/upload/)),
exactly what the `snap` job of `release.yml` runs. Rotation: re-export
annually (default TTL) or set a chosen `--expires`; the store rejects
nothing about re-exporting, and `whoami`'s `expires:` line makes probing
trivial. **OIDC/federation: no such option is documented anywhere in the
snapcraft or store API docs**: store authentication is Ubuntu One SSO with
macaroon credentials (Candid was the legacy alternative, now removed).
Verdict confirmed: no federation route; a long-lived exported macaroon in
Actions secrets is the documented design, kept short-lived by `--expires`.

---

## 6. Flathub

### The model

An app lives in its own GitHub repository under the flathub org,
`flathub/<app-id>`, for us `flathub/com.epher.Desktop` (does not exist
yet; the name follows the Tauri bundle identifier, ADR-0061).

- **First submission**: a PR to `flathub/flathub` (against the `new-pr`
  base branch), reviewed by volunteers; *"Once the pull request has been
  submitted, a test build can be started on the pull request by commenting
  `bot, build`"* ([submission](https://docs.flathub.org/docs/for-app-authors/submission)).
  On approval the reviewers create the org repo and invite the author as a
  writer; 2FA on GitHub is required and the invite expires in a week.
- **Updates**: *"Flathub builds and publishes app updates after a change is
  made to an app's manifest"*; a PR to the app repo gets a test build;
  merging creates the official build, which *"if successful, will be
  directly published to Flathub. The exact time to publish can vary
  depending on the publish queue"*
  ([maintenance](https://docs.flathub.org/docs/for-app-authors/maintenance)).
  No credential ever touches Flathub's own infrastructure, the buildbot
  watches the repo; all we manage is GitHub access *to the repo*.

### How the bot pushes: what the docs actually say

Our `store-bumps` job pushes straight to the app repo's `master` with an
SSH deploy key (`FLATHUB_SSH_KEY`, per ADR-0061). The current docs
complicate that: *"The master branch … **and** `main`, `stable`, `beta/*`
and `stable/*` **are automatically protected** which means that you can
only merge pull requests and not push directly to them"*
([maintenance](https://docs.flathub.org/docs/for-app-authors/maintenance)).
The documented update workflow is PR-based. Neither deploy keys nor
fine-grained PATs are endorsed or forbidden anywhere in the docs, that
gap is real, not an oversight of this research. What *is* documented for
automation on Flathub-hosted repos
([github-actions](https://docs.flathub.org/docs/for-app-authors/github-actions)):
custom Actions that commit/push are allowed "in a controlled manner", but
on Flathub-repos *only* actions from the flathub org plus
`peter-evans/create-pull-request` are permitted. **Action item:** before
first publish, decide between (a) a PR-based bump, e.g. a scheduled or
`repository_dispatch` workflow *inside* the flathub app repo using
`GITHUB_TOKEN` and `peter-evans/create-pull-request`, no stored secret at
all, and (b) testing whether direct pushes with the deploy key still pass
branch protection. Do not assume ADR-0061's direct-push works.

### The alternative that deletes the problem: x-checker-data

Flathub runs *"a global External Data Checker action for all repositories
in the GitHub organisation **every two hours**"* (default branch only); if
the manifest's sources carry `x-checker-data` and an update is detected,
**the bot opens the update PR itself**
([maintenance](https://docs.flathub.org/docs/for-app-authors/maintenance),
[flatpak-external-data-checker](https://github.com/flathub-infra/flatpak-external-data-checker)).
Checker types include `rotating-url`, `html` (version-pattern +
url-template against a releases page), `json`, and more. For epher: point a
checker at the GitHub releases feed and let it emit the versioned
`override-pull` URL our manifest already consumes. Then epher's release
train doesn't bump Flathub at all, the bump PR arrives within two hours of
the tag and a human merges it after eyeballing the test build. Automerging
bot PRs (`"automerge-flathubbot-prs": true` in `flathub.json`) exists but
is restricted, granted when the app is *verified* or update volume is low,
and it needs a linter exception
([maintenance](https://docs.flathub.org/docs/for-app-authors/maintenance)).
epher.org domain verification (the
[verification](https://docs.flathub.org/docs/for-app-authors/verification)
flow) is therefore worth doing at submission time.

Small honesty note: the brief asked about an "endtoendtest" buildbot
setting. The documented test-build command on submission PRs is `bot,
build` (quoted above); I could **not** verify an `endtoendtest` variant
from a primary source in this session. The publish-delay reality is the
"publish queue" sentence quoted above; no fixed delay is documented.

---

## 7. Microsoft Store (Windows 11)

### The submission REST API

The
[Microsoft Store submission API](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services)
(the docs still say "Azure Active Directory"; read "Microsoft Entra ID"):

- **Setup, one time:** an Entra directory (association needs Global
  administrator on it); in Partner Center *Account settings → Users*, add
  the Entra application that will call the API, with the **Manager** role;
  copy its **Tenant ID** and **Client ID**, and *Add new key* to get the
  secret (**key**). The app must already exist in Partner Center (name
  reserved) and **one submission must be created manually in Partner
  Center first**, including the age-ratings questionnaire, before the
  API accepts programmatic submissions for that app. After that, a
  submission created by the API must be managed only through the API
  (touching it in Partner Center locks the API out).
- **Token:** `POST https://login.microsoftonline.com/<tenant_id>/oauth2/token`
  with `grant_type=client_credentials`, `client_id`, `client_secret`,
  `resource=https://manage.devcenter.microsoft.com`; valid **60 minutes**;
  refetch freely.
- **Endpoints:** base `https://manage.devcenter.microsoft.com/v1.0/my/`,
  e.g. `POST /applications/{applicationId}/submissions`,
  `POST …/submissions/{submissionId}/commit`,
  `GET …/submissions/{submissionId}/status`
  ([manage app submissions](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions)).

**Does that service principal support workload identity federation?**
Mechanically, Entra app registrations accept **federated credentials**
(GitHub issuer `https://token.actions.githubusercontent.com`, audience
`api://AzureADTokenExchange`, case-sensitive subject match) and then
authenticate via client assertion instead of a secret
([Workload identity federation](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation)).
But the Store submission API docs **document only the tenant/client/key
(secret) triple** and say nothing about federated credentials for the
Partner Center-associated app. Verdict, stated honestly: GitHub-OIDC is
*plausible* (same client-credentials endpoint, assertion in place of
secret) but **not documented for this API**, plan for a stored secret with
expiry monitoring, and treat federation as an experiment, not a dependency.

### MSIX: the packaging gate, and the good news about signing

The Store ingests MSIX. Our Windows artifact is an NSIS installer, and the
Tauri bundler ships exactly two Windows formats, `msi` and `nsis`, no MSIX
([tauri-bundler source tree](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri-bundler/src/bundle/windows)).
So Store distribution means **repackaging to MSIX in CI**: an
`AppxManifest.xml` whose identity (Name, Publisher) comes from Partner
Center's *View app identity details*, the payload files, and `makeappx`
from the Windows SDK to pack (Win32 apps run under `runFullTrust`). The
docs cover exactly this non-VS path: "If you don't use Visual Studio to
create your package, you must create your package manifest manually"
([app package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)).

The good news is signing: *"Your MSIX and AppX packages don't have to be
signed with a certificate rooted in a trusted certificate authority when
submitting to the Microsoft Store. The Microsoft Store will automatically
re-sign your MSIX/AppX packages with a Microsoft certificate during the
publishing process"*, no CA cert, no PFX, no HSM; the Store replaces any
existing signature. The same page warns the re-signing **does not** apply
to MSI/EXE submissions ("You must Authenticode-sign your MSI/EXE installer
yourself"), one more reason the repack, not a raw-EXE submission, is the
right route for an unsigned build like ours.

Gates: after commit, the submission goes through certification review
before release; pre-release testing uses **package flights** (same API
family,
[create-and-manage-submissions](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services)).

**Adjacent no-store route, one line:** winget distributes via manifest PRs
to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), no
store account at all, just GitHub.

---

## Cross-cutting: CI holds the keys

### Secrets hygiene on a public repo

- **Fork PRs get nothing:** "With the exception of `GITHUB_TOKEN`, secrets
  are not passed to the runner when a workflow is triggered from a pull
  request from a forked repository"
  ([using secrets](https://docs.github.com/en/actions/security-for-github-actions/security-guides/using-secrets-in-github-actions)).
  Our existing skip-with-notice pattern already assumes secrets may be
  absent; fork PRs are therefore safe by default.
- **Environment secrets + required reviewers:** store-credential jobs
  should run in a GitHub *environment* whose secrets exist only there and
  whose protection rules (up to 6 required reviewers, wait timer,
  self-review prevention) pass before the job starts
  ([environments](https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment)).
  For epher: one environment (say `stores`) holding
  `SNAPCRAFT_STORE_CREDENTIALS`, the future
  `JETBRAINS_MARKETPLACE_TOKEN` / `VSCODE_MARKETPLACE_PAT` /
  `OVSX_TOKEN` / `STORE_SUBMISSION_CLIENT_SECRET`, with the maintainer as
  required reviewer, a free, human-in-the-loop gate on every publish.
- **OIDC federation to Entra:** GitHub mints a per-job JWT at issuer
  `https://token.actions.githubusercontent.com`. The `sub` claim for a job
  in an environment is `repo:ORG/REPO:environment:NAME`; a pull-request
  trigger yields `repo:ORG/REPO:pull_request`; a branch,
  `repo:ORG/REPO:ref:refs/heads/BRANCH`
  ([OIDC reference](https://docs.github.com/en/actions/reference/security/oidc)).
  **Format break:** repos created after **July 15, 2026**, which includes
  `upyesp/epher` (created 2026-08-13): use the *immutable* subject format
  with owner and repo IDs: `repo:OWNER@OWNER-ID/REPO@REPO-ID:environment:NAME`.
  Any federated credential configured on an Entra identity for this repo
  must use the ID-bearing form.

### A credential-expiry canary

One scheduled workflow (cron, read-only `permissions: issues: write` only)
probes each credential and opens/updates an alert issue N days before the
earliest expiry. The probes, per credential, all read-only:

| Credential | Probe | Where the expiry is visible |
|---|---|---|
| Snap Store export | `SNAPCRAFT_STORE_CREDENTIALS=… snapcraft whoami` | output `expires:` line |
| Azure DevOps PAT (VS Code) | `GET /_apis/tokens/pats?api-version=7.1-preview.1` with an Entra token | `validTo` in the response |
| Entra app secret (Store API) | client-credentials token request + `GET …/applications` | secret expiry is visible in Entra/Partner Center; the probe merely proves liveness: note PATs can't be minted by SPs, so this credential's rotation is manual by design |
| JetBrains token | authenticated call against plugins.jetbrains.com; no documented introspection endpoint | no documented expiry, track the mint date in the alert issue instead |
| Open VSX token | authenticated publish-dry probe (or none) | none, "until it is deleted" |
| Flathub deploy key / GPG key | `git push --dry-run`, `gpg --list-keys` | SSH keys and self-created GPG keys don't expire unless made to |
| VS Code via OIDC (target state) | `az login` + one cheap `az rest` | nothing expires; the probe proves the trust still resolves |

The pattern is plain: a failing or near-expiry probe edits a single
"Issue: store credentials" item with the date and the renewal command,
sourcing renewal instructions from this document.

---

## Summary table

| Store | Auth artifact | Where minted | Max lifetime | Rotation automatable? | OIDC possible? | CI route | Our current state |
|---|---|---|---|---|---|---|---|
| VS Code Marketplace | PAT (Marketplace→Manage, org-scoped) or Entra identity as publisher member | Azure DevOps user settings / Azure portal | policy-capped (docs example 90 days); global PATs die 2026-12-01 | yes, via PAT Lifecycle API, but only with a delegated user token | **yes**, managed identity + WIF, `vsce publish --azure-credential` (≥2.26.1) | `vsce publish` in the release job | not started; `epher-vscode.vsix` already built and attached |
| Open VSX | access token | open-vsx.org Settings | none (until deleted) | trivially (mint new, delete old) | no | `ovsx publish` | not started; same vsix |
| JetBrains Marketplace | permanent token | profile → My Tokens | none documented | no introspection API documented | no | `curl -F file=@… https://plugins.jetbrains.com/api/updates/upload` | not started; `epher-jetbrains.zip` already built |
| Eclipse Marketplace | none (web form) |, |, | n/a | n/a | **none**, listing points at GitHub Releases | not started; `epher-eclipse.jar` already attached |
| Sublime (Package Control) | none after merge (PR once) | channel repo PR |, | n/a (tags drive updates) | n/a | PR to `sublimehq/package_control_channel`; tags do the rest | not started; needs a root-package repo, not the config zip |
| Snap Store | exported macaroon | `snapcraft export-login` on the login workflow | 1 year default; `--expires` settable | yes, re-export; `whoami` exposes `expires:` | no | `snapcraft upload --release=stable` | built; gated on `SNAPCRAFT_STORE_CREDENTIALS`; login workflow needs the snapcraft 8.x pin |
| Flathub | GitHub write access to the app repo (deploy key, or none if bot-PR route) | repo settings / invite | deploy keys don't expire | re-mintable | not applicable | buildbot builds on commit/merge; `x-checker-data` can automate bumps | built; `store-bumps` gated on `FLATHUB_SSH_KEY`; direct-push vs protected `master` needs resolving |
| Microsoft Store | Entra app (tenant, client id, secret) associated in Partner Center | Partner Center + Entra | secret ≤ 24 months at creation (Entra default range) | re-mint secret; FIC unverified for this API | undocumented for this API, assume no | submission REST API after MSIX repack in CI | not started; Windows artifact is NSIS, MSIX repack not built |

## Current state vs target, epher specifically

What `release.yml` does today (all jobs present, most gated on absent
credentials):

- `build` + `release`, always run; attach `epher-vscode.vsix`,
  `epher-jetbrains.zip`, `epher-visualstudio.vsix`, `epher-eclipse.jar`,
  the config zips (`epher-sublime.zip` among them), and the `epher-lsp`
  binaries to the GitHub release. This is the ADR-0068 distribution: the
  releases page *is* the store for every editor family today.
- `linux-repos`, gated on `GPG_PRIVATE_KEY` (plus optional
  `GPG_PASSPHRASE`); publishes the apt/dnf trees to `gh-pages`.
- `snap`, gated on `SNAPCRAFT_STORE_CREDENTIALS`;
  `snapcore/action-build` then `snapcraft upload --release=stable`,
  `continue-on-error` until the store ships routinely.
- `store-bumps`, gated on `FLATHUB_SSH_KEY`; clones
  `git@github.com:flathub/com.epher.Desktop.git` and runs
  `scripts/bump-stores.sh flathub …` (a **direct push to `master`**).
- `.github/workflows/snap-store-login.yml`, the one-time device-flow
  provisioning that exports `SNAPCRAFT_STORE_CREDENTIALS`; currently pins
  nothing and sets `SNAPCRAFT_STORE_AUTH: candid`, which snapcraft 9
  rejects.

Target state, per store:

1. **VS Code + Open VSX**: create the publisher (one-time, human); mint the
   Entra identity + federated credential and add it to the publisher with
   Contributor; add a `stores` environment with `AZURE_CLIENT_ID` /
   `AZURE_TENANT_ID` (identifiers, not secrets) and `permissions:
   id-token: write`; `vsce publish --azure-credential` plus
   `ovsx publish epher-vscode.vsix -p $OVSX_TOKEN`.
2. **JetBrains**: reserve the plugin (`io.epher.jetbrains`, already the
   xmlId in the packaged plugin.xml), mint the My Tokens token into the
   environment, and add a one-step `curl -F file=@epher-jetbrains.zip …
   /api/updates/upload` to the release job, then wait out the review.
3. **Eclipse**: create the listing by hand, pointing at
   `https://github.com/upyesp/epher/releases/latest/download/epher-eclipse.jar`;
   nothing enters CI. Update the listing text manually when it changes.
4. **Sublime**: decide whether a channel presence is worth a root-package
   repo; if yes, one PR to `sublimehq/package_control_channel`, then tags
   only.
5. **Snap**: fix `snap-store-login.yml` to
   `snap install snapcraft --classic --channel=8.x/stable`, export with
   `--snaps=epher --channels=stable --acls=package_upload --expires=…`,
   drop the exported file into the `stores` environment.
6. **Flathub**: human submission PR; enable `x-checker-data` in the
   manifest so Flathub's checker opens bump PRs; resolve the
   direct-push-vs-protected-`master` question before trusting
   `store-bumps`; verify epher.org for the verified badge.
7. **Microsoft Store**: only when asked, Partner Center account, one
   manual submission, Entra app + secret in the environment, an MSIX repack
   leg on a Windows runner, then the submission API. Winget is the cheap
   adjacent substitute.

The ADR-0068 gate, "marketplace publication is parked until the user
explicitly asks", remains the controlling decision; every row above
descends from a primary source and is ready to execute one store at a time
when that gate opens.
