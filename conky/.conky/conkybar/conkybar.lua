--this is a lua script for use in conky
require 'cairo'
require 'imlib2'
require 'rsvg'
json = require 'json'

----------------------------------------------------------------------------
--    _     _ _______ _____        _____ _______ _____ _______ _______    --
--    |     |    |      |   |        |      |      |   |______ |______    --
--    |_____|    |    __|__ |_____ __|__    |    __|__ |______ ______|    --
----------------------------------------------------------------------------

-- keep cairo transformation matrix after execute cb()
-- cr: cairo_t, created by cairo_create()
-- cb: function, callback
function keep_mat(cr, cb)
    -- store current transformation matrix
    local mat = cairo_matrix_t.create()
    cairo_get_matrix(cr,mat)

    -- execute callback function
    cb()

    -- restore transformation matrix
    cairo_set_matrix(cr, mat)
    mat = nil
end -- function keep_mat

-- skew current transformation matrix(CTM)
-- in cr by T on X axis
-- supplement to cairo's transform
function skewX(cr, T)
    local mat = cairo_matrix_t.create()

    cairo_matrix_init(mat,
      1, 0, T,
      1, 0, 0
    )

    cairo_transform(cr, mat)
    mat = nil
end -- function skewX

-- skew current transformation matrix(CTM)
-- in cr by T on Y axis
-- supplement to cairo's transform
function skewY(cr, T)
    local mat = cairo_matrix_t.create()

    cairo_matrix_init(mat,
      1, T, 0,
      1, 0, 0
    )

    cairo_transform(cr, mat)
    mat = nil
end -- function skewY

-- Draw SVG function
-- Usage:
-- draw_svg({x=0,y=0,h=20,w=20,file="/path/to/awesome.svg"})
function draw_svg(im)
    local x, y, w, h = nil, nil, nil, nil
    local file = nil
    x = (im.x or 0)
    y = (im.y or 0)
    w = (im.w or 0)
    h = (im.h or 0)
    file = tostring(im.file)
    if file ==  nil then print("need svg file") end
    -----------
    local handle = rsvg_handle_new_from_file(file)

    keep_mat(im.cr, function ()
        local dimensions = RsvgDimensionData.create()
        rsvg_handle_get_dimensions(handle, dimensions)
        local x_factor = w / dimensions['width']
        local y_factor = h / dimensions['height']
        local scale_factor = math.min(x_factor, y_factor)

        if scale_factor == 0 then scale_factor = 1 end

        cairo_translate(im.cr, x, y) -- translate bofore scale!
        cairo_scale(im.cr, scale_factor, scale_factor)
        rsvg_handle_render_cairo(handle, im.cr)
    end)

    rsvg_destroy_handle(handle)
    handle = nil
    dimensions = nil
end -- function draw_svg

-- Draw raster image function
-- https://github.com/brndnmtthws/conky/wiki/Using-Lua-scripts-in-conky:-Useful-functions-and-code#image-display-function
-- usage:
-- image({x=100,y=100,h=50,w=50,file="/home/username/cute_puppy.png"})
function draw_raster(im)
    local x, y, w, h = nil, nil, nil, nil
    local file = nil
    x = (im.x or 0)
    y = (im.y or 0)
    w = (im.w or 0)
    h = (im.h or 0)
    file = tostring(im.file)
    if file ==  nil then print("need svg file") end
    ---------------------------------------------
    local show = imlib_load_image(file)
    if show == nil then return end
    local width, height = 0, 0
    imlib_context_set_image(show)
    if tonumber(w) == 0 then
        width = imlib_image_get_width()
    else
        width = tonumber(w)
    end
    if tonumber(h) == 0 then
        height = imlib_image_get_height()
    else
        height = tonumber(h)
    end
    imlib_context_set_image(show)
    local scaled = imlib_create_cropped_scaled_image(
        0, 0,
        imlib_image_get_width(), imlib_image_get_height(),
        width, height)
    imlib_free_image()
    imlib_context_set_image(scaled)
    imlib_render_image_on_drawable(x, y)
    imlib_free_image()
    show = nil
end -- function draw_raster

