# Submitting epher to the Eclipse Marketplace

The channel is marketplace.eclipse.org, the listing site behind
Eclipse's Help, Eclipse Marketplace Client (MPC). The listing is a
pointer: once it carries an Update Site URL and Feature IDs, plugin
updates flow through p2 and the listing itself does not change. epher
is MIT, which satisfies the policy that MPC only returns listings with
an open source license (`docs/research/eclipse-marketplace-publishing.md`).

## What exists in this repo today

- **The LSP4E bundle** at
  `clients/eclipse/io.github.upyesp.epher.eclipse/`:
  `META-INF/MANIFEST.MF` (Bundle-SymbolicName
  `io.github.upyesp.epher.eclipse`, JavaSE-21, Require-Bundle
  `org.eclipse.lsp4e`), `plugin.xml` (the `.epher` content type, the
  LSP4E server plus content-type mapping, the shared TextMate grammar
  through TM4E, and the language configuration), and
  `EpherConnectionProvider.java`, which starts `epher-lsp` from PATH.
  `build-installers.yml` builds it into `epher-eclipse.jar`, a
  dropins bundle attached to every release. That install path works
  today but is invisible to MPC.
- **The listing body** in `DESCRIPTION.md` next to this file: the
  short description and the HTML body, adapted from
  `clients/vscode/README.md`.
- **The logo** at `icon.png`, 256x256 RGBA PNG. The Marketplace serves
  its `badge_logo` style at 80x80, and a square 256px render of the
  monogram is the safe upload.
- **The captures** in `clients/eclipse/images/` (`editor.png`,
  `hover.png`, `completion.png`, `results.png`, `demo.gif`) for the
  screenshots gallery. The body embeds no images; the top listings
  keep visuals in the gallery only.

## The one open build task: the p2 update site and feature

MPC cannot install a dropins jar. The listing's Install button needs a
p2 update site carrying a feature that wraps the existing bundle. This
is the only build work left for the channel, and it is deliberately
not implemented yet. The plan below is complete enough to execute.

### Tycho layout

```
clients/eclipse/
  pom.xml                                       aggregator, packaging pom
  io.github.upyesp.epher.eclipse/
    pom.xml                                     packaging eclipse-plugin
    build.properties                            new: what Tycho bundles
    META-INF/MANIFEST.MF                        existing
    plugin.xml                                  existing
    src/io/github/upyesp/epher/eclipse/
      EpherConnectionProvider.java              existing
  io.github.upyesp.epher.eclipse.feature/
    pom.xml                                     packaging eclipse-feature
    feature.xml
  io.github.upyesp.epher.eclipse.updatesite/
    pom.xml                                     packaging eclipse-repository
    category.xml
```

The aggregator pom owns the parent version, the module list, and the
target platform:

```xml
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>io.github.upyesp.epher</groupId>
  <artifactId>io.github.upyesp.epher.eclipse.parent</artifactId>
  <version>0.5.57</version>
  <packaging>pom</packaging>
  <modules>
    <module>io.github.upyesp.epher.eclipse</module>
    <module>io.github.upyesp.epher.eclipse.feature</module>
    <module>io.github.upyesp.epher.eclipse.updatesite</module>
  </modules>
  <properties>
    <!-- Pin the current Tycho release at implementation time. -->
    <tycho.version>PIN_AT_IMPLEMENTATION</tycho.version>
    <maven.compiler.release>21</maven.compiler.release>
    <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
  </properties>
  <repositories>
    <!-- The same LSP4E p2 release repo the current CI compiles against. -->
    <repository>
      <id>lsp4e</id>
      <layout>p2</layout>
      <url>https://download.eclipse.org/lsp4e/releases/latest/</url>
    </repository>
    <!-- Provides org.eclipse.equinox.common (IAdaptable), which the
         current CI takes from Maven Central; Tycho resolves it here. -->
    <repository>
      <id>eclipse-release</id>
      <layout>p2</layout>
      <url>https://download.eclipse.org/releases/2024-09</url>
    </repository>
  </repositories>
  <build>
    <plugins>
      <plugin>
        <groupId>org.eclipse.tycho</groupId>
        <artifactId>tycho-maven-plugin</artifactId>
        <version>${tycho.version}</version>
        <extensions>true</extensions>
      </plugin>
    </plugins>
  </build>
</project>
```

The bundle module keeps its existing files and gains a pom with
`<packaging>eclipse-plugin</packaging>` plus a `build.properties`:

```properties
source.. = src/
bin.includes = META-INF/, plugin.xml, syntaxes/, language-configuration.json
```

`plugin.xml` already points at `syntaxes/epher.tmLanguage.json` and
`language-configuration.json`. The current CI copies those two shared
files from `clients/shared/` and `clients/vscode/` into a temp bundle;
the Tycho build needs them inside the module, so the workflow keeps a
copy step in place (the same pattern as the Neovim runtime copies) or
the files are checked in with a sync script. Decide at implementation
time; the copies must come from the shared sources, not be edited here.

Version plumbing: `MANIFEST.MF` carries `Bundle-Version: 0.0.0` today
and the current CI rewrites it. With Tycho the version must agree
across the aggregator pom, `Bundle-Version`, and `feature.xml`; the
standard tool is `mvn tycho-versions:set-version -DnewVersion=<train
version>`, run by the release job the way the sed runs today, followed
by a lockstep check that all three carry the same `0.5.x`.

The feature and update-site modules are new and small. `feature.xml`:

