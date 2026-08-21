extends Control

const TABLE_PREVIEW_ROWS := 12

var title_label: Label
var status_label: Label
var detail_label: RichTextLabel
var systems_box: VBoxContainer
var tables_list: ItemList
var table_preview: RichTextLabel
var runtime_log: RichTextLabel
var refresh_button: Button
var tick_button: Button
var headless_button: Button
var campaign_button: Button
var last_tables: Array = []

func _ready() -> void:
	_build_ui()
	CmApi.bootstrap_loaded.connect(_on_bootstrap_loaded)
	CmApi.request_failed.connect(_on_request_failed)
	_refresh()

func _build_ui() -> void:
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 14)
	root.offset_left = 28
	root.offset_top = 24
	root.offset_right = -28
	root.offset_bottom = -24
	add_child(root)

	title_label = Label.new()
	title_label.text = "CM0102 Rust Control Desk"
	title_label.add_theme_font_size_override("font_size", 42)
	root.add_child(title_label)

	status_label = Label.new()
	status_label.text = "Connecting to Rust backend..."
	status_label.add_theme_font_size_override("font_size", 19)
	root.add_child(status_label)

	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 10)
	root.add_child(actions)

	refresh_button = _action_button("Refresh Rust Backend", _refresh)
	actions.add_child(refresh_button)
	tick_button = _action_button("Tick Runtime 1 Day", _tick_runtime)
	actions.add_child(tick_button)
	headless_button = _action_button("Run Headless 1 Day", _run_headless_day)
	actions.add_child(headless_button)
	campaign_button = _action_button("Run Campaign 30 Days", _run_campaign)
	actions.add_child(campaign_button)

	detail_label = RichTextLabel.new()
	detail_label.fit_content = true
	detail_label.bbcode_enabled = true
	detail_label.scroll_active = false
	root.add_child(detail_label)

	var columns := HSplitContainer.new()
	columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(columns)

	var left := VBoxContainer.new()
	left.custom_minimum_size = Vector2(420, 0)
	left.add_theme_constant_override("separation", 10)
	columns.add_child(left)

	var systems_heading := Label.new()
	systems_heading.text = "Gameplay Promotion Gates"
	systems_heading.add_theme_font_size_override("font_size", 20)
	left.add_child(systems_heading)

	var systems_scroll := ScrollContainer.new()
	systems_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(systems_scroll)

	systems_box = VBoxContainer.new()
	systems_box.add_theme_constant_override("separation", 10)
	systems_scroll.add_child(systems_box)

	var right := VBoxContainer.new()
	right.add_theme_constant_override("separation", 10)
	columns.add_child(right)

	var tables_heading := Label.new()
	tables_heading.text = "Rust-Owned Tables"
	tables_heading.add_theme_font_size_override("font_size", 20)
	right.add_child(tables_heading)

	tables_list = ItemList.new()
	tables_list.custom_minimum_size = Vector2(0, 220)
	tables_list.item_selected.connect(_on_table_selected)
	right.add_child(tables_list)

	table_preview = RichTextLabel.new()
	table_preview.bbcode_enabled = true
	table_preview.size_flags_vertical = Control.SIZE_EXPAND_FILL
	right.add_child(table_preview)

	runtime_log = RichTextLabel.new()
	runtime_log.bbcode_enabled = true
	runtime_log.fit_content = true
	runtime_log.scroll_active = false
	root.add_child(runtime_log)

func _action_button(text: String, callback: Callable) -> Button:
	var button := Button.new()
	button.text = text
	button.pressed.connect(callback)
	return button

func _set_actions_enabled(enabled: bool) -> void:
	refresh_button.disabled = not enabled
	tick_button.disabled = not enabled
	headless_button.disabled = not enabled
	campaign_button.disabled = not enabled

func _refresh() -> void:
	_set_actions_enabled(false)
	status_label.text = "Loading from %s..." % CmApi.base_url
	detail_label.text = ""
	table_preview.text = ""
	runtime_log.text = ""
	tables_list.clear()
	for child in systems_box.get_children():
		child.queue_free()
	CmApi.load_bootstrap()

func _on_bootstrap_loaded(payload: Dictionary) -> void:
	_set_actions_enabled(true)
	if not payload.errors.is_empty():
		status_label.text = "Rust backend reachable with %s warning(s)" % payload.errors.size()
	else:
		status_label.text = "Rust backend connected"
	var control: Dictionary = payload.responses.get("promotion_control_room", {})
	var summary: Dictionary = control.get("summary", {})
	var runtime: Dictionary = payload.responses.get("runtime_save", {})
	var table_index: Dictionary = payload.responses.get("tables", {})
	last_tables = table_index.get("datasets", [])
	_render_summary(summary, runtime, last_tables, control.get("cache", {}))
	for system in control.get("systems", []):
		_add_system_card(system)
	_populate_tables(last_tables)

