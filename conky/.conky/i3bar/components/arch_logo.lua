local util = require "util"

-- an Arch Linux logo <(=*/ω＼*=)>
return function (opt)
    util.draw_svg({cr = opt.cr,
              x = opt.x, y = opt.y,
              h = 20, w = 20,
              file = opt.RESOURCE_PATH .. "arch-logo.svg"})
end
