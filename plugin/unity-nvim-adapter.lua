local log_level = vim.log.levels.INFO -- Change for debugging

local function notif(msg, level)
	if level >= log_level then
		vim.notify(msg, level)
	end
end

local function find_unity_root(start_dir)
	local start = vim.fs.normalize(start_dir or vim.loop.cwd())
	local project_version = vim.fs.find("ProjectSettings/ProjectVersion.txt", { upward = true, path = start })[1]
	if project_version then
		return vim.fs.dirname(vim.fs.dirname(project_version))
	end
end

local function start_named_pipe_server(pipe_path)
	vim.fn.serverstart(pipe_path)
	notif("Unity adapter started on pipe: " .. pipe_path, vim.log.levels.DEBUG)
end

local function is_manual_listen()
	if vim.env.NVIM_LISTEN_ADDRESS then
		return true
	end

	local args = vim.fn.argv()
	---@cast args string[]
	for _, arg in ipairs(args) do
		if arg == "--listen" then
			return true
		end
	end

	return false
end

local function setup()
	-- If we start Neovim with the --listen flag or NVIM_LISTEN_ADDRESS env var we shouldn't try to start another.
	if is_manual_listen() then
		notif("Server is already running at " .. vim.v.servername, vim.log.levels.DEBUG)
		print(vim.env.NVIM_LISTEN_ADDRESS)
		return
	end

	local root = find_unity_root()
	if not root then
		notif("Not a Unity project", vim.log.levels.DEBUG)
		return
	end

	local temp_dir = root .. "/Temp"
	local pipe_dir = temp_dir .. "/unity_adapter_pipe"

	-- Protection for accidentally opening the same project twice.
	-- The servername check above stops this from triggering when
	-- the Rust side starts us with the --listen flag.
	if vim.fn.filereadable(pipe_dir) == 1 then
		notif("Unity adapter already running for this project (" .. pipe_dir .. ")", vim.log.levels.WARN)
		return
	end

	vim.fn.mkdir(temp_dir, "p")
	start_named_pipe_server(pipe_dir)
end

vim.api.nvim_create_autocmd("VimEnter", {
	callback = setup,
})
