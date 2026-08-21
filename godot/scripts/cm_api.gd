extends Node

signal bootstrap_loaded(payload: Dictionary)
signal request_failed(endpoint: String, message: String)

const DEFAULT_BASE_URL := "http://127.0.0.1:8770"

var base_url := DEFAULT_BASE_URL
var last_bootstrap: Dictionary = {}

func _ready() -> void:
	var configured := OS.get_environment("CM0102_RS_API")
	if not configured.is_empty():
		base_url = configured.rstrip("/")

func get_json(endpoint: String) -> void:
	var request := HTTPRequest.new()
	add_child(request)
	request.request_completed.connect(_on_json_completed.bind(request, endpoint), CONNECT_ONE_SHOT)
	var error := request.request(base_url + endpoint)
	if error != OK:
		request_failed.emit(endpoint, "Could not start request: %s" % error)
		request.queue_free()

func load_bootstrap() -> void:
	var endpoints := {
		"health": "/api/health",
		"backend_acceptance": "/api/backend-acceptance",
		"promotion_control_room": "/api/promotion-control-room-cached",
		"runtime_save": "/api/runtime-save",
		"tables": "/api/tables"
	}
	var payload := {
		"base_url": base_url,
		"endpoints": endpoints,
		"responses": {},
		"errors": []
	}
	for key in endpoints.keys():
		var response := await fetch_json(endpoints[key])
		if response.has("error"):
			payload.errors.append({"key": key, "endpoint": endpoints[key], "error": response.error})
		else:
			payload.responses[key] = response
	last_bootstrap = payload
	bootstrap_loaded.emit(payload)

func load_table(table_path: String) -> Dictionary:
	return await fetch_json("/api/table/%s" % table_path.uri_encode())

func tick_runtime(days: int = 1) -> Dictionary:
	return await post_json("/api/runtime-save/tick", {"days": days})

func run_headless(days: int = 1) -> Dictionary:
	return await post_json("/api/headless/run", {"days": days})

func run_campaign(days: int = 30, checkpoint_every: int = 10) -> Dictionary:
	return await post_json("/api/headless/campaign", {"days": days, "checkpoint_every": checkpoint_every})

func fetch_json(endpoint: String) -> Dictionary:
	var request := HTTPRequest.new()
	add_child(request)
	var started := request.request(base_url + endpoint)
	if started != OK:
		request.queue_free()
		return {"error": "Could not start request: %s" % started}
	var result: Array = await request.request_completed
	request.queue_free()
	var status: int = result[1]
	var body: PackedByteArray = result[3]
	if status < 200 or status >= 300:
		return {"error": "HTTP %s from %s" % [status, endpoint]}
	var text := body.get_string_from_utf8()
	var parsed: Variant = JSON.parse_string(text)
	if typeof(parsed) != TYPE_DICTIONARY:
		return {"error": "Invalid JSON from %s" % endpoint}
	return parsed

func post_json(endpoint: String, payload: Dictionary) -> Dictionary:
	var request := HTTPRequest.new()
	add_child(request)
	var body := JSON.stringify(payload)
	var headers := PackedStringArray(["Content-Type: application/json"])
	var started := request.request(base_url + endpoint, headers, HTTPClient.METHOD_POST, body)
	if started != OK:
		request.queue_free()
		return {"error": "Could not start request: %s" % started}
	var result: Array = await request.request_completed
	request.queue_free()
	var status: int = result[1]
	var response_body: PackedByteArray = result[3]
	if status < 200 or status >= 300:
		return {"error": "HTTP %s from %s: %s" % [status, endpoint, response_body.get_string_from_utf8()]}
	var parsed: Variant = JSON.parse_string(response_body.get_string_from_utf8())
	if typeof(parsed) != TYPE_DICTIONARY:
		return {"error": "Invalid JSON from %s" % endpoint}
	return parsed

func _on_json_completed(result: int, response_code: int, _headers: PackedStringArray, body: PackedByteArray, request: HTTPRequest, endpoint: String) -> void:
	request.queue_free()
	if result != HTTPRequest.RESULT_SUCCESS or response_code < 200 or response_code >= 300:
		request_failed.emit(endpoint, "HTTP request failed: result %s, status %s" % [result, response_code])
		return
	var parsed: Variant = JSON.parse_string(body.get_string_from_utf8())
	if typeof(parsed) != TYPE_DICTIONARY:
		request_failed.emit(endpoint, "Response was not a JSON object")
		return
