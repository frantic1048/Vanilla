--this is a lua script for use in conky
require 'cairo'
json =  require 'json'

function conky_conkybar()
    if conky_window == nil then
        return
    end
    local cs = cairo_xlib_surface_create(conky_window.display,
                                         conky_window.drawable,
                                         conky_window.visual,
                                         conky_window.width,
                                         conky_window.height)
    cr = cairo_create(cs)
    local updates=tonumber(conky_parse('${updates}'))
    if updates>3 then
        print ('W x H',conky_window.width,conky_window.height)
        local primary_font='Source Han Sans CN'
        local primary_font_size=16
        local primary_font_slant=CAIRO_FONT_SLANT_NORMAL
        local primary_font_face=CAIRO_FONT_WEIGHT_NORMAL

        local red,green,blue,alpha=1,1,1,1
        local xpos,ypos=0,0
        local text=''



        -- workspace indicator
        xpos,ypos=3,20
        red,green,blue,alpha=1,1,1,1
        cairo_move_to(cr,xpos,ypos)
        cairo_select_font_face(cr, primary_font, primary_font_slant, primary_font_face)
        cairo_set_font_size(cr, primary_font_size)
        cairo_set_source_rgba(cr,red,green,blue,alpha)

        local workspaces = json.decode(conky_parse('${exec i3-msg -t get_workspaces}'))
        -- parse ws here
        text=''
        for i,w in ipairs(workspaces) do
          if w['visible'] == true then
            text = text .. ' (' .. w.num .. ') '
          else
            text = text .. ' ' .. w.num .. ' '
          end
        end
        cairo_show_text(cr,text)
        cairo_stroke(cr)



        -- debugging info
        xpos,ypos=400,20
        red,green,blue,alpha=1,1,0,1
        cairo_move_to(cr,xpos,ypos)
        cairo_select_font_face(cr, primary_font, primary_font_slant, primary_font_face)
        cairo_set_font_size(cr, primary_font_size)
        cairo_set_source_rgba(cr,red,green,blue,alpha)

        text='w=' .. conky_window.width .. ',h=' .. conky_window.height
        cairo_show_text(cr,text)
        cairo_stroke(cr)



        -- date time
        xpos,ypos=653,20
        red,green,blue,alpha=1,1,1,1
        cairo_move_to(cr,xpos,ypos)
        cairo_select_font_face(cr, primary_font, primary_font_slant, primary_font_face)
        cairo_set_font_size(cr, primary_font_size)
        cairo_set_source_rgba(cr,red,green,blue,alpha)

        text=conky_parse('${time %a, %d %b %Y %T %z}')
        cairo_show_text(cr,text)
        cairo_stroke(cr)



        -- clementine playing
        xpos,ypos=1300,20
        red,green,blue,alpha=1,1,1,1
        cairo_move_to(cr,xpos,ypos)
        cairo_select_font_face(cr, primary_font, primary_font_slant, primary_font_face)
        cairo_set_font_size(cr, primary_font_size)
        cairo_set_source_rgba(cr,red,green,blue,alpha)
        text=conky_parse([[${if_running clementine}
${if_empty ${exec /home/chino/bin/wclementineplaying.py -a}}
${else} Music: ${exec /home/chino/bin/wclementineplaying.py -a} - ${exec /home/chino/bin/wclementineplaying.py -t}
${endif}
${endif}]])
        cairo_show_text(cr,text)
        cairo_stroke(cr)
    end
    cairo_destroy(cr)
    cairo_surface_destroy(cs)
    cr=nil
end
