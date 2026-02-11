# AeroSpace window manager configuration
# https://nikitabobko.github.io/AeroSpace/
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/aerospace/"],
    files = [
      {
        source = "aerospace.toml",
        config = {
          # Reference: https://github.com/i3/i3/blob/next/etc/config

          enable-normalization-flatten-containers = false,
          enable-normalization-opposite-orientation-for-nested-containers = false,

          workspace-to-monitor-force-assignment = {
            # FIXME: not work with multiple monitors in same name
            # 1 = 'DELL P2723QE (2)'
            # 2 = 'DELL P2723QE (1)'
            "1" = 1,
            "2" = 3,
            "3" = "built-in",
          },

          mode = {
            main = {
              binding = {
                alt-enter = "exec-and-forget /Applications/kitty.app/Contents/MacOS/kitty --single-instance ~",

                alt-n = "focus left",
                alt-e = "focus down",
                alt-i = "focus up",
                alt-o = "focus right",

                alt-shift-n = "move left",
                alt-shift-e = "move down",
                alt-shift-i = "move up",
                alt-shift-o = "move right",

                alt-h = "split horizontal",
                alt-v = "split vertical",

                alt-f = "fullscreen",

                alt-s = "layout v_accordion", # 'layout stacking' in i3
                alt-t = "layout h_accordion", # 'layout tabbed' in i3
                alt-p = "layout tiles horizontal vertical", # 'layout toggle split' in i3

                alt-shift-space = "layout floating tiling", # 'floating toggle' in i3

                alt-1 = "workspace 1",
                alt-2 = "workspace 2",
                alt-3 = "workspace 3",
                alt-4 = "workspace 4",
                alt-5 = "workspace 5",
                alt-6 = "workspace 6",
                alt-7 = "workspace 7",
                alt-8 = "workspace 8",
                alt-9 = "workspace 9",
                alt-0 = "workspace 10",

                alt-shift-1 = "move-node-to-workspace 1",
                alt-shift-2 = "move-node-to-workspace 2",
                alt-shift-3 = "move-node-to-workspace 3",
                alt-shift-4 = "move-node-to-workspace 4",
                alt-shift-5 = "move-node-to-workspace 5",
                alt-shift-6 = "move-node-to-workspace 6",
                alt-shift-7 = "move-node-to-workspace 7",
                alt-shift-8 = "move-node-to-workspace 8",
                alt-shift-9 = "move-node-to-workspace 9",
                alt-shift-0 = "move-node-to-workspace 10",

                alt-shift-c = "reload-config",

                alt-r = "mode resize",
              },
            },
            resize = {
              binding = {
                n = "resize width -50",
                e = "resize height +50",
                i = "resize height -50",
                o = "resize width +50",
                enter = "mode main",
                esc = "mode main",
              },
            },
          },

          on-window-detected = [
            {
              "if".app-id = "com.todoist.mac.Todoist",
              run = ["layout floating", "move-node-to-workspace 3"],
            },
            {
              "if".app-id = "com.apple.MobileSMS",
              run = ["move-node-to-workspace 3"],
            },
            {
              "if".app-id = "com.apple.Music",
              run = ["move-node-to-workspace 3"],
            },
            {
              "if".app-id = "com.apple.AppStore",
              run = ["layout floating", "move-node-to-workspace 3"],
            },
            {
              "if".app-id = "io.zsa.keymapp",
              run = ["layout floating"],
            },
            {
              "if".window-title-regex-substring = "^Playwright Inspector$",
              run = ["move-node-to-workspace 3"],
            },
          ],
        },
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