func _render_summary(summary: Dictionary, runtime: Dictionary, datasets: Array, cache: Dictionary) -> void:
	var save_date := _save_date(runtime)
	detail_label.text = "[b]Foundation:[/b] %s\n[b]Exact remake:[/b] %s\n[b]Playable headless:[/b] %s\n[b]Original capture:[/b] %s/%s rows, %s placeholders\n[b]Datasets:[/b] %s\n[b]Save date:[/b] %s\n[b]Control-room source:[/b] %s" % [
		summary.get("foundation_pass", false),
		summary.get("one_for_one_exact_remake", false),
		summary.get("playable_headless", false),
		summary.get("original_capture_rows_filled", 0),
		summary.get("original_capture_rows_expected", 0),
		summary.get("original_capture_placeholder_rows", 0),
		datasets.size(),
		save_date,
		cache.get("status", "live")
	]

func _save_date(save: Dictionary) -> String:
	var date: Dictionary = save.get("date", {})
	return "%04d-%02d-%02d phase %s" % [
		int(date.get("year", 0)),
		int(date.get("month", 0)),
		int(date.get("day", 0)),
		str(save.get("phase", "?"))
	]

func _populate_tables(datasets: Array) -> void:
	for dataset in datasets:
		var path := str(dataset.get("path", ""))
		var label := str(dataset.get("label", path))
		var rows := int(dataset.get("rows", 0))
		var index := tables_list.add_item("%s  (%s)" % [label, rows])
		tables_list.set_item_metadata(index, path)
	if tables_list.item_count > 0:
		tables_list.select(0)
		_on_table_selected(0)

func _add_system_card(system: Dictionary) -> void:
	var panel := PanelContainer.new()
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	panel.add_child(box)
	var heading := Label.new()
	heading.text = "%s  |  %s" % [system.get("system", "unknown"), system.get("slug", "")]
	heading.add_theme_font_size_override("font_size", 18)
	box.add_child(heading)
	var capture: Dictionary = system.get("original_capture", {})
	var parity: Dictionary = system.get("parity", {})
	var promotion: Dictionary = system.get("promotion", {})
	var body := Label.new()
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.text = "capture: %s (%s/%s filled, %s placeholders)\nparity: %s\npromotion: %s\nblockers: %s" % [
		capture.get("status", "unknown"),
		capture.get("filled_original_rows", 0),
		capture.get("expected_original_rows", 0),
		capture.get("placeholder_rows", 0),
		parity.get("status", "unknown"),
		promotion.get("status", "unknown"),
		str(promotion.get("open_blockers", []))
	]
	box.add_child(body)
	systems_box.add_child(panel)

func _on_table_selected(index: int) -> void:
	var table_path := str(tables_list.get_item_metadata(index))
	table_preview.text = "Loading [b]%s[/b]..." % table_path
	var response := await CmApi.load_table(table_path)
	if response.has("error"):
		table_preview.text = "[b]Table load failed[/b]\n%s" % response.error
		return
	var rows: Array = response.get("rows", [])
	var out := "[b]%s[/b]\n%s row(s)\n\n" % [response.get("label", table_path), rows.size()]
	for i in range(min(TABLE_PREVIEW_ROWS, rows.size())):
		out += "[b]Row %s[/b] %s\n" % [i, _compact_row(rows[i])]
	table_preview.text = out

func _compact_row(value: Variant) -> String:
	var text := JSON.stringify(value)
	if text.length() > 260:
		return text.substr(0, 260) + "..."
	return text

func _tick_runtime() -> void:
	await _run_backend_action("Ticking runtime one day", "/api/runtime-save/tick", {"days": 1})

func _run_headless_day() -> void:
	await _run_backend_action("Running headless one day", "/api/headless/run", {"days": 1})

func _run_campaign() -> void:
	await _run_backend_action("Running 30-day campaign", "/api/headless/campaign", {"days": 30, "checkpoint_every": 10})

func _run_backend_action(label: String, endpoint: String, payload: Dictionary) -> void:
	_set_actions_enabled(false)
	runtime_log.text = "%s..." % label
	var response: Dictionary = await CmApi.post_json(endpoint, payload)
	_set_actions_enabled(true)
	if response.has("error"):
		runtime_log.text = "[b]%s failed[/b]\n%s" % [label, response.error]
		return
	var save: Dictionary = response.get("save", response)
	var report: Dictionary = response.get("report", {})
	runtime_log.text = "[b]%s complete[/b]\nSave date: %s\nDays advanced: %s\nPhases advanced: %s\nBackend blockers stay expected until exact mutators are promoted." % [
		label,
		_save_date(save),
		report.get("days_advanced", "n/a"),
		report.get("phases_advanced", "n/a")
	]
	_refresh()

func _on_request_failed(endpoint: String, message: String) -> void:
	_set_actions_enabled(true)
	status_label.text = "Rust backend request failed"
	detail_label.text = "[b]%s[/b]\n%s\n\nStart it with:\n[code]cargo run -p cm-app -- serve-rust-db D:/cm0102-rs/rust-db 8770[/code]" % [endpoint, message]