```xml
<feature id="io.github.upyesp.epher.eclipse.feature"
         label="epher"
         version="0.5.57.qualifier"
         provider-name="epher">
  <description>epher language support for the Eclipse IDE.</description>
  <plugin id="io.github.upyesp.epher.eclipse" version="0.0.0" unpack="false"/>
</feature>
```

`category.xml` (registration only; no images in categories):

```xml
<site>
  <feature id="io.github.upyesp.epher.eclipse.feature" version="0.0.0">
    <category name="epher"/>
  </feature>
  <category-def name="epher" label="epher"/>
</site>
```

The update-site pom uses `<packaging>eclipse-repository</packaging>`.
`mvn -f clients/eclipse/pom.xml clean verify` produces the repository
at `clients/eclipse/io.github.upyesp.epher.eclipse.updatesite/target/repository/`
(p2 metadata plus `features/` and `plugins/`), which is the update
site. Tycho emits modern p2 metadata rather than the legacy `site.xml`;
the Marketplace Feature-ID field takes the feature id above (p2
installable units add `.feature.group`; confirm the exact string that
resolves in the Marketplace's own validation at listing time).

### Publishing the update site

1. Publish `target/repository/` to the gh-pages branch under
   `eclipse/updates/`, mirroring the apt/rpm publish already in
   `release.yml` (it clones gh-pages, rsyncs the trees, commits, and
   pushes with the repo `GITHUB_TOKEN`; no new secret).
2. Add `eclipse` to the `git archive FETCH_HEAD apt rpm` step in
   `site-build.yml`, so the deployed site serves it at the stable URL
   `https://epher.org/eclipse/updates/`. That URL is what the listing
   points at, so it must not move between trains. A sibling Pages
   repository (the Subclipse pattern) is the alternative; then the URL
   is that repository's Pages URL instead.
3. Smoke-test every train before the listing trusts it:
   - `curl -fsS https://epher.org/eclipse/updates/content.jar -o /dev/null`
   - a headless director install against the published URL:
     `eclipse -nosplash -application org.eclipse.equinox.p2.director -repository https://epher.org/eclipse/updates/ -installIU io.github.upyesp.epher.eclipse.feature.feature.group -destination /tmp/epher-p2-smoke -profile SDKProfile`
   - attach the update-site zip to the GitHub release as well.
4. The marketplace needs no notice when the feature version advances;
   p2 delivers the update. The existing `epher-eclipse.jar` dropins
   artifact stays for users and scripts that already use it.

## Human-only steps (marketplace.eclipse.org)

Everything here needs a browser session with a Foundation account; the
public REST API is read-only (plain GETs, no token, and no write
endpoint exists), so CI cannot create or edit a listing.

1. **Eclipse Foundation account**:
   `https://accounts.eclipse.org/user/register`. The Marketplace has
   no separate login; its Log in link redirects to the Foundation's
   Keycloak, so one account covers both.
2. **Create the listing**: logged in at marketplace.eclipse.org,
   Add Content, "Add a new Solutions Listing"
   (`/node/add/resource`; the form is only visible logged in).
3. **Fill the fields** (values to paste from `DESCRIPTION.md` and this
   file):

   | Field | Value |
   | --- | --- |
   | Name | epher |
   | Short description | the one sentence in `DESCRIPTION.md` (190 characters) |
   | Body | the HTML block in `DESCRIPTION.md` |
   | Categories | Editor; Programming Languages (1 to 5 honest ones) |
   | Tags | epher, calculator, language, lsp, syntax |
   | License | MIT |
   | Development Status | Production/Stable |
   | Platform Support | Windows, Mac, Linux/GTK |
   | Eclipse Versions | the platform releases verified for this train; update by hand, most listings carry a wide range |
   | Update Site URL | `https://epher.org/eclipse/updates/` once the p2 site is live |
   | Feature IDs | `io.github.upyesp.epher.eclipse.feature` (the Feature-ID how-to reads these from the update site; confirm against the generated metadata) |
   | Support URL | `https://github.com/upyesp/epher/issues` |
   | Homepage URL | `https://epher.org` |
   | Company/Organization | upyesp |
   | Logo | `icon.png` (256x256 RGBA, served at 80x80) |
   | Screenshots | the gallery files in `clients/eclipse/images/` |

4. **Submit into moderation**. Listings should appear within 24
   business hours. The criteria: related to Eclipse technologies, in
   English, working links, and installable solutions that work as
   described. Contact `marketplace@eclipse-foundation.org` for
   problems.
5. **Keep the account**. Edits happen at My Marketplace; the per-train
   edit (the displayed version string, Eclipse Versions, screenshots)
   is optional because p2 keeps updates flowing, but a listing that
   advertises an older version erodes trust, so budget the two-minute
   edit into the release checklist.

## Secrets

None, and none are possible. The Marketplace API the MPC client uses
is read-only: every endpoint is an unauthenticated GET, there is no
create or update route, and listing changes are session-login web
forms. Publishing the p2 site uses the repository's `GITHUB_TOKEN`
through the existing gh-pages path, so no entry belongs in the
`stores` environment.

## Order of operations

1. Implement the Tycho feature and update site (the open task above).
2. Publish the site and smoke-test the director install; make sure
   `site-build.yml` serves it.
3. Then create the listing with the real Update Site URL and Feature
   IDs, and pass moderation. A listing created earlier has no working
   Install button, which is the one thing the channel exists for.
