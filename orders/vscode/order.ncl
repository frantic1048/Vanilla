# VSCode settings configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/Library/Application Support/Code/User/"],
    files = [
      {
        source = "settings.json",
        format = "json",
        # Dynamic settings that may change locally
        ignore = [
          "window.zoomLevel",
          "workbench.colorTheme",
          "workbench.iconTheme",
          "playwright.env",
        ],
        config = {
          # Editor settings
          "editor.fontSize" = 13,
          "editor.fontFamily" = "JetBrains Mono, Monaspace Argon Var, Fantasque Sans Mono, monospace, Electroharmonix",
          "editor.fontLigatures" = "'calt', 'zero', 'ss02', 'cv14'",
          "editor.formatOnSave" = true,
          "editor.formatOnType" = false,
          "editor.tabSize" = 2,
          "editor.wordWrap" = "off",
          "editor.lineNumbers" = "on",
          "editor.minimap.enabled" = false,
          "editor.minimap.renderCharacters" = false,
          "editor.mouseWheelZoom" = true,
          "editor.multiCursorModifier" = "ctrlCmd",
          "editor.cursorBlinking" = "phase",
          "editor.accessibilitySupport" = "off",
          "editor.bracketPairColorization.independentColorPoolPerBracketType" = true,
          "editor.guides.bracketPairs" = true,
          "editor.guides.bracketPairsHorizontal" = true,
          "editor.guides.highlightActiveIndentation" = "always",
          "editor.inlineSuggest.enabled" = true,
          "editor.linkedEditing" = true,
          "editor.renderLineHighlight" = "gutter",
          "editor.renderLineHighlightOnlyWhenFocus" = false,
          "editor.renderWhitespace" = "all",
          "editor.semanticHighlighting.enabled" = true,
          "editor.showFoldingControls" = "mouseover",
          "editor.unicodeHighlight.invisibleCharacters" = true,

          # Terminal
          "terminal.integrated.fontFamily" = "JetBrainsMono NF",
          "terminal.integrated.stickyScroll.enabled" = true,

          # Window
          "window.menuBarVisibility" = "hidden",
          "window.titleBarStyle" = "native",
          "window.customTitleBarVisibility" = "never",
          "window.commandCenter" = false,

          # Workbench
          "workbench.activityBar.location" = "hidden",
          "workbench.editor.empty.hint" = "hidden",
          "workbench.localHistory.enabled" = true,
          "workbench.localHistory.maxFileEntries" = 500,
          "workbench.localHistory.maxFileSize" = 512,
          "workbench.sideBar.location" = "left",
          "workbench.layoutControl.enabled" = false,
          "workbench.experimental.share.enabled" = true,
          "workbench.colorCustomizations" = {
            "activityBar.background" = "#0000006a",
            "breadcrumb.background" = "#00000000",
            "breadcrumbPicker.background" = "#000000a0",
            "button.background" = "#00000040",
            "button.foreground" = "#eee",
            "button.hoverBackground" = "#00000080",
            "button.secondaryForeground" = "#eee",
            "editor.background" = "#00000030",
            "editor.lineHighlightBackground" = "#707effa0",
            "editor.lineHighlightBorder" = "#c4c6d3a0",
            "editorCursor.foreground" = "#80c1ff",
            "editorError.background" = "#eb178149",
            "editorError.foreground" = "#f9117ddb",
            "editorGroup.border" = "#51abff1a",
            "editorGroupHeader.tabsBackground" = "#00000050",
            "editorGutter.background" = "#00000030",
            "editorLineNumber.activeForeground" = "#FFF",
            "editorLineNumber.foreground" = "#aaa",
            "editorStickyScroll.background" = "#2c2868c5",
            "editorStickyScroll.border" = "#6331b340",
            "editorWarning.background" = "#ebcb1745",
            "editorWarning.foreground" = "#f9c311db",
            "extensionButton.prominentForeground" = "#eee",
            "input.background" = "#000000a0",
            "input.border" = "#51abff",
            "list.activeSelectionBackground" = "#000000aa",
            "list.hoverBackground" = "#00000040",
            "list.inactiveSelectionBackground" = "#00000060",
            "panel.background" = "#00000080",
            "panel.border" = "#00000000",
            "sideBar.background" = "#1111119a",
            "sideBarSectionHeader.background" = "#0000",
            "statusBar.background" = "#00000080",
            "statusBar.noFolderBackground" = "#00000080",
            "tab.activeBackground" = "#00000050",
            "tab.border" = "#00000000",
            "tab.inactiveBackground" = "#00000010",
            "titleBar.activeBackground" = "#00000030",
            "window.background" = "#00000080",
          },

          # Git
          "git.autorefresh" = true,
          "git.autoRepositoryDetection" = true,
          "git.blame.editorDecoration.enabled" = true,

          # GitHub Copilot
          "github.copilot.editor.enableAutoCompletions" = true,
          "github.copilot.nextEditSuggestions.enabled" = true,
          "github.copilot.enable" = {
            "*" = true,
            jsonnet = true,
            markdown = true,
            plaintext = true,
            properties = true,
            scminput = false,
            shellscript = true,
            typescript = true,
            typescriptreact = true,
            yaml = true,
          },

          # TypeScript
          "typescript.preferences.importModuleSpecifier" = "non-relative",
          "typescript.preferences.includePackageJsonAutoImports" = "on",
          "typescript.updateImportsOnFileMove.enabled" = "always",
          "typescript.validate.enable" = true,
          "typescript.tsserver.maxTsServerMemory" = 16384,
          "typescript.tsserver.experimental.useVsCodeWatcher" = false,
          "typescript.tsserver.experimental.enableProjectDiagnostics" = false,
          "typescript.experimental.useTsgo" = false,

          # JavaScript
          "javascript.preferences.importModuleSpecifier" = "non-relative",
          "javascript.validate.enable" = true,

          # ESLint
          "eslint.execArgv" = ["--max-old-space-size=16384"],
          "eslint.lintTask.enable" = true,
          "eslint.runtime" = "node",
          "eslint.useFlatConfig" = true,

          # Diff
          "diffEditor.maxComputationTime" = 0,
          "diffEditor.wordWrap" = "off",

          # Problems
          "problems.sortOrder" = "position",

          # Update
          "update.mode" = "start",

          # Zen mode
          "zenMode.hideLineNumbers" = false,
          "zenMode.hideStatusBar" = false,

          # Chat
          "chat.commandCenter.enabled" = false,
          "claudeCode.preferredLocation" = "sidebar",

          # File associations
          "files.associations" = {
            ".git-blame-ignore-revs" = "ignore",
            ".prototools" = "toml",
            "*.bu" = "yaml",
            "*.ign" = "json",
            "*.sarif" = "json",
            "*.sast" = "json",
            Brewfile = "ruby",
            "Brewfile.{app,desktop,fonts,util,makedep}" = "ruby",
            Dangerfile = "ruby",
            skhdrc = "shellscript",
            yabairc = "shellscript",
          },

          # Cursor
          "cursor.cpp.disabledLanguages" = ["plaintext"],

          # Debug
          "debug.disassemblyView.showSourceCode" = false,

          # Telemetry
          "redhat.telemetry.enabled" = false,

          # Even Better TOML
          "evenBetterToml.schema.associations" = {
            "vector.toml" = "https://www.schemastore.org/vector.json",
          },

          # Graphviz
          "graphviz-interactive-preview.renderLockAdditionalTimeout" = 5000,

          # Jsonnet
          "jsonnet.languageServer.lint" = true,

          # Playwright
          "playwright.reuseBrowser" = false,

          # Settings Sync
          "settingsSync.ignoredExtensions" = [
            "oderwat.indent-rainbow",
            "leodevbro.blockman",
          ],

          # OXC
          "oxc.typeAware" = true,

          # GitLens
          "gitlens.advanced.blame.customArguments" = [],
          "gitlens.codeLens.enabled" = false,
          "gitlens.currentLine.enabled" = false,
          "gitlens.currentLine.scrollable" = false,
          "gitlens.hovers.currentLine.over" = "line",
          "gitlens.plusFeatures.enabled" = false,

          # Todo Highlight
          "todohighlight.keywords" = [
            { text = "FIXME:", backgroundColor = "#e85854", color = "#fefefe" },
            { text = "TODO:", backgroundColor = "#57b031", color = "#fefefe" },
            { text = "MEMO:", backgroundColor = "#0597f2", color = "#fefefe" },
            { text = "WARN:", backgroundColor = "#dbac12", color = "#fefefe" },
          ],

          # SemanticDiff
          "semanticdiff.closeOriginalTab" = true,
          "semanticdiff.defaultDiffViewer" = true,

          # YAML
          "yaml.customTags" = ["!reference sequence"],

          # Tinymist
          "tinymist.formatterMode" = "typstyle",

          # Language-specific settings
          "[css]" = { "editor.defaultFormatter" = "esbenp.prettier-vscode" },
          "[dockercompose]" = {
            "editor.autoIndent" = "advanced",
            "editor.defaultFormatter" = "redhat.vscode-yaml",
            "editor.insertSpaces" = true,
            "editor.tabSize" = 2,
          },
          "[github-actions-workflow]" = { "editor.defaultFormatter" = "redhat.vscode-yaml" },
          "[ignore]" = { "editor.defaultFormatter" = "foxundermoon.shell-format" },
          "[javascript]" = {
            "editor.codeActionsOnSave" = { "source.fixAll.eslint" = "explicit" },
            "editor.defaultFormatter" = "esbenp.prettier-vscode",
          },
          "[json]" = { "editor.defaultFormatter" = "denoland.vscode-deno" },
          "[jsonc]" = { "editor.defaultFormatter" = "esbenp.prettier-vscode" },
          "[markdown]" = { "editor.defaultFormatter" = "esbenp.prettier-vscode" },
          "[nushell]" = { "editor.defaultFormatter" = "TheNuProjectContributors.vscode-nushell-lang" },
          "[python]" = { "editor.formatOnType" = true },
          "[shellscript]" = { "editor.defaultFormatter" = "foxundermoon.shell-format" },
          "[typescript]" = {
            "editor.codeActionsOnSave" = { "source.fixAll.eslint" = "explicit" },
            "editor.defaultFormatter" = "denoland.vscode-deno",
          },
          "[typescriptreact]" = {
            "editor.codeActionsOnSave" = { "source.fixAll.eslint" = "explicit" },
            "editor.defaultFormatter" = "esbenp.prettier-vscode",
            "editor.stickyScroll.defaultModel" = "foldingProviderModel",
          },
          "[typst]" = { "editor.defaultFormatter" = "myriad-dreamin.tinymist" },
          "[yaml]" = { "editor.defaultFormatter" = "esbenp.prettier-vscode" },
        },
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