-----------------------------------------------------------------------------------------
--    _______  _____  _______  _____   _____  __   _ _______ __   _ _______ _______    --
--    |       |     | |  |  | |_____] |     | | \  | |______ | \  |    |    |______    --
--    |_____  |_____| |  |  | |       |_____| |  \_| |______ |  \_|    |    ______|    --
-----------------------------------------------------------------------------------------
-- all components are functions named with prefix `conkybar_`

-- an Arch Linux logo <(=*/ω＼*=)>
function conkybar_arch_logo(opt)
    draw_svg({cr = opt.cr,
              x = opt.x, y = opt.y,
              h = 20, w = 20,
              file = opt.RESOURCE_PATH .. "arch-logo.svg"})
end -- function conkybar_arch_logo

-- i3-wm workspace indicator
-- for i3 wm configured with 10 total workspaces (i3's default)
function conkybar_i3_workspace_indicator(opt)
    local xpos = opt.x
    local ypos = opt.y

    -- text color
    local r, g, b, a = 1, 1, 1, 1

    -- fetch i3 wm workspace information
    local workspacesData = conky_parse('${exec i3-msg -t get_workspaces}')
    local new_workspaces = json.decode(workspacesData) or {}
    local workspaces = {}
    local present_workspace_number = 0

    for i = 1, 10 do
        workspaces[i] = nil
    end

    for i, w in ipairs(new_workspaces) do
        workspaces[w['num']] = {
            ['num'] = w['num'],
            ['visible'] = w['visible']
        }
    end

    -- draw small text 'workspace'
    ypos = ypos + 14
    r, g, b, a = 0.9, 0.9, 0.9, 0.9
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, 9)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, 'workspace')
    cairo_stroke(opt.cr)

    xpos = xpos + 48
    ypos = ypos - 14
    draw_svg({cr = opt.cr,
        x = xpos, y = ypos,
        file = opt.RESOURCE_PATH .. "workspace-frame.svg"})

    xpos = xpos + 34
    -- upper indicator
    for i = 1,5 do
        -- shift right
        xpos = xpos + 9
        if workspaces[i] == nil then
            -- empty workspace
            draw_svg({cr = opt.cr,
            x = xpos, y = ypos,
            file = opt.RESOURCE_PATH .. "workspace-upper_empty.svg"})
        else
            if workspaces[i]['urgent'] == true then
                -- urgent
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-upper_urgent.svg"})
            elseif workspaces[i]['visible'] == true then
                -- present
                present_workspace_number = i
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-upper_present.svg"})
            else
                -- normal
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-upper_normal.svg"})
            end
        end
    end

    xpos = xpos - 46
    ypos = ypos + 14
    -- lower indicator
    for i = 6,10 do
        xpos = xpos + 9
        if workspaces[i] == nil then
            -- empty workspace
            draw_svg({cr = opt.cr,
            x = xpos, y = ypos,
            file = opt.RESOURCE_PATH .. "workspace-lower_empty.svg"})
        else
            if workspaces[i]['urgent'] == true then
                -- urgent
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-lower_urgent.svg"})
            elseif workspaces[i]['visible'] == true then
                -- present
                present_workspace_number = i
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-lower_present.svg"})
            else
                -- normal
                draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-lower_normal.svg"})
            end
        end
    end

    xpos = xpos - 53
    ypos = ypos + 3

    -- display workspace 10 as workspace 0
    if present_workspace_number == 10 then
      present_workspace_number = 0
    end

    r, g, b, a = 1, 1, 1, 1
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, opt.primary_font_size)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, present_workspace_number)
    cairo_stroke(opt.cr)

end -- function conkybar_i3_workspace_indicator

