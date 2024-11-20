return {
    {
      "nvim-treesitter/nvim-treesitter",
      build = function()
          require("nvim-treesitter.install").update({ with_sync = true })()
      end,
    },
    {
      "nvim-treesitter/nvim-treesitter-textobjects",
    },
    {
        "kylechui/nvim-surround",
        version = "*", -- Use for stability; omit to use `main` branch for the latest features
        event = "VeryLazy",
        config = function()
            require("nvim-surround").setup({
                -- Configuration here, or leave empty to use defaults
            })
        end
    }
}
