----------------------------------------------------------------------------
--    _     _ _______ _____        _____ _______ _____ _______ _______    --
--    |     |    |      |   |        |      |      |   |______ |______    --
--    |_____|    |    __|__ |_____ __|__    |    __|__ |______ ______|    --
----------------------------------------------------------------------------

require 'cairo'
require 'imlib2'
require 'rsvg'



local i3bar_util = {}



-- keep cairo transformation matrix after execute cb()
-- cr: cairo_t, created by cairo_create()
-- cb: function, callback
function i3bar_util.keep_mat(cr, cb)
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
function i3bar_util.skewX(cr, T)
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
function i3bar_util.skewY(cr, T)
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
function i3bar_util.draw_svg(im)
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

    i3bar_util.keep_mat(im.cr, function ()
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
function i3bar_util.draw_raster(im)
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

return i3bar_util
