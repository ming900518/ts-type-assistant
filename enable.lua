function start_lsp()
    print("Hi")
    require("lsp-debug-tools").restart({excepted = {} , name = "ts-type-assist", cmd = {"target/release/ts-type-assistant"}, root_dir = vim.loop.cwd()})
end

start_lsp()