-- show system load as bars
function conkybar_sys_load(opt)
    local xpos = opt.x
    local ypos = opt.y

    local r, g, b, a = 1, 1, 1, 1
    local bar_width = 160
    local bar_height = 5
    local bar_skewX  = 0.75

    -- fetch system load data
    local cpu_percent = conky_parse('${cpu cpu0}')
    local mem_percent = conky_parse('${memperc}')
    local mem_used = conky_parse('${mem}')
    local mem_total = conky_parse('${memmax}')

    -- draw small text 'system load'
    ypos = ypos + 16
    r, g, b, a = 0.9, 0.9, 0.9, 0.9
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, 9)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, 'Sys load')
    cairo_stroke(opt.cr)

    xpos = xpos + 44
    ypos = ypos - 16
    draw_svg({cr = opt.cr,
        x = xpos, y = ypos,
        file = opt.RESOURCE_PATH .. "cpu-load-frame.svg"})

    -- bars
    xpos = xpos + 9
    ypos = ypos + 7
    r, g, b, a = 0.9, 0.9, 0.9, 0.6
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_set_source_rgba(opt.cr, r, g, b, a)

    keep_mat(opt.cr, function()
        skewX(opt.cr, bar_skewX)
        cairo_rectangle(opt.cr, xpos, ypos, bar_width * cpu_percent * 0.01, bar_height)
        cairo_fill(opt.cr)
    end)

    xpos = xpos - 1
    ypos = ypos + 10
    r, g, b, a = 0.9, 0.9, 0.9, 0.6
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_set_source_rgba(opt.cr, r, g, b, a)

    keep_mat(opt.cr, function()
        skewX(opt.cr, bar_skewX)
        cairo_rectangle(opt.cr, xpos, ypos, bar_width * mem_percent * 0.01, bar_height)
        cairo_fill(opt.cr)
    end)

    -- bar text
    xpos = xpos + 175
    ypos = ypos - 6
    r, g, b, a = 0.9, 0.9, 0.9, 0.9
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, 9)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, cpu_percent .. '%')
    cairo_stroke(opt.cr)

    xpos = xpos + 6
    ypos = ypos + 11
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_show_text(opt.cr, mem_used .. '/' .. mem_total)
    cairo_stroke(opt.cr)
end -- function conkybar_sys_load

-- show NVIDIA GPU load as bars
function conkybar_gpu_load(opt)
    local xpos = opt.x
    local ypos = opt.y

    local r, g, b, a = 1, 1, 1, 1
    local bar_width = 160
    local bar_height = 5
    local bar_skewX  = 0.75

    -- fetch gpu load data
    -- command found here:
    -- https://github.com/brndnmtthws/conky/blob/e84ca1f966b8c2903cd792914f5e0ee6d3181b68/doc/variables.xml#L2781
    local gpu_percent = conky_parse('${nvidia gpuutil}')
    local mem_percent = conky_parse('${nvidia memutil}')
    local mem_used = conky_parse('${nvidia memused}MiB')
    local mem_total = conky_parse('${nvidia memmax}MiB')

    -- draw small text 'GPU load'
    ypos = ypos + 16
    r, g, b, a = 0.9, 0.9, 0.9, 0.9
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, 9)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, 'GPU load')
    cairo_stroke(opt.cr)

    xpos = xpos + 44
    ypos = ypos - 16
    draw_svg({cr = opt.cr,
        x = xpos, y = ypos,
        file = opt.RESOURCE_PATH .. "cpu-load-frame.svg"})

    -- bars
    xpos = xpos + 9
    ypos = ypos + 7
    r, g, b, a = 0.9, 0.9, 0.9, 0.6
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_set_source_rgba(opt.cr, r, g, b, a)

    keep_mat(opt.cr, function()
        skewX(opt.cr, bar_skewX)
        cairo_rectangle(opt.cr, xpos, ypos, bar_width * gpu_percent * 0.01, bar_height)
        cairo_fill(opt.cr)
    end)

    xpos = xpos - 1
    ypos = ypos + 10
    r, g, b, a = 0.9, 0.9, 0.9, 0.6
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_set_source_rgba(opt.cr, r, g, b, a)

    keep_mat(opt.cr, function()
        skewX(opt.cr, bar_skewX)
        cairo_rectangle(opt.cr, xpos, ypos, bar_width * mem_percent * 0.01, bar_height)
        cairo_fill(opt.cr)
    end)

    -- bar text
    xpos = xpos + 175
    ypos = ypos - 6
    r, g, b, a = 0.9, 0.9, 0.9, 0.9
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_select_font_face(
        opt.cr,
        opt.primary_font,
        opt.primary_font_slant,
        opt.primary_font_face)
    cairo_set_font_size(opt.cr, 9)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, gpu_percent .. '%')
    cairo_stroke(opt.cr)

    xpos = xpos + 6
    ypos = ypos + 11
    cairo_move_to(opt.cr, xpos, ypos)
    cairo_show_text(opt.cr, mem_used .. '/' .. mem_total)
    cairo_stroke(opt.cr)
