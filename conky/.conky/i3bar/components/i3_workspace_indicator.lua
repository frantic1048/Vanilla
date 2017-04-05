local json = require 'json'

local i3bar_util = require "util"

-- i3-wm workspace indicator
-- for i3 wm configured with 10 total workspaces (i3's default)
return function (opt)
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
    i3bar_util.draw_svg({cr = opt.cr,
        x = xpos, y = ypos,
        file = opt.RESOURCE_PATH .. "workspace-frame.svg"})

    xpos = xpos + 34
    -- upper indicator
    for i = 1,5 do
        -- shift right
        xpos = xpos + 9
        if workspaces[i] == nil then
            -- empty workspace
            i3bar_util.draw_svg({cr = opt.cr,
            x = xpos, y = ypos,
            file = opt.RESOURCE_PATH .. "workspace-upper_empty.svg"})
        else
            if workspaces[i]['urgent'] == true then
                -- urgent
                i3bar_util.draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-upper_urgent.svg"})
            elseif workspaces[i]['visible'] == true then
                -- present
                present_workspace_number = i
                i3bar_util.draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-upper_present.svg"})
            else
                -- normal
                i3bar_util.draw_svg({cr = opt.cr,
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
            i3bar_util.draw_svg({cr = opt.cr,
            x = xpos, y = ypos,
            file = opt.RESOURCE_PATH .. "workspace-lower_empty.svg"})
        else
            if workspaces[i]['urgent'] == true then
                -- urgent
                i3bar_util.draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-lower_urgent.svg"})
            elseif workspaces[i]['visible'] == true then
                -- present
                present_workspace_number = i
                i3bar_util.draw_svg({cr = opt.cr,
                x = xpos, y = ypos,
                file = opt.RESOURCE_PATH .. "workspace-lower_present.svg"})
            else
                -- normal
                i3bar_util.draw_svg({cr = opt.cr,
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

end
