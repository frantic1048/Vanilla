require 'cairo'

-- simple text date time
return function (opt)
    local r, g, b, a = 1, 1, 1, 1
    local text = conky_parse('${time %a, %d %b %Y %T %z}')
    cairo_move_to(opt.cr, opt.x, opt.y)
    cairo_select_font_face(opt.cr, opt.primary_font, opt.primary_font_slant, opt.primary_font_face)
    cairo_set_font_size(opt.cr, opt.primary_font_size - 3)
    cairo_set_source_rgba(opt.cr, r, g, b, a)
    cairo_show_text(opt.cr, text)
    cairo_stroke(opt.cr)
end