end -- function conkybar_gpu_load

-- simple text date time
function conkybar_date_time(opt)
    local r, g, b, a = 1, 1, 1, 1
    local text = conky_parse('${time %a, %d %b %Y %T %z}')
    cairo_move_to(opt.cr, opt.x, opt.y)
    cairo_select_font_face(opt.cr, opt.primary_font, opt.primary_font_slant, opt.primary_font_face)
    cairo_set_font_size(opt.cr, opt.primary_font_size - 3)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, text)
    cairo_stroke(opt.cr)
end -- function conkybar_date_time

-- displays what Clementine is playing.
-- draw nothiong if Clementine is not running
function conkybar_clementine_play(opt)
    local r, g, b, a = 1, 1, 1, 1
    local text = conky_parse([[
          ${if_running clementine}
          ${if_empty ${exec /home/chino/bin/wclementineplaying.py -a}}
          ${else}${exec /home/chino/bin/wclementineplaying.py -a} - ${exec /home/chino/bin/wclementineplaying.py -t}
          ${endif}
          ${endif}
    ]])
    cairo_move_to(opt.cr, opt.x, opt.y)
    cairo_select_font_face(opt.cr, 'Source Han Sans SC', opt.primary_font_slant, opt.primary_font_face)
    cairo_set_font_size(opt.cr, opt.primary_font_size - 3)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, text)
    cairo_stroke(opt.cr)
end -- function conkybar_clementine_play


--------------------------------------------------------------------------
--    _______  _____  __   _ _     _ __   __ ______  _______  ______    --
--    |       |     | | \  | |____/    \_/   |_____] |_____| |_____/    --
--    |_____  |_____| |  \_| |    \_    |    |_____] |     | |    \_    --
--------------------------------------------------------------------------
function conky_conkybar()
    local RESOURCE_PATH = debug.getinfo(1).source:match("@?(.*/)") .. 'resource/'
    if conky_window == nil then
        return
    end
    local cs = cairo_xlib_surface_create(
        conky_window.display,
        conky_window.drawable,
        conky_window.visual,
        conky_window.width,
        conky_window.height)
    local cr = cairo_create(cs)
    local updates = tonumber(conky_parse('${updates}'))
    local primary_font = 'Fira Code'
    local primary_font_size = 16
    local primary_font_slant = CAIRO_FONT_SLANT_NORMAL
    local primary_font_face = CAIRO_FONT_WEIGHT_NORMAL

    local primary_font_options = cairo_font_options_create()
    cairo_font_options_set_antialias(primary_font_options, CAIRO_ANTIALIAS_SUBPIXEL)
    cairo_font_options_set_subpixel_order(primary_font_options, CAIRO_SUBPIXEL_ORDER_RGB)
    cairo_font_options_set_hint_style(primary_font_options, CAIRO_HINT_STYLE_FULL)
    cairo_font_options_set_hint_metrics(primary_font_options, CAIRO_HINT_METRICS_DEFAULT)
    cairo_set_font_options(cr, primary_font_options)

    function draw_component(component_func, pos)
      -- pass essential variables to component_func
      return component_func{
        RESOURCE_PATH = RESOURCE_PATH,
        cr = cr,
        cs = cs,
        primary_font = primary_font,
        primary_font_size = primary_font_size,
        primary_font_slant = primary_font_slant,
        primary_font_face = primary_font_face,
        x = pos.x,
        y = pos.y
      }
    end -- function draw_component

    if updates>3 then -- start drawing
        local red,green,blue,alpha = 1, 1, 1, 1
        local xpos, ypos = 0, 0
        local text = ''

        draw_component(conkybar_arch_logo, {x = 3, y = 5})
        draw_component(conkybar_i3_workspace_indicator, {x = 48, y = 2})
        draw_component(conkybar_sys_load, {x = 216, y = 0})
        draw_component(conkybar_gpu_load, {x = 549, y = 0})
        draw_component(conkybar_date_time, {x = 876, y = 19})
        draw_component(conkybar_clementine_play, {x = 1300, y = 18})
    end
    cairo_font_options_destroy(primary_font_options)
    cairo_destroy(cr)
    cairo_surface_destroy(cs)
    cr = nil
end -- function conky_conkybar
